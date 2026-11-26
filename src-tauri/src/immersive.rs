//! Mode immersif (lot 0.14.0) : son qui tourne autour de la tête (« 8D ») et acoustique de
//! salle, appliqués en direct sur le flux audio, juste après l'égaliseur.
//!
//! - **Rotation 8D** : le son (ramené au centre) fait le tour de la tête. Deux moteurs :
//!   - *tête simplifiée* : modèle sphérique de Brown et Duda (1998) : écart de temps entre
//!     les deux oreilles et ombre de la tête (filtre du premier ordre), plus un léger
//!     assourdissement quand le son passe derrière. Aucun fichier de données, très léger ;
//!   - *HRTF mesuré* : réponses d'une vraie tête de mesure (base SADIE II, sujet D1,
//!     Neumann KU100, Université d'York, licence Apache 2.0 : `assets/hrir/`), plan
//!     horizontal tous les 5°, 256 coefficients, interpolées entre deux directions voisines.
//! - **Salle** : réverbération de type Freeverb (8 filtres en peigne et 4 passe-tout par
//!   canal ; 4 et 2 en mode léger), taille, amortissement, pré-délai et dosage réglables.
//!
//! Les réglages sont publiés par les commandes dans un emplacement global avec un numéro de
//! version : le thread audio ne lit qu'un entier atomique par bloc de 32 trames et ne prend
//! le verrou que si les réglages ont changé (jamais d'attente pendant la lecture). Les
//! changements (activation, dosage) passent par des rampes : pas de clic.
//!
//! Le son sort toujours en stéréo (un fichier mono est dédoublé) ; un flux à plus de deux
//! canaux passe sans effet.

use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

// ---------------------------------------------------------------------
// Réglages
// ---------------------------------------------------------------------

/// Moteur de la rotation 8D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Engine3d {
    /// Modèle de tête sphérique calculé.
    #[default]
    Head,
    /// Mesures d'une vraie tête (SADIE II).
    Hrtf,
}

/// Réglages du mode immersif (globaux, ou propres à une playlist).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ImmersiveSettings {
    /// Interrupteur général.
    pub enabled: bool,
    /// Rotation 8D.
    pub rotation: bool,
    pub engine: Engine3d,
    /// Vitesse de rotation, en tours par minute (2 à 40).
    pub speed: f32,
    /// Sens des aiguilles d'une montre (vu du dessus, en partant de devant vers la droite).
    pub clockwise: bool,
    /// Dosage de l'effet 8D (0 : son d'origine, 1 : effet complet).
    pub intensity: f32,
    /// Acoustique de salle.
    pub room: bool,
    /// Préréglage choisi (affichage seulement ; les valeurs ci-dessous font foi).
    pub room_preset: String,
    /// Taille de la salle (0 à 1).
    pub room_size: f32,
    /// Amortissement des aigus (0 : salle claire, 1 : salle mate).
    pub room_damping: f32,
    /// Dosage de la réverbération (0 à 1).
    pub room_mix: f32,
    /// Pré-délai (ms), temps avant les premières réflexions (0 à 120).
    pub room_predelay_ms: f32,
}

impl Default for ImmersiveSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            rotation: true,
            engine: Engine3d::Head,
            speed: 10.0,
            clockwise: true,
            intensity: 0.85,
            room: false,
            room_preset: "concert".to_string(),
            room_size: 0.72,
            room_damping: 0.45,
            room_mix: 0.3,
            room_predelay_ms: 25.0,
        }
    }
}

fn finite_clamp(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

impl ImmersiveSettings {
    /// Bornes acceptées par le moteur (fichier modifié à la main, ancienne version).
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        self.speed = finite_clamp(self.speed, 2.0, 40.0, d.speed);
        self.intensity = finite_clamp(self.intensity, 0.0, 1.0, d.intensity);
        self.room_size = finite_clamp(self.room_size, 0.0, 1.0, d.room_size);
        self.room_damping = finite_clamp(self.room_damping, 0.0, 1.0, d.room_damping);
        self.room_mix = finite_clamp(self.room_mix, 0.0, 1.0, d.room_mix);
        self.room_predelay_ms = finite_clamp(self.room_predelay_ms, 0.0, 120.0, d.room_predelay_ms);
        self.room_preset.truncate(40);
        self
    }
}

/// Réglages enregistrés dans `session.json` : globaux, propres à des playlists, et playlist
/// d'où vient la lecture en cours (dont l'ambiance s'applique si elle en a une).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ImmersiveProfiles {
    pub global: ImmersiveSettings,
    pub playlists: BTreeMap<String, ImmersiveSettings>,
    pub context: Option<String>,
}

impl ImmersiveProfiles {
    pub fn sanitized(mut self) -> Self {
        self.global = self.global.sanitized();
        self.playlists = std::mem::take(&mut self.playlists)
            .into_iter()
            .filter(|(id, _)| !id.trim().is_empty())
            .map(|(id, s)| (id, s.sanitized()))
            .collect();
        self.context = self.context.filter(|id| !id.trim().is_empty());
        self
    }

    /// Réglages qui s'appliquent : ceux de la playlist en cours si elle en a, sinon les globaux.
    pub fn effective(&self) -> &ImmersiveSettings {
        self.context
            .as_ref()
            .and_then(|id| self.playlists.get(id))
            .unwrap_or(&self.global)
    }
}

// ---------------------------------------------------------------------
// Publication vers le thread audio
// ---------------------------------------------------------------------

/// Paramètres effectivement appliqués par le moteur.
#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub settings: ImmersiveSettings,
    /// « Comparer » maintenu : le son passe sans effet, les réglages restent.
    pub bypass: bool,
    /// Mode léger : réverbération allégée (moitié moins de filtres).
    pub lite: bool,
}

impl Params {
    fn rotation_on(&self) -> bool {
        !self.bypass && self.settings.enabled && self.settings.rotation
    }
    fn room_on(&self) -> bool {
        !self.bypass && self.settings.enabled && self.settings.room
    }
}

static PARAMS: Mutex<Option<Params>> = Mutex::new(None);
static VERSION: AtomicU64 = AtomicU64::new(0);
static BYPASS: AtomicBool = AtomicBool::new(false);
static LITE: AtomicBool = AtomicBool::new(false);
static LAST: Mutex<Option<ImmersiveSettings>> = Mutex::new(None);

fn store(params: Params) {
    *PARAMS.lock().unwrap_or_else(|e| e.into_inner()) = Some(params);
    VERSION.fetch_add(1, Ordering::SeqCst);
}

fn republish() {
    let settings = LAST.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_default();
    store(Params { settings, bypass: BYPASS.load(Ordering::SeqCst), lite: LITE.load(Ordering::SeqCst) });
}

/// Applique des réglages (déjà choisis entre globaux et playlist) au son en cours.
pub fn publish(settings: &ImmersiveSettings) {
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(settings.clone().sanitized());
    republish();
}

pub fn set_bypass(on: bool) {
    BYPASS.store(on, Ordering::SeqCst);
    republish();
}

pub fn set_lite(on: bool) {
    LITE.store(on, Ordering::SeqCst);
    republish();
}

fn current_params() -> Params {
    PARAMS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or(Params { settings: ImmersiveSettings::default(), bypass: false, lite: false })
}

// ---------------------------------------------------------------------
// Tables HRTF (SADIE II, sujet D1)
// ---------------------------------------------------------------------

/// Directions de la table : tous les 5°, sens trigonométrique (0 : devant, 90 : à gauche).
const HRTF_DIRECTIONS: usize = 72;
const HRTF_STEP_DEG: f32 = 5.0;
const HRTF_TAPS: usize = 256;

static HRIR_44100: &[u8] = include_bytes!("../assets/hrir/sadie2_d1_44100.f32");
static HRIR_48000: &[u8] = include_bytes!("../assets/hrir/sadie2_d1_48000.f32");

/// Réponses d'une direction et d'une oreille ; `taps` coefficients, déjà adaptés à la
/// fréquence d'échantillonnage du morceau.
struct HrirTable {
    taps: usize,
    /// [direction][oreille][coefficient], oreille 0 = gauche.
    data: Vec<f32>,
    /// Retard propre des réponses (phase linéaire) : le son d'origine est retardé d'autant
    /// pour que le mélange des deux ne creuse pas le spectre.
    latency: usize,
}

impl HrirTable {
    fn ir(&self, direction: usize, ear: usize) -> &[f32] {
        let start = (direction * 2 + ear) * self.taps;
        &self.data[start..start + self.taps]
    }
}

fn decode_table(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect()
}

/// Table à la fréquence `rate` : 44,1 et 48 kHz telles quelles (mesures d'origine) ;
/// toute autre fréquence est rééchantillonnée depuis 48 kHz (interpolation linéaire).
fn build_table(rate: u32) -> HrirTable {
    let (data, taps) = match rate {
        44_100 => (decode_table(HRIR_44100), HRTF_TAPS),
        48_000 => (decode_table(HRIR_48000), HRTF_TAPS),
        _ => {
            let source = decode_table(HRIR_48000);
            let ratio = 48_000.0 / rate as f32;
            let taps = ((HRTF_TAPS as f32 / ratio).ceil() as usize).clamp(16, 1024);
            let mut data = vec![0.0; HRTF_DIRECTIONS * 2 * taps];
            for ir in 0..HRTF_DIRECTIONS * 2 {
                let src = &source[ir * HRTF_TAPS..(ir + 1) * HRTF_TAPS];
                for k in 0..taps {
                    let pos = k as f32 * ratio;
                    let i = pos.floor() as usize;
                    let frac = pos - i as f32;
                    let a = src.get(i).copied().unwrap_or(0.0);
                    let b = src.get(i + 1).copied().unwrap_or(0.0);
                    // Plus de coefficients à fréquence plus haute : chacun pèse d'autant moins (même gain).
                    data[ir * taps + k] = (a + (b - a) * frac) * ratio;
                }
            }
            (data, taps)
        }
    };
    let mut table = HrirTable { taps, data, latency: 0 };
    // Retard propre : position du pic des deux oreilles pour un son de face.
    let front: Vec<f32> = table.ir(0, 0).iter().zip(table.ir(0, 1)).map(|(l, r)| l.abs() + r.abs()).collect();
    table.latency = front
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .unwrap_or(0);
    table
}

fn table_for(rate: u32) -> &'static HrirTable {
    static T44: OnceLock<HrirTable> = OnceLock::new();
    static T48: OnceLock<HrirTable> = OnceLock::new();
    static OTHER: Mutex<Vec<(u32, &'static HrirTable)>> = Mutex::new(Vec::new());
    match rate {
        44_100 => T44.get_or_init(|| build_table(44_100)),
        48_000 => T48.get_or_init(|| build_table(48_000)),
        _ => {
            let mut other = OTHER.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((_, t)) = other.iter().find(|(r, _)| *r == rate) {
                return t;
            }
            // Quelques fréquences au plus dans la vie de l'appli (88,2 ; 96 ; 22,05 kHz…).
            let table: &'static HrirTable = Box::leak(Box::new(build_table(rate)));
            other.push((rate, table));
            table
        }
    }
}

/// Produit scalaire en 8 accumulateurs (vectorisé par le compilateur).
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (chunks_a, rest_a) = a[..n].as_chunks::<8>();
    let (chunks_b, rest_b) = b[..n].as_chunks::<8>();
    let mut acc = [0.0f32; 8];
    for (ca, cb) in chunks_a.iter().zip(chunks_b) {
        for i in 0..8 {
            acc[i] += ca[i] * cb[i];
        }
    }
    let rest: f32 = rest_a.iter().zip(rest_b).map(|(x, y)| x * y).sum();
    acc.iter().sum::<f32>() + rest
}

// ---------------------------------------------------------------------
// Briques de traitement
// ---------------------------------------------------------------------

/// Ligne à retard circulaire, lue avec un retard fractionnaire (interpolation linéaire).
struct DelayLine {
    buf: Vec<f32>,
    pos: usize,
}

impl DelayLine {
    fn new(len: usize) -> Self {
        Self { buf: vec![0.0; len.max(2)], pos: 0 }
    }
    fn push(&mut self, x: f32) {
        self.pos = (self.pos + 1) % self.buf.len();
        self.buf[self.pos] = x;
    }
    /// Échantillon d'il y a `delay` échantillons (0 : le dernier poussé).
    fn read(&self, delay: f32) -> f32 {
        let len = self.buf.len();
        let delay = delay.clamp(0.0, (len - 2) as f32);
        let whole = delay.floor() as usize;
        let frac = delay - whole as f32;
        let a = self.buf[(self.pos + len - whole) % len];
        let b = self.buf[(self.pos + len - whole - 1) % len];
        a + (b - a) * frac
    }
    fn clear(&mut self) {
        self.buf.iter_mut().for_each(|x| *x = 0.0);
    }
}

/// Filtre du premier ordre y = b0·x + b1·x₁ − a1·y₁.
#[derive(Default, Clone, Copy)]
struct OnePole {
    b0: f32,
    b1: f32,
    a1: f32,
    x1: f32,
    y1: f32,
}

impl OnePole {
    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 - self.a1 * self.y1;
        self.x1 = x;
        self.y1 = if y.abs() < 1e-25 { 0.0 } else { y };
        self.y1
    }
    fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }
}

/// Rayon de tête moyen (m) et vitesse du son (m/s), modèle de Brown et Duda.
const HEAD_RADIUS: f32 = 0.0875;
const SPEED_OF_SOUND: f32 = 343.0;
const ALPHA_MIN: f32 = 0.1;
const THETA_MIN: f32 = 5.0 * PI / 6.0;

/// Ombre de la tête pour une oreille : H(s) = (α·s + β) / (s + β), β = 2c/a, discrétisé par
/// la transformation bilinéaire. `incidence` : angle entre le son et l'axe de l'oreille.
fn head_shadow(sample_rate: f32, incidence: f32) -> (f32, f32, f32) {
    let alpha = (1.0 + ALPHA_MIN / 2.0) + (1.0 - ALPHA_MIN / 2.0) * (incidence * PI / THETA_MIN).cos();
    let beta = 2.0 * SPEED_OF_SOUND / HEAD_RADIUS;
    let k = 2.0 * sample_rate;
    let norm = k + beta;
    ((alpha * k + beta) / norm, (beta - alpha * k) / norm, (beta - k) / norm)
}

/// Retard (s) d'une oreille selon l'incidence, décalé pour rester positif.
fn head_delay(incidence: f32) -> f32 {
    let a_c = HEAD_RADIUS / SPEED_OF_SOUND;
    if incidence < FRAC_PI_2 {
        a_c * (1.0 - incidence.cos())
    } else {
        a_c * (1.0 + incidence - FRAC_PI_2)
    }
}

/// Angle (0 à π) entre la direction du son `azimuth` (0 devant, sens trigonométrique) et
/// l'axe d'une oreille (gauche : +90°, droite : −90°).
fn incidence(azimuth: f32, left: bool) -> f32 {
    let cos = if left { azimuth.sin() } else { -azimuth.sin() };
    cos.clamp(-1.0, 1.0).acos()
}

/// Nombre de trames entre deux mises à jour (direction, filtres, réglages).
const BLOCK: usize = 32;

/// Rotation 8D, modèle de tête simplifié.
struct HeadModel {
    delay: DelayLine,
    shadow: [OnePole; 2],
    /// Assourdissement quand le son passe derrière (passe-bas vers 4 kHz).
    back: [OnePole; 2],
    delay_from: [f32; 2],
    delay_to: [f32; 2],
    back_amount: f32,
    sample_rate: f32,
}

impl HeadModel {
    fn new(sample_rate: f32) -> Self {
        let max_delay = (HEAD_RADIUS / SPEED_OF_SOUND * (1.0 + FRAC_PI_2) * sample_rate).ceil() as usize + 4;
        let mut back = OnePole::default();
        let x = (-TAU * 4000.0 / sample_rate).exp();
        back.b0 = 1.0 - x;
        back.a1 = -x;
        Self {
            delay: DelayLine::new(max_delay + 2),
            shadow: [OnePole::default(); 2],
            back: [back; 2],
            delay_from: [0.0; 2],
            delay_to: [0.0; 2],
            back_amount: 0.0,
            sample_rate,
        }
    }

    /// Retard ajouté au son d'origine pour rester aligné avec le son spatialisé (de face).
    fn latency(&self) -> f32 {
        head_delay(FRAC_PI_2) * self.sample_rate
    }

    fn set_direction(&mut self, azimuth: f32) {
        for (ear, left) in [(0, true), (1, false)] {
            let inc = incidence(azimuth, left);
            let (b0, b1, a1) = head_shadow(self.sample_rate, inc);
            let f = &mut self.shadow[ear];
            f.b0 = b0;
            f.b1 = b1;
            f.a1 = a1;
            self.delay_from[ear] = self.delay_to[ear];
            self.delay_to[ear] = head_delay(inc) * self.sample_rate;
        }
        self.back_amount = 0.5 * (-azimuth.cos()).max(0.0);
    }

    fn process(&mut self, mono: f32, step: usize) -> (f32, f32) {
        self.delay.push(mono);
        let t = step as f32 / BLOCK as f32;
        let mut out = [0.0; 2];
        for (ear, sample) in out.iter_mut().enumerate() {
            let d = self.delay_from[ear] + (self.delay_to[ear] - self.delay_from[ear]) * t;
            let x = self.shadow[ear].process(self.delay.read(d));
            let muffled = self.back[ear].process(x);
            *sample = x + (muffled - x) * self.back_amount;
        }
        (out[0], out[1])
    }

    fn reset(&mut self) {
        self.delay.clear();
        self.shadow.iter_mut().chain(self.back.iter_mut()).for_each(OnePole::reset);
    }
}

/// Rotation 8D avec les réponses mesurées (convolution directe, 2 oreilles).
struct HrtfModel {
    table: &'static HrirTable,
    /// Historique du son, écrit deux fois pour toujours lire une tranche contiguë.
    history: Vec<f32>,
    pos: usize,
    /// Réponses courantes, à l'envers (pour un produit scalaire dans l'ordre chronologique).
    coeffs: [Vec<f32>; 2],
}

impl HrtfModel {
    fn new(sample_rate: u32) -> Self {
        let table = table_for(sample_rate);
        Self { table, history: vec![0.0; table.taps * 2], pos: 0, coeffs: [vec![0.0; table.taps], vec![0.0; table.taps]] }
    }

    fn latency(&self) -> f32 {
        self.table.latency as f32
    }

    fn set_direction(&mut self, azimuth: f32) {
        let deg = azimuth.to_degrees().rem_euclid(360.0);
        let pos = deg / HRTF_STEP_DEG;
        let i0 = (pos.floor() as usize) % HRTF_DIRECTIONS;
        let i1 = (i0 + 1) % HRTF_DIRECTIONS;
        let frac = pos - pos.floor();
        let taps = self.table.taps;
        for ear in 0..2 {
            let a = self.table.ir(i0, ear);
            let b = self.table.ir(i1, ear);
            let c = &mut self.coeffs[ear];
            for k in 0..taps {
                c[taps - 1 - k] = a[k] + (b[k] - a[k]) * frac;
            }
        }
    }

    fn process(&mut self, mono: f32) -> (f32, f32) {
        let taps = self.table.taps;
        self.history[self.pos] = mono;
        self.history[self.pos + taps] = mono;
        self.pos = (self.pos + 1) % taps;
        // Tranche des `taps` derniers échantillons, du plus ancien au plus récent.
        let window = &self.history[self.pos..self.pos + taps];
        (dot(window, &self.coeffs[0]), dot(window, &self.coeffs[1]))
    }

    fn reset(&mut self) {
        self.history.iter_mut().for_each(|x| *x = 0.0);
    }
}

/// Filtre en peigne amorti (Freeverb).
struct Comb {
    buf: Vec<f32>,
    len: usize,
    idx: usize,
    store: f32,
}

impl Comb {
    fn process(&mut self, input: f32, feedback: f32, damp: f32) -> f32 {
        let out = self.buf[self.idx];
        self.store = out * (1.0 - damp) + self.store * damp;
        if self.store.abs() < 1e-25 {
            self.store = 0.0;
        }
        self.buf[self.idx] = input + self.store * feedback;
        self.idx += 1;
        if self.idx >= self.len {
            self.idx = 0;
        }
        out
    }
}

/// Filtre passe-tout (Freeverb).
struct Allpass {
    buf: Vec<f32>,
    idx: usize,
}

impl Allpass {
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buf[self.idx];
        let out = delayed - input;
        let mut stored = input + delayed * 0.5;
        if stored.abs() < 1e-25 {
            stored = 0.0;
        }
        self.buf[self.idx] = stored;
        self.idx = (self.idx + 1) % self.buf.len();
        out
    }
}

const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD: usize = 23;
/// Échelle des longueurs de peigne : de 0,6 (petite pièce) à 1,6 (très grand volume).
const MAX_ROOM_SCALE: f32 = 1.6;

fn room_scale(size: f32) -> f32 {
    0.6 + size.clamp(0.0, 1.0)
}

/// Réverbération stéréo (Freeverb de Jezar, domaine public), longueurs ajustées à la taille.
struct Room {
    combs: [Vec<Comb>; 2],
    allpasses: [Vec<Allpass>; 2],
    predelay: DelayLine,
    rate_scale: f32,
    feedback: f32,
    damp: f32,
    predelay_samples: f32,
    lite: bool,
    scale: f32,
    sample_rate: f32,
}

impl Room {
    fn new(sample_rate: f32) -> Self {
        let rate_scale = sample_rate / 44_100.0;
        let make_combs = |spread: usize| {
            COMB_TUNING
                .iter()
                .map(|&t| {
                    let max = (((t + spread) as f32) * rate_scale * MAX_ROOM_SCALE).ceil() as usize + 1;
                    Comb { buf: vec![0.0; max], len: max, idx: 0, store: 0.0 }
                })
                .collect::<Vec<_>>()
        };
        let make_allpasses = |spread: usize| {
            ALLPASS_TUNING
                .iter()
                .map(|&t| Allpass { buf: vec![0.0; ((((t + spread) as f32) * rate_scale).ceil() as usize).max(1)], idx: 0 })
                .collect::<Vec<_>>()
        };
        let mut room = Self {
            combs: [make_combs(0), make_combs(STEREO_SPREAD)],
            allpasses: [make_allpasses(0), make_allpasses(STEREO_SPREAD)],
            predelay: DelayLine::new((0.125 * sample_rate) as usize + 4),
            rate_scale,
            feedback: 0.84,
            damp: 0.2,
            predelay_samples: 0.0,
            lite: false,
            scale: 0.0,
            sample_rate,
        };
        room.configure(&ImmersiveSettings::default(), false);
        room
    }

    fn configure(&mut self, s: &ImmersiveSettings, lite: bool) {
        self.feedback = s.room_size * 0.26 + 0.7;
        self.damp = s.room_damping * 0.4;
        self.predelay_samples = s.room_predelay_ms / 1000.0 * self.sample_rate;
        self.lite = lite;
        let scale = room_scale(s.room_size);
        if (scale - self.scale).abs() > 1e-4 {
            self.scale = scale;
            for (ch, spread) in [(0, 0), (1, STEREO_SPREAD)] {
                for (comb, &t) in self.combs[ch].iter_mut().zip(COMB_TUNING.iter()) {
                    comb.len = ((((t + spread) as f32) * self.rate_scale * scale).round() as usize).clamp(1, comb.buf.len());
                    if comb.idx >= comb.len {
                        comb.idx = 0;
                    }
                }
            }
        }
    }

    fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        // Entrée commune (Freeverb), atténuée : les peignes s'additionnent.
        self.predelay.push((l + r) * 0.015);
        let input = self.predelay.read(self.predelay_samples);
        let step = if self.lite { 2 } else { 1 };
        let gain = if self.lite { 2.0 } else { 1.0 };
        let mut out = [0.0f32; 2];
        for (ch, o) in out.iter_mut().enumerate() {
            let mut acc = 0.0;
            for comb in self.combs[ch].iter_mut().step_by(step) {
                acc += comb.process(input, self.feedback, self.damp);
            }
            acc *= gain;
            for ap in self.allpasses[ch].iter_mut().step_by(step) {
                acc = ap.process(acc);
            }
            *o = acc;
        }
        (out[0], out[1])
    }

    fn reset(&mut self) {
        self.predelay.clear();
        for ch in 0..2 {
            for c in &mut self.combs[ch] {
                c.buf.iter_mut().for_each(|x| *x = 0.0);
                c.store = 0.0;
            }
            for a in &mut self.allpasses[ch] {
                a.buf.iter_mut().for_each(|x| *x = 0.0);
            }
        }
    }
}

// ---------------------------------------------------------------------
// Chaîne complète
// ---------------------------------------------------------------------

/// Amortissement des rampes par bloc (environ 60 ms pour passer d'un réglage à l'autre).
const RAMP: f32 = 0.12;
/// Gain de la réverbération (Freeverb : 3) ; le son direct baisse un peu quand elle monte.
const WET_GAIN: f32 = 3.0;
/// Le son ramené au centre puis spatialisé perd environ 2 dB : compensé.
const ROTATION_GAIN: f32 = 1.25;

struct Processor {
    sample_rate: u32,
    params: Params,
    seen_version: u64,
    step: usize,
    azimuth: f32,
    head: HeadModel,
    hrtf: Option<HrtfModel>,
    /// Moteur réellement utilisé : un changement attend que l'effet soit redescendu à zéro.
    active_engine: Engine3d,
    dry_delay: [DelayLine; 2],
    rotation_amount: f32,
    room: Option<Room>,
    room_amount: f32,
}

impl Processor {
    fn new(sample_rate: u32) -> Self {
        let params = current_params();
        let mut p = Self {
            sample_rate,
            seen_version: VERSION.load(Ordering::SeqCst),
            step: 0,
            azimuth: 0.0,
            head: HeadModel::new(sample_rate as f32),
            hrtf: None,
            active_engine: params.settings.engine,
            dry_delay: [DelayLine::new(1100), DelayLine::new(1100)],
            rotation_amount: 0.0,
            room: None,
            room_amount: 0.0,
            params,
        };
        p.ensure_engine();
        p.apply_params();
        // Effet déjà actif au lancement d'un morceau : directement à son niveau.
        p.rotation_amount = p.rotation_target();
        p.room_amount = p.room_target();
        p
    }

    fn ensure_engine(&mut self) {
        if self.active_engine == Engine3d::Hrtf && self.hrtf.is_none() {
            self.hrtf = Some(HrtfModel::new(self.sample_rate));
        }
    }

    fn apply_params(&mut self) {
        if self.params.room_on() || self.room.is_some() {
            let lite = self.params.lite;
            let settings = self.params.settings.clone();
            self.room.get_or_insert_with(|| Room::new(self.sample_rate as f32)).configure(&settings, lite);
        }
    }

    fn rotation_target(&self) -> f32 {
        if !self.params.rotation_on() || self.params.settings.engine != self.active_engine {
            0.0
        } else {
            self.params.settings.intensity
        }
    }

    fn room_target(&self) -> f32 {
        if self.params.room_on() {
            self.params.settings.room_mix
        } else {
            0.0
        }
    }

    fn latency(&self) -> f32 {
        match (self.active_engine, &self.hrtf) {
            (Engine3d::Hrtf, Some(h)) => h.latency(),
            _ => self.head.latency(),
        }
    }

    fn update_block(&mut self) {
        let version = VERSION.load(Ordering::Relaxed);
        if version != self.seen_version {
            if let Ok(guard) = PARAMS.try_lock() {
                if let Some(p) = guard.as_ref() {
                    self.params = p.clone();
                }
                self.seen_version = version;
                drop(guard);
                self.apply_params();
            }
        }
        // Changement de moteur : une fois l'effet redescendu, on bascule.
        if self.params.settings.engine != self.active_engine && self.rotation_amount < 0.002 {
            self.active_engine = self.params.settings.engine;
            self.ensure_engine();
            self.head.reset();
            if let Some(h) = &mut self.hrtf {
                h.reset();
            }
        }
        self.rotation_amount += (self.rotation_target() - self.rotation_amount) * RAMP;
        if self.rotation_amount < 0.001 && self.rotation_target() == 0.0 {
            self.rotation_amount = 0.0;
        }
        self.room_amount += (self.room_target() - self.room_amount) * RAMP;
        if self.room_amount < 0.001 && self.room_target() == 0.0 {
            self.room_amount = 0.0;
        }

        if self.rotation_amount > 0.0 {
            let s = &self.params.settings;
            let turns_per_sec = s.speed / 60.0;
            let dir = if s.clockwise { -1.0 } else { 1.0 };
            self.azimuth = (self.azimuth + dir * TAU * turns_per_sec * BLOCK as f32 / self.sample_rate as f32).rem_euclid(TAU);
            match (self.active_engine, &mut self.hrtf) {
                (Engine3d::Hrtf, Some(h)) => h.set_direction(self.azimuth),
                _ => self.head.set_direction(self.azimuth),
            }
        }
    }

    fn frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        if self.step == 0 {
            self.update_block();
        }
        let step = self.step;
        self.step = (self.step + 1) % BLOCK;

        let (mut out_l, mut out_r) = (l, r);
        if self.rotation_amount > 0.0 {
            let mono = 0.5 * (l + r);
            let (el, er) = match (self.active_engine, &mut self.hrtf) {
                (Engine3d::Hrtf, Some(h)) => h.process(mono),
                _ => self.head.process(mono, step),
            };
            // Son d'origine retardé comme le son spatialisé, puis mélange selon le dosage.
            self.dry_delay[0].push(l);
            self.dry_delay[1].push(r);
            let latency = self.latency();
            let dl = self.dry_delay[0].read(latency);
            let dr = self.dry_delay[1].read(latency);
            let a = self.rotation_amount;
            out_l = dl + (el * ROTATION_GAIN - dl) * a;
            out_r = dr + (er * ROTATION_GAIN - dr) * a;
        } else {
            self.dry_delay[0].push(l);
            self.dry_delay[1].push(r);
        }

        if self.room_amount > 0.0 {
            if let Some(room) = &mut self.room {
                let (wl, wr) = room.process(out_l, out_r);
                let a = self.room_amount;
                let dry = 1.0 - 0.35 * a;
                out_l = out_l * dry + wl * WET_GAIN * a;
                out_r = out_r * dry + wr * WET_GAIN * a;
            }
        }
        (out_l, out_r)
    }

    fn reset(&mut self) {
        self.head.reset();
        if let Some(h) = &mut self.hrtf {
            h.reset();
        }
        if let Some(room) = &mut self.room {
            room.reset();
        }
        self.dry_delay.iter_mut().for_each(DelayLine::clear);
    }
}

/// Source rodio qui applique le mode immersif. Sortie toujours stéréo (mono dédoublé),
/// sauf pour un flux à plus de deux canaux, transmis tel quel.
pub struct ImmersiveSource<I> {
    input: I,
    in_channels: u16,
    sample_rate: SampleRate,
    passthrough: bool,
    processor: Processor,
    pending: Option<f32>,
}

impl<I> ImmersiveSource<I>
where
    I: Source<Item = f32>,
{
    pub fn new(input: I) -> Self {
        let in_channels = input.channels().get();
        let sample_rate = input.sample_rate();
        Self {
            passthrough: in_channels > 2,
            processor: Processor::new(sample_rate.get()),
            input,
            in_channels,
            sample_rate,
            pending: None,
        }
    }
}

impl<I> Iterator for ImmersiveSource<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.passthrough {
            return self.input.next();
        }
        if let Some(right) = self.pending.take() {
            return Some(right);
        }
        let l = self.input.next()?;
        let r = if self.in_channels == 2 { self.input.next()? } else { l };
        let (out_l, out_r) = self.processor.frame(l, r);
        self.pending = Some(out_r);
        Some(out_l)
    }
}

impl<I> Source for ImmersiveSource<I>
where
    I: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        let len = self.input.current_span_len()?;
        Some(if self.in_channels == 1 { len * 2 } else { len } + usize::from(self.pending.is_some()))
    }
    fn channels(&self) -> ChannelCount {
        if self.passthrough {
            self.input.channels()
        } else {
            ChannelCount::MIN.saturating_add(1)
        }
    }
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.input.try_seek(pos)?;
        self.pending = None;
        self.processor.reset();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les tests partagent l'emplacement global des réglages : un seul à la fois.
    static SERIAL: Mutex<()> = Mutex::new(());

    struct TestSource {
        samples: std::vec::IntoIter<f32>,
        sample_rate: u32,
        channels: u16,
    }

    impl TestSource {
        fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
            Self { samples: samples.into_iter(), sample_rate, channels }
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

    /// Bruit pseudo-aléatoire reproductible.
    fn noise(len: usize, seed: u32) -> Vec<f32> {
        let mut x = seed;
        (0..len)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x as f32 / u32::MAX as f32) * 0.5 - 0.25
            })
            .collect()
    }

    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len().max(1) as f32).sqrt()
    }

    fn stereo(mono: &[f32]) -> Vec<f32> {
        mono.iter().flat_map(|&x| [x, x]).collect()
    }

    fn run(settings: ImmersiveSettings, input: Vec<f32>, rate: u32, channels: u16) -> Vec<f32> {
        set_bypass(false);
        set_lite(false);
        publish(&settings);
        ImmersiveSource::new(TestSource::new(input, rate, channels)).collect()
    }

    fn split(v: &[f32]) -> (Vec<f32>, Vec<f32>) {
        (v.iter().step_by(2).copied().collect(), v.iter().skip(1).step_by(2).copied().collect())
    }

    #[test]
    fn disabled_effect_leaves_stereo_unchanged() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let input: Vec<f32> = noise(4000, 7);
        let out = run(ImmersiveSettings::default(), input.clone(), 44_100, 2);
        assert_eq!(out, input);
    }

    #[test]
    fn mono_is_duplicated_to_stereo() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let input = noise(1000, 3);
        let out = run(ImmersiveSettings::default(), input.clone(), 48_000, 1);
        assert_eq!(out, stereo(&input));
        let src = ImmersiveSource::new(TestSource::new(input, 48_000, 1));
        assert_eq!(src.channels().get(), 2);
    }

    #[test]
    fn more_than_two_channels_pass_through() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let input = noise(600, 5);
        let settings = ImmersiveSettings { enabled: true, room: true, ..Default::default() };
        let out = run(settings, input.clone(), 48_000, 6);
        assert_eq!(out, input);
    }

    /// Énergie de l'oreille gauche et de la droite quand le son est fixé à `azimuth`.
    fn ears_at(engine: Engine3d, azimuth: f32, rate: u32) -> (f32, f32) {
        let mut p = Processor::new(rate);
        p.params = Params {
            settings: ImmersiveSettings { enabled: true, rotation: true, engine, intensity: 1.0, ..Default::default() },
            bypass: false,
            lite: false,
        };
        p.active_engine = engine;
        p.ensure_engine();
        p.rotation_amount = 1.0;
        let input = noise(rate as usize / 2, 11);
        let mut l = Vec::new();
        let mut r = Vec::new();
        for (i, &x) in input.iter().enumerate() {
            if i % BLOCK == 0 {
                match (p.active_engine, &mut p.hrtf) {
                    (Engine3d::Hrtf, Some(h)) => h.set_direction(azimuth),
                    _ => p.head.set_direction(azimuth),
                }
            }
            let (a, b) = match (p.active_engine, &mut p.hrtf) {
                (Engine3d::Hrtf, Some(h)) => h.process(x),
                _ => p.head.process(x, i % BLOCK),
            };
            l.push(a);
            r.push(b);
        }
        (rms(&l[2000..]), rms(&r[2000..]))
    }

    #[test]
    fn a_sound_on_the_left_is_louder_in_the_left_ear_for_both_engines() {
        for engine in [Engine3d::Head, Engine3d::Hrtf] {
            for rate in [44_100, 48_000, 96_000] {
                let (l, r) = ears_at(engine, FRAC_PI_2, rate);
                assert!(l > r * 1.5, "{engine:?} {rate} : gauche {l} droite {r}");
                let (l, r) = ears_at(engine, -FRAC_PI_2, rate);
                assert!(r > l * 1.5, "{engine:?} {rate} : gauche {l} droite {r}");
            }
        }
    }

    #[test]
    fn spatialized_level_stays_close_to_the_original() {
        for engine in [Engine3d::Head, Engine3d::Hrtf] {
            let input_rms = rms(&noise(24_000, 11)[2000..]);
            for az in [0.0f32, 1.0, 2.0, 3.0] {
                let (l, r) = ears_at(engine, az, 48_000);
                let level = ((l * l + r * r) / 2.0).sqrt() / input_rms;
                assert!((0.4..2.2).contains(&level), "{engine:?} az {az} : niveau relatif {level}");
            }
        }
    }

    #[test]
    fn rotation_moves_the_sound_between_the_ears_and_never_explodes() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        for engine in [Engine3d::Head, Engine3d::Hrtf] {
            let settings = ImmersiveSettings { enabled: true, rotation: true, engine, speed: 30.0, intensity: 1.0, ..Default::default() };
            // 2 s à 30 tours/min : un tour complet.
            let out = run(settings, stereo(&noise(88_200, 9)), 44_100, 2);
            assert!(out.iter().all(|x| x.is_finite() && x.abs() < 4.0));
            let (l, r) = split(&out);
            // Balance gauche/droite par quart de seconde : elle doit franchement varier.
            let balances: Vec<f32> = l
                .chunks(11_025)
                .zip(r.chunks(11_025))
                .map(|(a, b)| (rms(a) - rms(b)) / (rms(a) + rms(b) + 1e-9))
                .collect();
            let max = balances.iter().cloned().fold(f32::MIN, f32::max);
            let min = balances.iter().cloned().fold(f32::MAX, f32::min);
            assert!(max > 0.2 && min < -0.2, "{engine:?} : balances {balances:?}");
        }
    }

    #[test]
    fn room_adds_a_tail_after_the_sound_stops() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        for lite in [false, true] {
            let mut input = stereo(&noise(22_050, 4));
            input.extend(std::iter::repeat_n(0.0, 44_100));
            let settings = ImmersiveSettings { enabled: true, rotation: false, room: true, room_mix: 0.5, ..Default::default() };
            set_bypass(false);
            set_lite(lite);
            publish(&settings);
            let out: Vec<f32> = ImmersiveSource::new(TestSource::new(input, 44_100, 2)).collect();
            assert!(out.iter().all(|x| x.is_finite() && x.abs() < 4.0));
            // 0,1 s après la fin du son : la salle résonne encore.
            let tail = &out[2 * (22_050 + 4_410)..2 * (22_050 + 8_820)];
            assert!(rms(tail) > 0.005, "léger {lite} : queue {}", rms(tail));
        }
        set_lite(false);
    }

    #[test]
    fn bypass_returns_to_the_original_sound_without_a_jump() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let settings = ImmersiveSettings { enabled: true, rotation: true, room: true, ..Default::default() };
        set_lite(false);
        set_bypass(false);
        publish(&settings);
        let input = stereo(&noise(48_000, 21));
        let mut src = ImmersiveSource::new(TestSource::new(input.clone(), 48_000, 2));
        let mut out: Vec<f32> = (&mut src).take(48_000).collect();
        set_bypass(true);
        out.extend(src);
        set_bypass(false);
        // Pas de saut brutal au moment du basculement.
        let jumps = out.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
        assert!(jumps < 1.5, "saut {jumps}");
        // Une fois la rampe et la queue de salle passées, le son d'origine revient.
        let end = out.len() - 2000;
        let diff: f32 = out[end..].iter().zip(&input[end..]).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
        assert!(diff < 0.02, "écart {diff}");
    }

    #[test]
    fn playlist_settings_win_over_global_ones_only_in_their_context() {
        let mut profiles = ImmersiveProfiles::default();
        let soiree = ImmersiveSettings { enabled: true, room: true, room_preset: "club".into(), ..Default::default() };
        profiles.playlists.insert("p1".into(), soiree.clone());
        assert_eq!(profiles.effective(), &profiles.global);
        profiles.context = Some("p1".into());
        assert_eq!(profiles.effective(), &soiree);
        profiles.context = Some("p2".into());
        assert_eq!(profiles.effective(), &profiles.global);
    }

    #[test]
    fn out_of_range_settings_are_clamped() {
        let s = ImmersiveSettings { speed: 1000.0, intensity: f32::NAN, room_mix: -3.0, room_predelay_ms: 9999.0, ..Default::default() }.sanitized();
        assert_eq!(s.speed, 40.0);
        assert_eq!(s.intensity, ImmersiveSettings::default().intensity);
        assert_eq!(s.room_mix, 0.0);
        assert_eq!(s.room_predelay_ms, 120.0);
    }

    #[test]
    fn hrir_tables_are_complete_and_resampled_for_other_rates() {
        for rate in [44_100, 48_000, 96_000, 22_050] {
            let t = table_for(rate);
            assert_eq!(t.data.len(), HRTF_DIRECTIONS * 2 * t.taps);
            assert!(t.data.iter().all(|x| x.is_finite()));
            assert!(t.latency < t.taps);
        }
        assert_eq!(table_for(96_000).taps, 512);
    }

    /// Coût et niveaux mesurés (`cargo test --release --lib -- --ignored --nocapture cout_du_mode_immersif`).
    #[test]
    #[ignore]
    fn cout_du_mode_immersif() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let secs = 30;
        let input = stereo(&noise(48_000 * secs, 2));
        for (name, s, lite) in [
            ("tête", ImmersiveSettings { enabled: true, engine: Engine3d::Head, ..Default::default() }, false),
            ("HRTF", ImmersiveSettings { enabled: true, engine: Engine3d::Hrtf, ..Default::default() }, false),
            ("HRTF + salle", ImmersiveSettings { enabled: true, engine: Engine3d::Hrtf, room: true, ..Default::default() }, false),
            ("HRTF + salle légère", ImmersiveSettings { enabled: true, engine: Engine3d::Hrtf, room: true, ..Default::default() }, true),
        ] {
            set_bypass(false);
            set_lite(lite);
            publish(&s);
            let t = std::time::Instant::now();
            let out: Vec<f32> = ImmersiveSource::new(TestSource::new(input.clone(), 48_000, 2)).collect();
            let el = t.elapsed();
            println!("{name:>20} : {secs} s traitées en {:?} ({:.2} % d'un cœur), niveau {:.2}", el, el.as_secs_f64() / secs as f64 * 100.0, rms(&out) / rms(&input));
        }
        set_lite(false);
    }
}
