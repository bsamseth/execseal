use core::fmt::Display;

use aes_gcm_siv::{
    Aes256GcmSiv, Nonce,
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
    let nonce = Aes256GcmSiv::generate_nonce(&mut OsRng);
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

/// Decrypt an encrypted ELF.
///
/// # Errors
/// Can fail if the provided contents are malformed, the password is incorrect, or the resulting
/// payload does not appear to be an ELF file.
pub fn decrypt_executable(mut contents: Vec<u8>, password: &str) -> anyhow::Result<Vec<u8>> {
    use anyhow::Context;

    let boundary_offset = contents
        .array_windows()
        .enumerate()
        .rev()
        .find_map(|(offset, window)| {
            if *window == BOUNDARY {
                Some(offset)
            } else {
                None
            }
        })
        .context("Searching for boundary to encrypted binary")?;

    let nonce = contents
        .get(boundary_offset + BOUNDARY.len()..)
        .into_iter()
        .find_map(|slice| slice.get(..12))
        .context("Extracting encryption nonce from binary")?;
    // SAFETY: The size is exactly 12 bytes.
    let nonce: [u8; 12] = unsafe { nonce.try_into().unwrap_unchecked() };
    let mut contents = contents.split_off(boundary_offset + BOUNDARY.len() + nonce.len());
    decrypt_in_place(&mut contents, password, &nonce).context("Decrypting binary")?;

    if !contents.starts_with(b"\x7fELF") {
        anyhow::bail!("Decryption OK but didn't produce an ELF file, abort!");
    }
    Ok(contents)
}

fn key_from_password(password: &str) -> Aes256GcmSiv {
    let mut hasher = sha3::Sha3_256::new_with_prefix(b"execseal");
    hasher.update(password.as_bytes());
    let key = hasher.finalize();
    Aes256GcmSiv::new(&key)
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
