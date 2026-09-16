//! Identidad criptográfica de la red quipu.
//!
//! Una identidad es un keypair ed25519. La clave pública es la "dirección"
//! a la que se le atribuyen objetos firmados; el `PeerId` es un hash de esa
//! clave pública, pensado para mostrarse y compararse.
//!
//! Este crate no sabe nada de red ni de objetos: solo genera claves,
//! firma bytes y verifica firmas.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const PUBLIC_KEY_LEN: usize = 32;
pub const SECRET_KEY_LEN: usize = 32;
pub const SIGNATURE_LEN: usize = 64;
pub const PEER_ID_LEN: usize = 32;

/// Errores de identidad criptográfica.
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("longitud de clave inválida: esperaba {expected} bytes, recibí {got}")]
    InvalidKeyLength { expected: usize, got: usize },
    #[error("firma inválida o malformada")]
    InvalidSignature,
    #[error("la clave pública no corresponde a un punto ed25519 válido")]
    InvalidPublicKey,
}

/// Clave pública ed25519. Es la identidad pública de un autor.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(pub [u8; PUBLIC_KEY_LEN]);

impl PublicKey {
    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_LEN] {
        &self.0
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex(&self.0))
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug muestra solo el prefijo: suficiente para distinguir claves en logs.
        write!(f, "PublicKey({}…)", hex(&self.0[..4]))
    }
}

/// Firma ed25519 sobre un mensaje.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "serde_big_array::BigArray")] pub [u8; SIGNATURE_LEN]);

impl Signature {
    pub fn as_bytes(&self) -> &[u8; SIGNATURE_LEN] {
        &self.0
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({}…)", hex(&self.0[..4]))
    }
}

/// Identificador de peer derivado de la clave pública (blake3).
/// Sirve para logs, comparaciones y direccionamiento lógico.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; PEER_ID_LEN]);

impl PeerId {
    pub fn as_bytes(&self) -> &[u8; PEER_ID_LEN] {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex(&self.0))
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({}…)", hex(&self.0[..4]))
    }
}

/// Una identidad completa: la clave secreta nunca sale de esta estructura.
pub struct Identity {
    signing_key: SigningKey,
}

impl Identity {
    /// Genera una identidad nueva con aleatoriedad del sistema operativo.
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    /// Reconstruye una identidad desde sus 32 bytes de clave secreta.
    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self, IdentityError> {
        let arr: [u8; SECRET_KEY_LEN] =
            bytes
                .try_into()
                .map_err(|_| IdentityError::InvalidKeyLength {
                    expected: SECRET_KEY_LEN,
                    got: bytes.len(),
                })?;
        Ok(Self {
            signing_key: SigningKey::from_bytes(&arr),
        })
    }

    /// Expone los bytes de la clave secreta para persistencia.
    /// El llamador es responsable de guardarlos de forma segura.
    pub fn secret_bytes(&self) -> [u8; SECRET_KEY_LEN] {
        self.signing_key.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.signing_key.verifying_key().to_bytes())
    }

    pub fn peer_id(&self) -> PeerId {
        PeerId(*blake3::hash(&self.public_key().0).as_bytes())
    }

    /// Firma un mensaje arbitrario.
    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(self.signing_key.sign(message).to_bytes())
    }
}

/// Verifica que `signature` sea una firma válida de `message` hecha por `public_key`.
pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(&public_key.0) else {
        return false;
    };
    let sig = ed25519_dalek::Signature::from_bytes(&signature.0);
    verifying_key.verify(message, &sig).is_ok()
}

/// Deriva el PeerId de una clave pública sin necesidad de la identidad completa.
pub fn peer_id_of(public_key: &PublicKey) -> PeerId {
    PeerId(*blake3::hash(&public_key.0).as_bytes())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let id = Identity::generate();
        let msg = b"hola quipu";
        let sig = id.sign(msg);
        assert!(verify(&id.public_key(), msg, &sig));
    }

    #[test]
    fn verify_rejects_wrong_message() {
        let id = Identity::generate();
        let sig = id.sign(b"mensaje original");
        assert!(!verify(&id.public_key(), b"mensaje alterado", &sig));
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let a = Identity::generate();
        let b = Identity::generate();
        let sig = a.sign(b"data");
        assert!(!verify(&b.public_key(), b"data", &sig));
    }

    #[test]
    fn identity_roundtrip_from_secret_bytes() {
        let id = Identity::generate();
        let restored = Identity::from_secret_bytes(&id.secret_bytes()).unwrap();
        assert_eq!(id.public_key(), restored.public_key());
    }

    #[test]
    fn peer_id_is_deterministic() {
        let id = Identity::generate();
        assert_eq!(id.peer_id(), peer_id_of(&id.public_key()));
    }

    #[test]
    fn from_secret_bytes_rejects_bad_length() {
        assert!(Identity::from_secret_bytes(&[1, 2, 3]).is_err());
    }
}
