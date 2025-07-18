# securepkg
**securepkg** is a package manager in Rust that allows you to securely compress, encrypt, sign, publish, and install packages.

## Install

```bash
git clone https://github.com/Gonzaa21/securepkg.git
cd securepkg
cargo build
```

## CLI commands
*[] are optional arguments*
```bash
cargo run -- init   # start local repo
cargo run -- package [COMMAND]
                - build <path> <name> <version> [--author]   # compress, encrypt and save the package to the DB
                - publish <name> <version> [--export] [--repo <path>]   # sign the .pkg and export it
                - export <name> <version> [--repo <path>]   # copy the .pkg and its signature to a folder
                - install <name> <version> [--from-file <path>]   # verify, decrypt and install the package
                - list   # show all packages registered in the db
```

# Security
- Cipher: ChaCha20-Poly1305
- Signature: RSA 2048 bits + SHA-256
- DB: SQLite

## Structure
```bash
    securepkg/
    ├── src/
    │   ├── main.rs
    │   ├── cli.rs
    │   ├── dsl.rs
    │   ├── package.rs
    │   ├── storage.rs
    │   └── orm/
    │       ├── mod.rs
    │       ├── models.rs 
    │       └── publish_fn.rs 
    └── Cargo.toml
```