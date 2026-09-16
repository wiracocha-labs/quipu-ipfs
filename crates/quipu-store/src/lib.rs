//! Almacenamiento genérico de objetos firmados para la red quipu.
//!
//! El `SignedObject` es deliberadamente agnóstico: el campo `kind` es un
//! string libre ("message", "post", "model-weights", lo que una app defina)
//! y `data` son bytes opacos. Este crate no interpreta ninguno de los dos —
//! esa es la decisión central de diseño del proyecto.
//!
//! `ObjectStore` es el trait mínimo que la capa de red necesita. `MemoryStore`
//! es la implementación de la Fase 0 (volátil); la persistencia en disco
//! llega en la Fase 2 como otra implementación del mismo trait.

use quipu_identity::{verify, Identity, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub const OBJECT_HASH_LEN: usize = 32;

/// Hash de contenido de un objeto (blake3 sobre sus campos canónicos).
/// Es la "dirección" por la que la red pide objetos.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectHash(pub [u8; OBJECT_HASH_LEN]);

impl ObjectHash {
    pub fn as_bytes(&self) -> &[u8; OBJECT_HASH_LEN] {
        &self.0
    }

    /// Parsea un hash desde su representación hex de 64 caracteres.
    pub fn from_hex(s: &str) -> Result<Self, StoreError> {
        let bytes = decode_hex(s).ok_or(StoreError::InvalidHashHex)?;
        let arr: [u8; OBJECT_HASH_LEN] =
            bytes.try_into().map_err(|_| StoreError::InvalidHashHex)?;
        Ok(ObjectHash(arr))
    }
}

impl fmt::Display for ObjectHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ObjectHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectHash({}…)", &self.to_string()[..8])
    }
}

/// Errores del store.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("la firma del objeto no verifica contra su autor")]
    InvalidSignature,
    #[error("hash hex inválido (se esperaban 64 caracteres hex)")]
    InvalidHashHex,
}

/// Un objeto de datos genérico firmado por su autor.
///
/// La firma cubre el hash de contenido de (kind, data, author, created_at),
/// así que cualquier alteración de esos campos invalida la verificación.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedObject {
    /// Tipo de objeto, definido por la aplicación (el core no lo interpreta).
    pub kind: String,
    /// Payload opaco. La app decide el formato (JSON, protobuf, texto plano...).
    pub data: Vec<u8>,
    /// Clave pública del autor.
    pub author: PublicKey,
    /// Timestamp de creación (segundos desde UNIX_EPOCH).
    pub created_at: u64,
    /// Firma ed25519 del autor sobre `content_hash()`.
    pub signature: Signature,
}

impl SignedObject {
    /// Crea y firma un objeto nuevo con la identidad dada.
    pub fn new(kind: impl Into<String>, data: Vec<u8>, author: &Identity) -> Self {
        let mut obj = Self {
            kind: kind.into(),
            data,
            author: author.public_key(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            signature: Signature([0u8; 64]),
        };
        let hash = obj.content_hash();
        obj.signature = author.sign(hash.as_bytes());
        obj
    }

    /// Serialización canónica de los campos firmados.
    /// NO depende del formato de transporte: es una concatenación
    /// con prefijos de longitud explícitos.
    fn hashable_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.kind.len() + 8 + self.data.len() + 32 + 8);
        buf.extend_from_slice(&(self.kind.len() as u64).to_be_bytes());
        buf.extend_from_slice(self.kind.as_bytes());
        buf.extend_from_slice(&(self.data.len() as u64).to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf.extend_from_slice(self.author.as_bytes());
        buf.extend_from_slice(&self.created_at.to_be_bytes());
        buf
    }

    /// El hash de contenido: lo que se firma y lo que direcciona al objeto.
    pub fn content_hash(&self) -> ObjectHash {
        ObjectHash(*blake3::hash(&self.hashable_bytes()).as_bytes())
    }

    /// Verifica que la firma corresponda al autor y al contenido actual.
    pub fn verify_signature(&self) -> bool {
        verify(
            &self.author,
            self.content_hash().as_bytes(),
            &self.signature,
        )
    }
}

impl fmt::Debug for SignedObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignedObject")
            .field("kind", &self.kind)
            .field("data_len", &self.data.len())
            .field("author", &self.author)
            .field("hash", &self.content_hash())
            .finish()
    }
}

/// El contrato mínimo que la capa de red necesita de un almacenamiento.
/// Cualquier backend (memoria, disco, remoto) implementa esto.
pub trait ObjectStore: Send + Sync {
    /// Guarda un objeto. Implementaciones deben rechazar firmas inválidas.
    /// Retorna el hash de contenido del objeto guardado.
    fn put(&self, object: SignedObject) -> Result<ObjectHash, StoreError>;

    /// Recupera un objeto por su hash de contenido.
    fn get(&self, hash: &ObjectHash) -> Option<SignedObject>;

    fn contains(&self, hash: &ObjectHash) -> bool {
        self.get(hash).is_some()
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Store en memoria. Volátil — suficiente para Fase 0.
#[derive(Default)]
pub struct MemoryStore {
    objects: RwLock<HashMap<ObjectHash, SignedObject>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ObjectStore for MemoryStore {
    fn put(&self, object: SignedObject) -> Result<ObjectHash, StoreError> {
        if !object.verify_signature() {
            return Err(StoreError::InvalidSignature);
        }
        let hash = object.content_hash();
        self.objects
            .write()
            .expect("store lock poisoned")
            .insert(hash, object);
        Ok(hash)
    }

    fn get(&self, hash: &ObjectHash) -> Option<SignedObject> {
        self.objects
            .read()
            .expect("store lock poisoned")
            .get(hash)
            .cloned()
    }

    fn len(&self) -> usize {
        self.objects.read().expect("store lock poisoned").len()
    }
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_object(data: &[u8]) -> SignedObject {
        SignedObject::new("note", data.to_vec(), &Identity::generate())
    }

    #[test]
    fn new_object_verifies() {
        assert!(make_object(b"hola").verify_signature());
    }

    #[test]
    fn tampered_data_fails_verification() {
        let mut obj = make_object(b"contenido original");
        obj.data = b"contenido alterado".to_vec();
        assert!(!obj.verify_signature());
    }

    #[test]
    fn tampered_kind_fails_verification() {
        let mut obj = make_object(b"data");
        obj.kind = "otro-kind".to_string();
        assert!(!obj.verify_signature());
    }

    #[test]
    fn store_put_get_roundtrip() {
        let store = MemoryStore::new();
        let obj = make_object(b"payload");
        let hash = store.put(obj.clone()).unwrap();
        assert_eq!(store.get(&hash).unwrap().data, b"payload");
        assert!(store.contains(&hash));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn store_rejects_invalid_signature() {
        let store = MemoryStore::new();
        let mut obj = make_object(b"data");
        obj.data = "modificado después de firmar".as_bytes().to_vec();
        assert!(matches!(store.put(obj), Err(StoreError::InvalidSignature)));
        assert!(store.is_empty());
    }

    #[test]
    fn hash_hex_roundtrip() {
        let obj = make_object(b"x");
        let hash = obj.content_hash();
        let parsed = ObjectHash::from_hex(&hash.to_string()).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn from_hex_rejects_garbage() {
        assert!(ObjectHash::from_hex("zzzz").is_err());
        assert!(ObjectHash::from_hex("abcd").is_err()); // muy corto
    }
}
