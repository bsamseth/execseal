use std::{io::Write, os::unix::fs::PermissionsExt, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use execseal_common::{BOUNDARY, encrypt_in_place};

const RUNTIME: &[u8] = include_bytes!(env!("EXECSEAL_RUNTIME"));

#[derive(Debug, Parser)]
struct Args {
    #[arg(index = 1)]
    executable: PathBuf,
    #[arg(short, long)]
    password: String,
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut contents = std::fs::read(args.executable).context("Reading executable")?;
    let nonce = encrypt_in_place(&mut contents, &args.password).context("Encrypting executable")?;

    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&args.output)?;
    output
        .write_all(RUNTIME)
        .context("Writing runtime stub to output")?;
    output
        .write_all(&BOUNDARY)
        .context("Writing runtime stub boundary to output")?;
    output
        .write_all(&nonce)
        .context("Writing nonce to output")?;
    output
        .write_all(&contents)
        .context("Writing encrypted data")?;
    output.sync_all().context("Flushing data to disk")?;

    // Make the output file executable.
    let mut permissions = output.metadata()?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    output.set_permissions(permissions)?;
    Ok(())
}
