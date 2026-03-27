use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use anyhow::{Context, Result};

use execseal::{BOUNDARY, encrypt_in_place};

fn main() -> Result<()> {
    let password = std::env::var("EXECSEALPASS").context("Reading password from environment")?;
    let mut us = BufReader::new(File::open("/proc/self/exe")?);

    let mut found_internal = false;
    let mut buf = [BOUNDARY[0]; BOUNDARY.len()];
    loop {
        us.skip_until(BOUNDARY[0])
            .context("searching for start of embedded binary")?;
        us.read_exact(&mut buf[1..])
            .context("reading 15 bytes to check for potential execseal boundary")?;
        if buf == BOUNDARY {
            if !found_internal {
                found_internal = true;
                continue;
            }
            break;
        }
    }

    let mut encrypted_content = Vec::with_capacity(
        usize::try_from(
            us.get_ref()
                .metadata()
                .context("Reading metadata on /proc/self/exe")?
                .len(),
        )
        .context("Converting file size to usize")?,
    );
    us.read_to_end(&mut encrypted_content)
        .context("Reading remaining content from /proc/self/exe")?;
    drop(us);

    encrypt_in_place(&mut encrypted_content, &password);

    assert!(
        encrypted_content.starts_with(b"\x7fELF"),
        "Decryption failed"
    );

    let tmp_file = memfd::MemfdOptions::new()
        .close_on_exec(true)
        .create("execseal")?;
    std::io::copy(
        &mut std::io::Cursor::new(encrypted_content),
        &mut tmp_file.as_file(),
    )
    .context("Copying decrypted content to memfd")?;

    // TODO: Copy out argv and env
    nix::unistd::fexecve(tmp_file.into_file(), &[c"execseal"], &[c""])
        .context("Executing decrypted payload")?;

    panic!("Executing decrypted program failed!");
}
