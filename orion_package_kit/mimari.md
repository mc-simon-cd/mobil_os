# Mimari Bakış (Architecture) — mcpkg

`mcpkg` follows a strictly modular, decoupled architecture.

## Hierarchy of Modules

```mermaid
graph TD
    CLI[mcpkg-cli] --> Pipeline[mcpkg-pipeline]
    Pipeline --> Core[mcpkg-core]
    Pipeline --> Crypto[mcpkg-crypto]
    Pipeline --> Sandbox[mcpkg-sandbox]
    
    Core --> Manifest[manifest.toml Parser]
    Core --> Policy[sandbox.policy Parser]
    Core --> Checksum[SHA-256 Verifier]
    
    Crypto --> Ed25519[Sign & Verify]
    
    Sandbox --> NS[Namespaces]
    Sandbox --> AA[AppArmor]
    Sandbox --> SC[Seccomp]
```

## Component Roles

1.  **Orchestrator (Pipeline)**: Manages the `InstallState` state machine.
2.  **Security Engine (Sandbox)**: Translates high-level policies into kernel-level restrictions.
3.  **Trust Module (Crypto)**: Ensures packages come from verified developers.
4.  **Schema Module (Core)**: Validates package metadata against specifications.
