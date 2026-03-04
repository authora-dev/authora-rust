use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::error::AuthoraError;

pub struct KeyPair {
    pub private_key: String,
    pub public_key: String,
}

pub fn generate_key_pair() -> KeyPair {
    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    KeyPair {
        private_key: to_base64url(signing_key.as_bytes()),
        public_key: to_base64url(verifying_key.as_bytes()),
    }
}

pub fn get_public_key(private_key_b64: &str) -> Result<String, AuthoraError> {
    let seed = from_base64url(private_key_b64)?;
    let bytes: [u8; 32] = seed
        .try_into()
        .map_err(|_| AuthoraError::Crypto("invalid private key length".into()))?;
    let signing_key = SigningKey::from_bytes(&bytes);
    Ok(to_base64url(signing_key.verifying_key().as_bytes()))
}

pub fn to_base64url(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

pub fn from_base64url(b64url: &str) -> Result<Vec<u8>, AuthoraError> {
    URL_SAFE_NO_PAD
        .decode(b64url)
        .map_err(|e| AuthoraError::Crypto(format!("base64url decode: {e}")))
}

pub fn build_signature_payload(
    method: &str,
    path: &str,
    timestamp: &str,
    body: Option<&str>,
) -> String {
    let body_hash = match body {
        Some(b) if !b.is_empty() => sha256_hash(b),
        _ => String::new(),
    };
    format!(
        "{}\n{}\n{}\n{}",
        method.to_uppercase(),
        path,
        timestamp,
        body_hash
    )
}

pub fn sign(message: &str, private_key_b64: &str) -> Result<String, AuthoraError> {
    let seed = from_base64url(private_key_b64)?;
    let bytes: [u8; 32] = seed
        .try_into()
        .map_err(|_| AuthoraError::Crypto("invalid private key length".into()))?;
    let signing_key = SigningKey::from_bytes(&bytes);
    let signature = signing_key.sign(message.as_bytes());
    Ok(to_base64url(&signature.to_bytes()))
}

pub fn verify(message: &str, signature_b64: &str, public_key_b64: &str) -> bool {
    let Ok(sig_bytes) = from_base64url(signature_b64) else {
        return false;
    };
    let Ok(pub_bytes) = from_base64url(public_key_b64) else {
        return false;
    };
    let Ok(sig_arr): Result<[u8; 64], _> = sig_bytes.try_into() else {
        return false;
    };
    let Ok(pub_arr): Result<[u8; 32], _> = pub_bytes.try_into() else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pub_arr) else {
        return false;
    };
    let signature = ed25519_dalek::Signature::from_bytes(&sig_arr);
    verifying_key.verify(message.as_bytes(), &signature).is_ok()
}

pub fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    to_base64url(&hasher.finalize())
}
