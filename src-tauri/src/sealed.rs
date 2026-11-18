//! Flux chiffrés pour le partage à proximité : chaque trame (64 Kio au plus) est chiffrée et
//! authentifiée (ChaCha20-Poly1305, compteur de trames comme nonce). Une trame modifiée ou
//! rejouée est refusée. Les clés viennent d'un échange SPAKE2 au début de la connexion (voir
//! nearby.rs), comme pour le partage par code (remote.rs).

use std::io::{self, Read, Write};

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;

/// Texte clair au plus par trame.
const FRAME: usize = 64 * 1024;
/// Étiquette d'authentification ajoutée à chaque trame.
const TAG: usize = 16;

fn nonce(counter: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[4..].copy_from_slice(&counter.to_be_bytes());
    n
}

/// Clés d'émission et de réception tirées du secret partagé (une par sens).
pub fn ciphers(secret: &[u8], sender: bool) -> Result<(ChaCha20Poly1305, ChaCha20Poly1305), String> {
    let hkdf = Hkdf::<Sha256>::new(None, secret);
    let (mut forward, mut backward) = ([0u8; 32], [0u8; 32]);
    hkdf.expand(b"proximite envoi vers reception", &mut forward).map_err(|e| e.to_string())?;
    hkdf.expand(b"proximite reception vers envoi", &mut backward).map_err(|e| e.to_string())?;
    let (seal, open) = if sender { (forward, backward) } else { (backward, forward) };
    Ok((
        ChaCha20Poly1305::new_from_slice(&seal).map_err(|e| e.to_string())?,
        ChaCha20Poly1305::new_from_slice(&open).map_err(|e| e.to_string())?,
    ))
}

/// Écrit en trames chiffrées.
pub struct SealWriter<W: Write> {
    inner: W,
    cipher: ChaCha20Poly1305,
    counter: u64,
}

impl<W: Write> SealWriter<W> {
    pub fn new(inner: W, cipher: ChaCha20Poly1305) -> Self {
        Self { inner, cipher, counter: 0 }
    }
}

impl<W: Write> Write for SealWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let n = buf.len().min(FRAME);
        let sealed = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce(self.counter)), &buf[..n])
            .map_err(|_| io::Error::other("Chiffrement impossible."))?;
        self.counter += 1;
        let len = u32::try_from(sealed.len()).map_err(|_| io::Error::other("Trame trop longue."))?;
        self.inner.write_all(&len.to_be_bytes())?;
        self.inner.write_all(&sealed)?;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Lit des trames chiffrées ; refuse toute trame altérée, trop longue ou hors d'ordre.
pub struct OpenReader<R: Read> {
    inner: R,
    cipher: ChaCha20Poly1305,
    counter: u64,
    plain: Vec<u8>,
    at: usize,
}

impl<R: Read> OpenReader<R> {
    pub fn new(inner: R, cipher: ChaCha20Poly1305) -> Self {
        Self { inner, cipher, counter: 0, plain: Vec::new(), at: 0 }
    }
}

impl<R: Read> Read for OpenReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.at >= self.plain.len() {
            let mut len = [0u8; 4];
            // Fin propre seulement entre deux trames.
            let first = self.inner.read(&mut len[..1])?;
            if first == 0 {
                return Ok(0);
            }
            self.inner.read_exact(&mut len[1..])?;
            let len = u32::from_be_bytes(len) as usize;
            if !(TAG..=FRAME + TAG).contains(&len) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Trame invalide."));
            }
            let mut sealed = vec![0u8; len];
            self.inner.read_exact(&mut sealed)?;
            self.plain = self
                .cipher
                .decrypt(Nonce::from_slice(&nonce(self.counter)), sealed.as_slice())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Données altérées ou mauvaise clé."))?;
            self.counter += 1;
            self.at = 0;
        }
        let n = out.len().min(self.plain.len() - self.at);
        out[..n].copy_from_slice(&self.plain[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> ((ChaCha20Poly1305, ChaCha20Poly1305), (ChaCha20Poly1305, ChaCha20Poly1305)) {
        (ciphers(b"secret commun", true).unwrap(), ciphers(b"secret commun", false).unwrap())
    }

    #[test]
    fn what_is_sealed_is_opened_and_never_readable_on_the_wire() {
        let ((seal, _), (_, open)) = pair();
        let message: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        let mut wire = Vec::new();
        SealWriter::new(&mut wire, seal).write_all(&message).unwrap();
        assert!(wire.len() > message.len(), "étiquettes et longueurs ajoutées");
        assert!(!wire.windows(64).any(|w| w == &message[1000..1064]), "le texte clair n'apparaît pas");
        let mut back = Vec::new();
        OpenReader::new(wire.as_slice(), open).read_to_end(&mut back).unwrap();
        assert_eq!(back, message);
    }

    #[test]
    fn a_tampered_frame_or_a_wrong_key_is_refused() {
        let ((seal, _), (_, open)) = pair();
        let mut wire = Vec::new();
        SealWriter::new(&mut wire, seal).write_all(b"offre de partage").unwrap();
        let mut tampered = wire.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 1;
        let mut out = Vec::new();
        assert!(OpenReader::new(tampered.as_slice(), open).read_to_end(&mut out).is_err());
        let (_, other) = ciphers(b"autre secret", false).unwrap();
        assert!(OpenReader::new(wire.as_slice(), other).read_to_end(&mut out).is_err());
    }
}
