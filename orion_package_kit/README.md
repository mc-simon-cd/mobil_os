# opk — Orion Package Kit

`opk` is a modular, security-first package management system designed for **Orion OS**. It focuses on application isolation (sandboxing), cryptographic integrity, and a streamlined installation pipeline.

## Core Features

- **Modular Architecture**: Built as a Rust Cargo workspace with clearly separated crates for core logic, crypto, sandbox, and pipeline management.
- **Enhanced Security**: Integrated Support for Linux Namespaces and AppArmor profiles.
- **Verified Integrity**: Ed25519 signatures and SHA-256 checksums are mandatory for all packages.
- **Atomic Pipelines**: Installation, updates, and removals are handled via an atomic state machine.

## Project Structure

- `mcpkg-core`: TOML manifest and policy parsing, checksum verification.
- `mcpkg-crypto`: Ed25519 signing and verification.
- `mcpkg-sandbox`: Linux process isolation primitives.
- `mcpkg-pipeline`: The installation lifecycle state machine.
- `mcpkg-cli`: Command-line interface (`opk` binary) for users and developers.

## Usage

```bash
# Install a package
opk install app-1.0.0.opk

# Verify package integrity
opk verify app-1.0.0.opk

# Generate developer keys
opk keygen
```

## Documentation

Detailed technical specifications are available in the `docs/opk` directory.
- [Format Spec](docs/opk/opk-format-spec.md)
- [Pipeline Spec](docs/opk/opk-install-pipeline.md)
- [Sandbox Impl](docs/opk/opk-sandbox-impl.md)

---
*Developed for the Orion OS ecosystem.*
