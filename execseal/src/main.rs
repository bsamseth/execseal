use std::{
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;

use execseal_common::{BOUNDARY, encrypt_in_place};

const RUNTIME: &[u8] = include_bytes!(env!("EXECSEAL_RUNTIME"));

/// Execseal - Password Protected executables.
///
/// Turn any ELF executable into a self-decrypting copy. The resulting binary will attempt to
/// self-decrypt based on a password provided as an environment variable `EXECSEALPASS`. If this is
/// set to the correct password, the binary acts just like the original, unencrypted one. If the
/// password is not provided, or a wrong one is provided the binary exits with a error.
#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
struct Args {
    /// The executable to encrypt, or use `-` to read it from stdin.
    #[arg(index = 1)]
    executable: PathBuf,
    /// The password used to encrypt/decrypt.
    #[arg(short, long)]
    password: String,
    /// Where to write the output.
    #[arg(short, long)]
    output: PathBuf,
    /// Recover the original executable from an already encrypted file.
    #[arg(short, long, default_value = "false")]
    decrypt: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let contents = if args.executable.as_os_str() == "-" {
        let mut exec = Vec::new();
        std::io::stdin()
            .read_to_end(&mut exec)
            .context("Reading executable from stdin")?;
        exec
    } else {
        std::fs::read(&args.executable).context("Reading executable")?
    };

    if args.decrypt {
        make_decrypted(&args, contents).context("Decrypting binary")
    } else {
        make_encrypted(&args, contents).context("Making encrypted binary")
    }
}

fn make_encrypted(args: &Args, mut contents: Vec<u8>) -> Result<()> {
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

    make_file_executable(&output).context("Making output executable")?;

    println!(
        "Password protected copy written to {}.",
        args.output.display()
    );
    println!("To run it:");
    println!("\tEXECSEALPASS=*** {}", args.output.display());
    Ok(())
}

fn make_decrypted(args: &Args, contents: Vec<u8>) -> Result<()> {
    let contents = execseal_common::decrypt_executable(contents, &args.password)
        .context("Decrypting executable")?;
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&args.output)?;
    output
        .write_all(&contents)
        .context("Writing decrypted executable to disk")?;
    output.sync_all().context("Flushing data to disk")?;
    make_file_executable(&output).context("Makig output executable")?;

    println!(
        "Decrypted original executable written to {}.",
        args.output.display()
    );
    Ok(())
}

fn make_file_executable(file: &std::fs::File) -> Result<()> {
    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    file.set_permissions(permissions)?;
    Ok(())
}
