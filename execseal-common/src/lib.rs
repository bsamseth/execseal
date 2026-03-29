use core::fmt::Display;

use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{AeadCore, AeadMutInPlace, KeyInit, OsRng},
};
use sha3::Digest;

/// Boundary used to separate runtime loader stub from the encrypted executable.
pub const BOUNDARY: [u8; 16] = *b"EXECSEALBOUNDARY";

/// The possible errors when encrypting/decrypting.
#[derive(Debug)]
pub enum CryptoError {
    Encryption,
    Decryption,
    WrongNonceSize,
}

/// Encrypt data in place using the provided password.
///
/// # Errors
/// Errors with a [`CryptoError`] if something goes wrong. See the possible variants for details.
pub fn encrypt_in_place(data: &mut Vec<u8>, password: &str) -> Result<Nonce, CryptoError> {
    let mut cipher = key_from_password(password);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    cipher
        .encrypt_in_place(&nonce, b"EXECSEAL", data)
        .map_err(|_| CryptoError::Encryption)?;

    Ok(nonce)
}

/// Decrypt data in place using the provided password.
///
/// # Errors
/// Errors with a [`CryptoError`] if something goes wrong. See the possible variants for details.
pub fn decrypt_in_place(
    data: &mut Vec<u8>,
    password: &str,
    nonce: &[u8],
) -> Result<(), CryptoError> {
    let mut cipher = key_from_password(password);
    if nonce.len() != 12 {
        return Err(CryptoError::WrongNonceSize);
    }
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt_in_place(nonce, b"EXECSEAL", data)
        .map_err(|_| CryptoError::Decryption)?;
    Ok(())
}

fn key_from_password(password: &str) -> ChaCha20Poly1305 {
    let mut hasher = sha3::Sha3_256::new_with_prefix(b"execseal");
    hasher.update(password.as_bytes());
    let key = hasher.finalize();
    ChaCha20Poly1305::new(&key)
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::write;
        match self {
            CryptoError::Encryption => write!(f, "encryption failed"),
            CryptoError::Decryption => write!(f, "decryption failed"),
            CryptoError::WrongNonceSize => write!(f, "wrong nonce size"),
        }
    }
}
impl core::error::Error for CryptoError {}
