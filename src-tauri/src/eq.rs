//! Égaliseur 10 bandes d'octave (32 Hz à 16 kHz, lot 0.13.0), implémenté comme une cascade
//! de filtres "peaking" biquad (formules RBJ Audio EQ Cookbook), appliqués en direct sur le
//! flux audio via un `rodio::Source` maison. Les gains sont partagés dans un
//! `Arc<Mutex<Gains>>` pour être ajustés pendant la lecture, sans reconstruire le pipeline
//! audio. Une bande à 0 dB est sautée (moins de calcul sur les téléphones modestes).
//! Les réglages des versions à 3 bandes (basses, médiums, aigus) sont convertis à la lecture.

use crate::state::Locked;
use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Nombre de bandes et leurs fréquences centrales (Hz), une par octave.
pub const BANDS: usize = 10;
pub const BAND_FREQS: [f32; BANDS] = [32.0, 64.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
/// Largeur d'une octave : les bandes voisines se recouvrent juste assez pour une courbe lisse.
const Q: f32 = 1.41;

/// Gains en dB des 10 bandes.
pub type Gains = [f32; BANDS];

/// Réglage d'une ancienne version à 3 bandes (basses 120 Hz, médiums 1 kHz, aigus 8 kHz),
/// réparti sur les bandes les plus proches.
pub fn from_three([bass, mid, treble]: [f32; 3]) -> Gains {
    [bass, bass, bass, bass, mid, mid, mid, treble, treble, treble]
}

/// Gains lus d'un fichier : 10 valeurs, ou 3 d'une ancienne version (converties) ; bornés.
pub fn gains_from_values(values: &[f32]) -> Option<Gains> {
    let gains = match values.len() {
        BANDS => {
            let mut g = [0.0; BANDS];
            g.copy_from_slice(values);
            g
        }
        3 => from_three([values[0], values[1], values[2]]),
        _ => return None,
    };
    Some(gains.map(|g| if g.is_finite() { g.clamp(-12.0, 12.0) } else { 0.0 }))
}

/// Désérialisation tolérante (session.json d'une version à 3 bandes).
pub fn deserialize_gains<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Gains, D::Error> {
    let values: Vec<f32> = serde::Deserialize::deserialize(deserializer)?;
    Ok(gains_from_values(&values).unwrap_or([0.0; BANDS]))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

/// Coefficients d'un filtre "peaking EQ" (formule RBJ Audio EQ Cookbook),
/// déjà normalisés par `a0`.
pub fn peaking_eq_coeffs(sample_rate: f32, freq: f32, q: f32, gain_db: f32) -> BiquadCoeffs {
    let a = 10f32.powf(gain_db / 40.0);
    let w0 = 2.0 * std::f32::consts::PI * freq / sample_rate;
    let alpha = w0.sin() / (2.0 * q);
    let cos_w0 = w0.cos();

    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * cos_w0;
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha / a;

    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn process(&mut self, x0: f32, c: &BiquadCoeffs) -> f32 {
        let y0 = c.b0 * x0 + c.b1 * self.x1 + c.b2 * self.x2 - c.a1 * self.y1 - c.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        y0
    }
}

/// Gains courants en dB des bandes, partagés entre le thread audio et
/// les commandes Tauri qui les modifient depuis le frontend.
pub type EqGains = Arc<Mutex<Gains>>;

pub fn new_eq_gains(initial: Gains) -> EqGains {
    Arc::new(Mutex::new(initial))
}

/// Coefficients des bandes actives (gain non nul, sous la fréquence de Nyquist), dans l'ordre.
fn compute_active(sample_rate: f32, gains: &Gains) -> Vec<(usize, BiquadCoeffs)> {
    (0..BANDS)
        .filter(|&i| gains[i].abs() >= 0.05 && BAND_FREQS[i] < sample_rate * 0.45)
        .map(|i| (i, peaking_eq_coeffs(sample_rate, BAND_FREQS[i], Q, gains[i])))
        .collect()
}

pub struct EqSource<I> {
    input: I,
    channels: ChannelCount,
    sample_rate: SampleRate,
    gains: EqGains,
    cached_gains: Gains,
    /// Bandes actives (numéro de bande, coefficients) : les bandes à 0 dB sont sautées.
    active: Vec<(usize, BiquadCoeffs)>,
    /// État des filtres par canal (index = numéro de canal) puis par bande.
    state: Vec<[BiquadState; BANDS]>,
    channel_cursor: usize,
    /// Égaliseur neutre (gains nuls) : les filtres sont sautés, le son est identique et
    /// le processeur du téléphone travaille beaucoup moins.
    flat: bool,
}

/// Les gains ne sont relus qu'une fois tous ces échantillons (~20 fois par seconde),
/// au lieu de prendre un verrou à chaque échantillon.
const GAIN_CHECK_INTERVAL: usize = 2048;

fn is_flat(gains: &Gains) -> bool {
    gains.iter().all(|g| g.abs() < 0.05)
}

impl<I> EqSource<I>
where
    I: Source<Item = f32>,
{
    pub fn new(input: I, gains: EqGains) -> Self {
        let channels = input.channels();
        let sample_rate = input.sample_rate();
        let cached_gains = *gains.locked();
        let active = compute_active(sample_rate.get() as f32, &cached_gains);
        let state = vec![[BiquadState::default(); BANDS]; channels.get() as usize];
        Self {
            input,
            channels,
            sample_rate,
            gains,
            cached_gains,
            active,
            state,
            channel_cursor: 0,
            flat: is_flat(&cached_gains),
        }
    }
}

impl<I> Iterator for EqSource<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.input.next()?;

        // On ne bloque jamais le thread audio pour aller lire un changement de gain : relu
        // de temps en temps, et si le verrou est momentanément pris par le thread des
        // commandes, on garde les coefficients précédents, imperceptible à l'oreille.
        if self.channel_cursor.is_multiple_of(GAIN_CHECK_INTERVAL) {
            if let Ok(current) = self.gains.try_lock() {
                if *current != self.cached_gains {
                    self.cached_gains = *current;
                    self.active = compute_active(self.sample_rate.get() as f32, &self.cached_gains);
                    self.flat = is_flat(&self.cached_gains);
                }
            }
        }

        let channel = self.channel_cursor % self.channels.get() as usize;
        self.channel_cursor = self.channel_cursor.wrapping_add(1);
        if self.flat {
            return Some(sample);
        }

        let channel_state = &mut self.state[channel];
        let mut out = sample;
        for (band, coeffs) in &self.active {
            out = channel_state[*band].process(out, coeffs);
        }
        Some(out)
    }
}

impl<I> Source for EqSource<I>
where
    I: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        self.input.current_span_len()
    }
    fn channels(&self) -> ChannelCount {
        self.channels
    }
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
    /// Transmis à la source : sans cela, le curseur de position ne pouvait rien déplacer.
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.input.try_seek(pos)?;
        self.state
            .iter_mut()
            .for_each(|s| *s = [BiquadState::default(); BANDS]);
        self.channel_cursor = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Source de test minimaliste : rejoue un buffer f32 fixe, mono,
    /// à un taux d'échantillonnage donné.
    struct TestSource {
        samples: std::vec::IntoIter<f32>,
        sample_rate: u32,
        channels: u16,
    }

    impl TestSource {
        fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
            Self {
                samples: samples.into_iter(),
                sample_rate,
                channels,
            }
        }
    }

    impl Iterator for TestSource {
        type Item = f32;
        fn next(&mut self) -> Option<f32> {
            self.samples.next()
        }
    }

    impl Source for TestSource {
        fn current_span_len(&self) -> Option<usize> {
            None
        }
        fn channels(&self) -> ChannelCount {
            ChannelCount::new(self.channels).unwrap()
        }
        fn sample_rate(&self) -> SampleRate {
            SampleRate::new(self.sample_rate).unwrap()
        }
        fn total_duration(&self) -> Option<Duration> {
            None
        }
    }

    fn test_signal(len: usize) -> Vec<f32> {
        (0..len).map(|i| (i as f32 * 0.37).sin() * 0.5).collect()
    }

    #[test]
    fn zero_gain_band_is_numerically_an_identity_filter() {
        // Formule RBJ : à 0 dB, A = 1, donc b_i == a_i, ce qui donne une
        // fonction de transfert H(z) = 1 sur tout le plan z : la sortie
        // doit être identique à l'entrée à la précision flottante près.
        let coeffs = peaking_eq_coeffs(44_100.0, 1_000.0, 0.9, 0.0);
        let mut state = BiquadState::default();
        let input = test_signal(64);
        for &x in &input {
            let y = state.process(x, &coeffs);
            assert!((y - x).abs() < 1e-4, "attendu {x}, obtenu {y}");
        }
    }

    #[test]
    fn eq_source_at_zero_gain_leaves_audio_unchanged() {
        let input = test_signal(200);
        let source = TestSource::new(input.clone(), 44_100, 1);
        let gains = new_eq_gains([0.0; BANDS]);
        let eq = EqSource::new(source, gains);
        let output: Vec<f32> = eq.collect();
        assert_eq!(output.len(), input.len());
        for (x, y) in input.iter().zip(output.iter()) {
            assert!((x - y).abs() < 1e-3, "{x} vs {y}");
        }
    }

    #[test]
    fn eq_source_preserves_channel_count_and_sample_rate() {
        let source = TestSource::new(test_signal(16), 48_000, 2);
        let gains = new_eq_gains(from_three([2.0, -3.0, 0.0]));
        let eq = EqSource::new(source, gains);
        assert_eq!(eq.channels().get(), 2);
        assert_eq!(eq.sample_rate().get(), 48_000);
    }

    #[test]
    fn boosting_a_band_does_not_produce_nan_or_infinite_output() {
        let input = test_signal(1000);
        let source = TestSource::new(input, 44_100, 1);
        let gains = new_eq_gains([12.0; BANDS]); // boost max
        let eq = EqSource::new(source, gains);
        for sample in eq {
            assert!(
                sample.is_finite(),
                "l'égaliseur a produit une valeur non finie"
            );
        }
    }

    #[test]
    fn live_gain_change_is_picked_up_without_rebuilding_the_source() {
        let input = test_signal(3 * GAIN_CHECK_INTERVAL);
        let source = TestSource::new(input.clone(), 44_100, 1);
        let gains = new_eq_gains([0.0; BANDS]);
        let gains_handle = gains.clone();
        let mut eq = EqSource::new(source, gains);

        // Consomme quelques échantillons à gain nul (égaliseur neutre : son inchangé)...
        for (i, expected) in input.iter().enumerate().take(50) {
            assert_eq!(eq.next(), Some(*expected), "échantillon {i}");
        }
        // ...puis change le gain "en direct" comme le ferait une commande
        // Tauri pendant la lecture.
        *gains_handle.locked() = from_three([6.0, 0.0, 0.0]);

        let rest: Vec<f32> = eq.collect();
        assert!(rest.iter().all(|s| s.is_finite()));
        // Pris en compte au plus tard après GAIN_CHECK_INTERVAL échantillons.
        let tail = &rest[GAIN_CHECK_INTERVAL..];
        let original = &input[50 + GAIN_CHECK_INTERVAL..];
        assert!(tail.iter().zip(original).any(|(a, b)| (a - b).abs() > 1e-4));
    }

    #[test]
    fn old_three_band_settings_are_spread_over_ten_bands() {
        assert_eq!(gains_from_values(&[6.0, -2.0, 3.0]).unwrap(), [6.0, 6.0, 6.0, 6.0, -2.0, -2.0, -2.0, 3.0, 3.0, 3.0]);
        let ten: Vec<f32> = (0..10).map(|i| i as f32).collect();
        assert_eq!(gains_from_values(&ten).unwrap()[9], 9.0);
        assert_eq!(gains_from_values(&[40.0, f32::NAN, -40.0]).unwrap()[0], 12.0);
        assert_eq!(gains_from_values(&[40.0, f32::NAN, -40.0]).unwrap()[5], 0.0);
        assert!(gains_from_values(&[1.0, 2.0]).is_none());
    }

    #[test]
    fn a_boosted_band_raises_its_own_frequency_much_more_than_a_far_one() {
        // Sinusoïde pure à 64 Hz puis à 8 kHz, bande de 64 Hz montée de 12 dB.
        let rms_after = |freq: f32| {
            let n = 44_100;
            let input: Vec<f32> = (0..n).map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 44_100.0).sin() * 0.25).collect();
            let mut gains = [0.0; BANDS];
            gains[1] = 12.0;
            let out: Vec<f32> = EqSource::new(TestSource::new(input, 44_100, 1), new_eq_gains(gains)).collect();
            let tail = &out[n / 2..];
            (tail.iter().map(|s| s * s).sum::<f32>() / tail.len() as f32).sqrt() / (0.25 / 2f32.sqrt())
        };
        let low = rms_after(64.0);
        let high = rms_after(8000.0);
        assert!(low > 3.5, "64 Hz amplifié d'environ x4 : {low}");
        assert!((high - 1.0).abs() < 0.05, "8 kHz inchangé : {high}");
    }

    #[test]
    fn bands_at_zero_or_above_nyquist_are_skipped() {
        let mut gains = [0.0; BANDS];
        gains[5] = 3.0;
        gains[9] = 6.0;
        assert_eq!(compute_active(44_100.0, &gains).iter().map(|(b, _)| *b).collect::<Vec<_>>(), vec![5, 9]);
        // 16 kHz n'est pas sous 0,45 × 22 050 Hz : sautée.
        assert_eq!(compute_active(22_050.0, &gains).iter().map(|(b, _)| *b).collect::<Vec<_>>(), vec![5]);
    }
}
