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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygen_and_sign_verify() {
        let (signing_key, verifying_key) = CryptoManager::generate_keypair();
        let message = b"Hello Orion Package Kit";

        let sig = CryptoManager::sign(&signing_key, message);
        let res = CryptoManager::verify(&verifying_key, message, &sig);
        assert!(res.is_ok());
    }

    #[test]
    fn test_signature_tampering_fails() {
        let (signing_key, verifying_key) = CryptoManager::generate_keypair();
        let message = b"Critical security payload";

        let sig = CryptoManager::sign(&signing_key, message);
        let mut sig_bytes = CryptoManager::signature_to_bytes(&sig);
        sig_bytes[0] ^= 0xFF; // Flip bits to tamper

        let tampered_sig = CryptoManager::signature_from_bytes(&sig_bytes).unwrap();
        let res = CryptoManager::verify(&verifying_key, message, &tampered_sig);
        assert!(res.is_err());
    }

    #[test]
    fn test_message_tampering_fails() {
        let (signing_key, verifying_key) = CryptoManager::generate_keypair();
        let message = b"Original message";
        let tampered_message = b"Tampered message";

        let sig = CryptoManager::sign(&signing_key, message);
        let res = CryptoManager::verify(&verifying_key, tampered_message, &sig);
        assert!(res.is_err());
    }

    #[test]
    fn test_signature_serialization_roundtrip() {
        let (signing_key, _) = CryptoManager::generate_keypair();
        let sig = CryptoManager::sign(&signing_key, b"test roundtrip");
        let bytes = CryptoManager::signature_to_bytes(&sig);
        assert_eq!(bytes.len(), 64);

        let recovered = CryptoManager::signature_from_bytes(&bytes).unwrap();
        assert_eq!(sig, recovered);
    }

    #[test]
    fn test_invalid_signature_length_fails() {
        let short_bytes = [0u8; 32];
        let res = CryptoManager::signature_from_bytes(&short_bytes);
        assert!(res.is_err());
    }
}
