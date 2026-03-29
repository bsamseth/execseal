use std::{
    convert::Infallible,
    ffi::CString,
    io::Write,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStringExt,
    },
};

use anyhow::Context;

use execseal_common::{BOUNDARY, decrypt_in_place};

fn main() {
    if std::env::var_os("EXECSEALPASS").is_none() {
        println!(
            "This executable is encrypted. To unlock, provide the password as an environment variable: EXECSEALPASS"
        );
        return;
    }
    // This only returns if it returns with an error, so unconditional unpack is possible.
    let Err(err) = decrypt_and_run();
    eprintln!("Failed to start. Error context: {err:#}");
    unsafe { libc::exit(libc::EXIT_FAILURE) }
}

/// Attempt to decrypt the embedded binary and exec it.
///
/// This will either not return or produce an error. It will never return an `Ok`.
fn decrypt_and_run() -> Result<Infallible, anyhow::Error> {
    let password =
        std::env::var("EXECSEALPASS").context("Getting EXECSEALPASS environment variable")?;
    let mut contents = std::fs::read("/proc/self/exe").context("Reading /proc/self/exe")?;

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

    let nonce: [u8; 12] = contents
        .get(boundary_offset + BOUNDARY.len()..)
        .into_iter()
        .find_map(|slice| slice.get(..12))
        .context("Extracting encryption nonce from binary")?
        .try_into()
        .unwrap(); // Length checked.
    let mut contents = contents.split_off(boundary_offset + BOUNDARY.len() + nonce.len());
    decrypt_in_place(&mut contents, &password, &nonce).context("Decrypting binary")?;

    if !contents.starts_with(b"\x7fELF") {
        anyhow::bail!("Decryption OK but didn't produce an ELF file, abort!");
    }

    let mut memfd = {
        let fd = unsafe { libc::memfd_create(c"execseal".as_ptr(), libc::MFD_CLOEXEC) };
        if fd < 0 {
            anyhow::bail!("Failed to create memfd for decrypted binary.");
        }
        unsafe { std::fs::File::from_raw_fd(fd) }
    };
    std::io::copy(&mut std::io::Cursor::new(contents), &mut memfd)
        .context("Copying decrypted binary into memfd")?;
    memfd.flush()?;

    let argv_storage = std::env::args_os()
        .map(|arg| {
            CString::new(arg.into_vec())
                .expect("argument can't contain null bytes")
                .into_boxed_c_str()
        })
        .collect::<Vec<_>>();
    let argv = argv_storage
        .iter()
        .map(|arg| (**arg).as_ptr())
        .chain([std::ptr::null()])
        .collect::<Vec<_>>();

    let envp_storage = std::env::vars_os()
        .filter_map(|(key, value)| {
            if key == "EXECSEALPASS" {
                return None;
            }
            let mut env_var = key.into_vec();
            env_var.push(b'=');
            env_var.extend_from_slice(&value.into_vec());
            Some(
                CString::new(env_var)
                    .expect("argument can't contain null bytes")
                    .into_boxed_c_str(),
            )
        })
        .collect::<Vec<_>>();
    let envp = envp_storage
        .iter()
        .map(|var| (**var).as_ptr())
        .chain([std::ptr::null()])
        .collect::<Vec<_>>();

    unsafe { libc::fexecve(memfd.as_raw_fd(), argv.as_ptr(), envp.as_ptr()) };
    anyhow::bail!(
        "Executing decrypted program failed with error: {}",
        std::io::Error::last_os_error()
    );
}
