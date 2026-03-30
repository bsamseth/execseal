# Execseal - Password Protected Executables

This tool lets you encrypt a binary (currently only linux ELF) into a
self-decrypting copy. The generated binary will try to decrypt itself using a
password supplied through the `EXECSEALPASS` environment variable. When the
correct password is provided, it behaves identically to the original
unencrypted program. If the password is missing or incorrect, the binary
terminates with an error.

## Why?

Because I wanted to. But in theory it could be useful to distribute a program
while restricting who can actually run it.
