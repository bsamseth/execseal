# Execseal - Password Protected Executables

This tool lets you encrypt a binary (currently only linux ELF) into a
self-decrypting copy. The generated binary will try to decrypt itself using a
password supplied through the `EXECSEALPASS` environment variable. When the
correct password is provided, it behaves identically to the original
unencrypted program. If the password is missing or incorrect, the binary
terminates with an error.

## Example

```text
❯ execseal -p password123 -o secret /bin/ls
Password protected executable written to secret.
To run it:
        EXECSEALPASS=*** secret

❯ ./secret -la secret
This executable is encrypted. To unlock, provide the password as an environment variable: EXECSEALPASS

❯ EXECSEALPASS=password123 ./secret -la secret
-rwxrwxr-x 1 user user 212340 Mar 30 10:41 secret

❯ execseal -p password123 -d -o recovered secret
Decrypted original executable written to recovered.

❯ md5sum recovered /bin/ls
5229649db44886ed74f9096b373032f4  recovered
5229649db44886ed74f9096b373032f4  /bin/ls
```

## Install

`execseal` works by prepending the encrypted binary with a runtime that can
decrypt itself. This runtime takes up some space. The space it takes depends on
what features are enabled

|    Features    |   Size   |
|----------------|----------|
| Nothing        |  365 KB  |
| `upx`          |  151 KB  |
| `nightly`      |  116 KB  |
| `minimal-size` |   54 KB  |

The feature `minimal-size` exists as a shorthand to enable all space saving features.


```bash
cargo install execseal --features ...
```

### Requirements for Smallest Possible Runtime

```bash
sudo apt-get install -y upx-ucl  # To compress the runtime.

# The rust standard library used by the runtime will be built from source,
# with options set to minmize the size. This requires the `rust-src` component.
# Building without this installed will emitt an error telling you to install the component.
# Something like this, adapting the toolchain name as needed:
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
```

## Why?

Because I wanted to. But in theory it could be useful to distribute a program
while restricting who can actually run it.
