use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

pub struct CryptoManager;

impl CryptoManager {
    /// Generates a new Ed25519 key pair.
    pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = VerifyingKey::from(&signing_key);
        (signing_key, verifying_key)
    }

    /// Signs a message using the provided signing key.
    pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
        signing_key.sign(message)
    }

    /// Verifies a signature using the provided verifying key.
    pub fn verify(verifying_key: &VerifyingKey, message: &[u8], signature: &Signature) -> Result<(), ed25519_dalek::SignatureError> {
        verifying_key.verify(message, signature)
    }

    /// Converts a signature to a byte vector.
    pub fn signature_to_bytes(signature: &Signature) -> Vec<u8> {
        signature.to_bytes().to_vec()
    }

    /// Creates a signature from bytes.
    pub fn signature_from_bytes(bytes: &[u8]) -> Result<Signature, ed25519_dalek::SignatureError> {
        let bytes_array: [u8; 64] = bytes.try_into().map_err(|_| ed25519_dalek::SignatureError::new())?;
        Ok(Signature::from_bytes(&bytes_array))
    }
}
