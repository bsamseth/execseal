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
    #[arg(short, long, env = "EXECSEALPASS")]
    password: String,
    /// Where to write the output.
    ///
    /// By default the original file will be replaced. If the input was read from stdin the
    /// default output is stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,
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
        make_decrypted(args, contents).context("Decrypting binary")
    } else {
        make_encrypted(args, contents).context("Making encrypted binary")
    }
}

fn make_encrypted(args: Args, mut contents: Vec<u8>) -> Result<()> {
    fn write_to_output(
        mut output: impl std::io::Write,
        contents: &[u8],
        nonce: &[u8],
    ) -> Result<()> {
        output
            .write_all(RUNTIME)
            .context("Writing runtime stub to output")?;
        output
            .write_all(&BOUNDARY)
            .context("Writing runtime stub boundary to output")?;
        output.write_all(nonce).context("Writing nonce to output")?;
        output.write_all(contents).context("Writing encrypted data")
    }

    let nonce = encrypt_in_place(&mut contents, &args.password).context("Encrypting executable")?;

    if args.output.is_some() || args.executable.as_os_str() != "-" {
        let replace = args.output.is_none();
        let output_name = args.output.unwrap_or(args.executable);
        let mut output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&output_name)?;
        write_to_output(&mut output, &contents, &nonce).context("Writing output executable")?;
        output.sync_all().context("Flushing data to disk")?;
        if !replace {
            make_file_executable(&output).context("Making output executable")?;
        }

        eprintln!(
            "Password protected executable written to {}.",
            output_name.display()
        );
        eprintln!("To run it:");
        eprintln!("\tEXECSEALPASS=*** {}", output_name.display());
    } else {
        write_to_output(std::io::stdout(), &contents, &nonce)
            .context("Writing output executable to stdout")?;
        eprintln!("Password protected executable written to stdout.");
    }
    Ok(())
}

fn make_decrypted(args: Args, contents: Vec<u8>) -> Result<()> {
    let contents = execseal_common::decrypt_executable(contents, &args.password)
        .context("Decrypting executable")?;

    if args.output.is_some() || args.executable.as_os_str() != "-" {
        let replace = args.output.is_none();
        let output_name = args.output.unwrap_or(args.executable);
        let mut output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&output_name)?;
        output
            .write_all(&contents)
            .context("Writing decrypted executable to disk")?;
        output.sync_all().context("Flushing data to disk")?;
        if !replace {
            make_file_executable(&output).context("Making output executable")?;
        }

        eprintln!(
            "Decrypted original executable written to {}.",
            output_name.display()
        );
    } else {
        std::io::stdout()
            .write_all(&contents)
            .context("Writing decrypted executable to stdout")?;
        eprintln!("Decrypted original executable written to stdout.",);
    }
    Ok(())
}

fn make_file_executable(file: &std::fs::File) -> Result<()> {
    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    file.set_permissions(permissions)?;
    Ok(())
}
