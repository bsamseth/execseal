# Execseal - Password Protected Executables

This tool lets you encrypt a binary (currently only linux ELF) into a
self-decrypting copy. The resulting binary will attempt to self-decrypt based
on a password provided as an environment variable `EXECSEALPASS`. If this is
set to the correct password, the binary acts just like the original,
unencrypted one. If the password is not provided, or a wrong one is provided
the binary exits with a error.

## Why?

Because I wanted to. But in theory it could be useful to distribute a program
while restricting who can actually run it.
