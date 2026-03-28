#![no_std]

use core::fmt::Display;

/// Boundary used to separate runtime loader stub from the encrypted executable.
pub const BOUNDARY: [u8; 16] = *b"EXECSEALBOUNDARY";

/// The possible errors when encrypting/decrypting.
#[derive(Debug)]
pub enum CryptoError {
    IncorrectPassword,
}

/// Encrypt data in place using the provided password.
///
/// TODO: Make this a non-trivial transformation.
///
/// # Errors
/// Errors with a [`CryptoError`] if something goes wrong. See the possible variants for details.
pub fn encrypt_in_place(data: &mut [u8], password: &str) -> Result<(), CryptoError> {
    let key = *password
        .as_bytes()
        .first()
        .ok_or(CryptoError::IncorrectPassword)?;
    for b in data {
        *b ^= key;
    }
    Ok(())
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::write;
        match self {
            CryptoError::IncorrectPassword => write!(f, "incorrect password provided"),
        }
    }
}
impl core::error::Error for CryptoError {}
