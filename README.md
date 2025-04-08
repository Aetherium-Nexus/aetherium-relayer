# Aetherium Relayer implementation

Aetherium Relayer

### Setup

- install `rustup`
  - [link here](https://rustup.rs/)

Note: You should be running >= version `1.72.1` of the rustc compiler, you can see that version with this command and
should see similar output:

```
$ rustup --version
rustup 1.26.0 (5af9b9484 2023-04-05)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: The currently active `rustc` version is `rustc 1.72.1 (d5c2e9c34 2023-09-13)`
```

### Overview of Rust Workspace

This is the offchain agents workspace, most notably comprised of the relayer, validator, scraper and the Rust end-to-end tests (in `utils/run-locally`).
You can only run `cargo build`.

#### Apple Silicon

If your device has an Apple Silicon processor, you may need to install Rosetta 2:

```bash
softwareupdate --install-rosetta --agree-to-license
```

### Running Agents Locally

To run the validator, run:

```bash
cargo run --release --bin validator
```

Or build and then run the binary directly:

```bash
cargo build --release --bin validator
./target/release/validator
```

To run the relayer, run:

```bash
cargo run --release --bin relayer
```

Or build and then run the binary directly:

```bash
cargo build --release --bin relayer
./target/release/relayer
```
