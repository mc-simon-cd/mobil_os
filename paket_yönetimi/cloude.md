# Cloud Strategy — mcpkg

`mcpkg` is designed for cloud-native package distribution and verification.

## Cloud Registry Integrations

- **Central Registry (CDN)**: Uses `https://cdn.mcsimon.app` for high-performance package delivery.
- **Scalability**: Designed to handle millions of package lookups via a stateless Registry API.
- **CI/CD Pipeline**: Developers can push packages to the cloud, where they are automatically signed by the Mcsimon Gatekeeper Service.

## Deployment Environment

- **Target**: Mcsimon OS (Arch Linux-based)
- **Runtime**: Linux kernel 6.x+ with cgroups v2, AppArmor, and Namespace support.
- **Backend**: Rust-based stateful services integrated with SQLite for local auditing.
