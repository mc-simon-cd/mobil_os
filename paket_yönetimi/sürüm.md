# Sürüm Notları (Changelog) — mcpkg

## v0.1.0 (2026-04-28)
### Added
- Initial modularized project structure using Rust Cargo workspace.
- `mcpkg-core`: Manifest and sandbox policy TOML parsers.
- `mcpkg-crypto`: Ed25519 cryptographic primitives.
- `mcpkg-sandbox`: Linux namespace management and AppArmor profile generation.
- `mcpkg-pipeline`: Initial installation lifecycle state machine.
- `mcpkg-cli`: Base command-line interface.

### Design
- Established technical specifications for `.mcpkg` format.
- Defined security-first sandboxing strategy.
