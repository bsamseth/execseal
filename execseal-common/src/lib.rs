#![no_std]
extern crate alloc;

use alloc::vec::Vec;

/// Boundary used to separate runtime loader stub from the encrypted executable.
pub const BOUNDARY: [u8; 16] = *b"EXECSEALBOUNDARY";

/// Encrypt data in place using the provided password.
///
/// TODO: Make this a non-trivial transformation.
///
/// # Panics
/// Panics if the provided password is empty.
pub fn encrypt_in_place(data: &mut Vec<u8>, password: &str) {
    let key = *password
        .as_bytes()
        .first()
        .expect("password should not be empty");
    for b in data {
        *b ^= key;
    }
}
