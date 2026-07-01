# Project Progress — opk

## Milestone 1: Modular Foundation ✅
- [x] Workspace structure initialized
- [x] Sub-crates created (`core`, `crypto`, `sandbox`, `pipeline`, `cli`)
- [x] Basic TOML parsing for Manifest and Policy
- [x] Ed25519 Cryptographic primitives
- [x] Linux Namespace discovery and primitives
- [x] Installation state machine (Basic)

## Milestone 2: Functional Sandbox ✅
- [x] Implement Seccomp BPF filters
- [x] Connect AppArmor profile generator to `apparmor_parser`
- [x] Implement rootless user namespace mapping

## Milestone 3: Production Pipeline ✅
- [x] Zip archive handling and internal checksum validation
- [x] Atomic file moving with rollback support
- [x] Desktop entry generation

## Milestone 4: Store & Registry Integration ✅
- [x] Registry API client implementation
- [x] Package downloading with cache management

## Milestone 5: Developer Tooling ✅
- [x] `opk pack` — dizinden .opk üretme
- [x] `opk sign` — paket imzalama
- [x] `opk verify` — yerel doğrulama
- [x] `opk publish` — registry'ye yükleme
- [x] `opk keygen` — anahtar üretimi
