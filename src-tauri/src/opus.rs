//! Fichiers Opus (`.opus`, et `.ogg` qui contiennent de l'Opus : vocaux WhatsApp, enregistreurs,
//! vidéos converties) : le décodeur intégré ne les lit pas, et ffmpeg n'existe pas sur Android.
//! Décodés ici en WAV, en Rust pur, sur toutes les plateformes. Même chose pour l'Opus rangé
//! dans un conteneur MP4 (`.m4a` de certains enregistreurs, comme Audiolab) : symphonia
//! sépare les paquets, ce module les décode.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use ogg::PacketReader;
use opus_decoder::OpusDecoder;

/// Opus travaille toujours à 48 kHz (RFC 7845 : les positions Ogg aussi).
const RATE: u32 = 48_000;

struct Head {
    channels: usize,
    pre_skip: usize,
}

/// En-tête « OpusHead » du premier paquet. `None` : ce n'est pas de l'Opus ; les fichiers de plus
/// de deux voies (famille de canaux autre que 0) sont laissés à ffmpeg.
fn parse_head(packet: &[u8]) -> Option<Head> {
    if packet.len() < 19 || &packet[..8] != b"OpusHead" {
        return None;
    }
    let channels = usize::from(packet[9]);
    if !(1..=2).contains(&channels) || packet[18] != 0 {
        return None;
    }
    Some(Head { channels, pre_skip: usize::from(u16::from_le_bytes([packet[10], packet[11]])) })
}

/// Ce fichier est-il de l'Opus dans un conteneur Ogg ? (Lit seulement le début.)
pub fn is_ogg_opus(path: &Path) -> bool {
    let mut magic = [0u8; 4];
    let Ok(mut file) = File::open(path) else { return false };
    if file.read_exact(&mut magic).is_err() || &magic != b"OggS" {
        return false;
    }
    drop(file);
    let Ok(file) = File::open(path) else { return false };
    let mut packets = PacketReader::new(BufReader::new(file));
    matches!(packets.read_packet(), Ok(Some(p)) if parse_head(&p.data).is_some())
}

/// Opus dans un conteneur MP4 (`.m4a`, `.mp4`) ? Le lecteur intégré ne le décode pas.
pub fn is_mp4_opus(path: &Path) -> bool {
    let mut header = [0u8; 8];
    let Ok(mut file) = File::open(path) else { return false };
    if file.read_exact(&mut header).is_err() || &header[4..8] != b"ftyp" {
        return false;
    }
    drop(file);
    mp4_opus_track(path).is_some()
}

/// Opus, quel que soit le conteneur (Ogg ou MP4).
pub fn is_opus(path: &Path) -> bool {
    is_ogg_opus(path) || is_mp4_opus(path)
}

/// Piste Opus d'un MP4 : le lecteur du conteneur, son numéro et son en-tête.
fn mp4_opus_track(path: &Path) -> Option<(Box<dyn symphonia::core::formats::FormatReader>, u32, Head)> {
    use symphonia::core::codecs::CODEC_TYPE_OPUS;
    let format = crate::library::open_format(path)?;
    let track = format.tracks().iter().find(|t| t.codec_params.codec == CODEC_TYPE_OPUS)?;
    let head = parse_mp4_head(track.codec_params.extra_data.as_deref()?)?;
    let id = track.id;
    Some((format, id, head))
}

/// En-tête Opus d'un MP4 (boîte « dOps », précédée de « OpusHead » par symphonia) : mêmes
/// champs que dans l'Ogg, mais écrits en gros-boutiste (norme ISO de l'Opus dans le MP4).
fn parse_mp4_head(extra: &[u8]) -> Option<Head> {
    if extra.len() < 19 || &extra[..8] != b"OpusHead" {
        return None;
    }
    let channels = usize::from(extra[9]);
    if !(1..=2).contains(&channels) || extra[18] != 0 {
        return None;
    }
    Some(Head { channels, pre_skip: usize::from(u16::from_be_bytes([extra[10], extra[11]])) })
}

/// Paquets décodés les uns après les autres, puis rassemblés en WAV.
struct Decoding {
    decoder: OpusDecoder,
    frame: Vec<i16>,
    pcm: Vec<i16>,
    channels: usize,
}

impl Decoding {
    fn new(channels: usize) -> Result<Self, String> {
        let decoder = OpusDecoder::new(RATE, channels).map_err(|e| format!("Opus : {e:?}"))?;
        let frame = vec![0i16; decoder.max_frame_size_per_channel() * channels];
        Ok(Self { decoder, frame, pcm: Vec::new(), channels })
    }

    fn push(&mut self, data: &[u8]) {
        // Paquet abîmé au milieu : un trou plutôt qu'un fichier entier refusé.
        if let Ok(samples) = self.decoder.decode(data, &mut self.frame, false) {
            self.pcm.extend_from_slice(&self.frame[..samples * self.channels]);
        }
    }

    /// `end` : dernière position utile (Ogg), sinon tout ce qui a été décodé.
    fn finish(self, pre_skip: usize, end: Option<usize>) -> Result<Vec<u8>, String> {
        if self.pcm.is_empty() {
            return Err("Ce fichier Opus ne contient aucun son lisible.".into());
        }
        // Le début (pré-roulement) et la fin (remplissage) ne font pas partie du son.
        let per_channel = self.pcm.len() / self.channels;
        let end = end.map_or(per_channel, |e| e.min(per_channel));
        let start = pre_skip.min(end);
        Ok(wav_bytes(&self.pcm[start * self.channels..end * self.channels], self.channels))
    }
}

/// Décode tout le fichier en WAV 16 bits (en mémoire), comme le fait `ffmpeg::decode_to_wav`.
pub fn decode_to_wav(path: &Path) -> Result<Vec<u8>, String> {
    if !is_ogg_opus(path) && is_mp4_opus(path) {
        return decode_mp4_to_wav(path);
    }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut packets = PacketReader::new(BufReader::new(file));
    let head = packets
        .read_packet()
        .ok()
        .flatten()
        .and_then(|p| parse_head(&p.data))
        .ok_or("Ce fichier n'est pas un fichier Opus lisible.")?;
    // Deuxième paquet : les étiquettes.
    packets.read_packet().map_err(|_| "Fichier Opus incomplet.".to_string())?;

    let mut decoding = Decoding::new(head.channels)?;
    let mut last_position = 0u64;
    // Fin de fichier abîmée (enregistrement coupé) : on garde ce qui précède.
    while let Ok(Some(packet)) = packets.read_packet() {
        if packet.last_in_page() {
            last_position = packet.absgp_page();
        }
        decoding.push(&packet.data);
    }
    decoding.finish(head.pre_skip, (last_position > 0).then_some(last_position as usize))
}

fn decode_mp4_to_wav(path: &Path) -> Result<Vec<u8>, String> {
    let (mut format, id, head) = mp4_opus_track(path).ok_or("Ce fichier n'est pas un fichier Opus lisible.")?;
    let mut decoding = Decoding::new(head.channels)?;
    // Fin abîmée : on garde ce qui précède.
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() == id {
            decoding.push(&packet.data);
        }
    }
    decoding.finish(head.pre_skip, None)
}

fn wav_bytes(samples: &[i16], channels: usize) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let block = (channels * 2) as u16;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(channels as u16).to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * u32::from(block)).to_le_bytes());
    out.extend_from_slice(&block.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
    }

    fn samples(wav: &[u8]) -> Vec<f64> {
        wav[44..].as_chunks::<2>().0.iter().map(|b| f64::from(i16::from_le_bytes(*b))).collect()
    }

    /// Énergie du signal à une fréquence (Goertzel), rapportée à l'énergie totale.
    fn share_at(signal: &[f64], freq: f64) -> f64 {
        let w = 2.0 * std::f64::consts::PI * freq / f64::from(RATE);
        let (mut a, mut b) = (0.0, 0.0);
        for s in signal {
            let c = s + 2.0 * w.cos() * a - b;
            b = a;
            a = c;
        }
        let power = a * a + b * b - 2.0 * w.cos() * a * b;
        let total: f64 = signal.iter().map(|s| s * s).sum();
        power / (total * signal.len() as f64 / 2.0)
    }

    #[test]
    fn decodes_music_and_voice_with_the_right_tone_and_length() {
        // Fichiers créés avec ffmpeg/libopus : musique (CELT), voix (SILK), Opus en « .ogg ».
        for (name, channels, freq) in [("ton-stereo.opus", 2u16, 440.0), ("voix-mono.opus", 1, 300.0), ("opus-dans-ogg.ogg", 1, 500.0), ("opus-dans-m4a.m4a", 1, 600.0)] {
            let path = fixture(name);
            assert!(is_opus(&path), "{name}");
            let wav = decode_to_wav(&path).unwrap_or_else(|e| panic!("{name} : {e}"));
            assert_eq!(&wav[..4], b"RIFF");
            assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), channels, "{name}");
            let mono: Vec<f64> = samples(&wav).iter().step_by(usize::from(channels)).copied().collect();
            let secs = mono.len() as f64 / f64::from(RATE);
            assert!((secs - 1.0).abs() < 0.02, "{name} : {secs} s");
            let middle = &mono[mono.len() / 4..mono.len() * 3 / 4];
            assert!(share_at(middle, freq) > 0.8, "{name} : pas le bon son ({})", share_at(middle, freq));
        }
    }

    #[test]
    fn the_player_reads_opus_without_ffmpeg() {
        let wav = decode_to_wav(&fixture("voix-mono.opus")).unwrap();
        let decoder = rodio::Decoder::new(std::io::Cursor::new(wav)).unwrap();
        assert_eq!(rodio::Source::channels(&decoder).get(), 1);
    }

    #[test]
    fn a_cut_recording_keeps_what_was_recorded() {
        let bytes = std::fs::read(fixture("ton-stereo.opus")).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let cut = dir.path().join("coupe.opus");
        std::fs::write(&cut, &bytes[..bytes.len() / 2]).unwrap();
        let wav = decode_to_wav(&cut).unwrap();
        assert!(wav.len() > 44 + 48_000);
    }

    #[test]
    fn other_files_are_refused_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("faux.opus");
        std::fs::write(&path, b"pas de l'opus du tout").unwrap();
        assert!(!is_ogg_opus(&path));
        assert!(decode_to_wav(&path).is_err());
        assert!(!is_ogg_opus(&dir.path().join("absent.opus")));
        assert!(!is_opus(&path));
    }

    #[test]
    fn opus_in_mp4_is_recognised_but_not_other_mp4() {
        // Enregistreur qui range de l'Opus dans un « .m4a » : lu sans ffmpeg (Android).
        assert!(is_mp4_opus(&fixture("opus-dans-m4a.m4a")));
        assert!(!is_ogg_opus(&fixture("opus-dans-m4a.m4a")));
        assert!(!is_mp4_opus(&fixture("voix-mono.opus")));
    }
}
