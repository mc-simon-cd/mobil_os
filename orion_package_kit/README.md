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
# Paket arama ve kurma
opk search editor
opk install com.orion.editor -y

# Uygulama çalıştırma ve durum kontrolü
opk launch com.orion.editor
opk list
opk info com.orion.editor

# Güncelleme ve kaldırma
opk update com.orion.editor
opk uninstall com.orion.editor --purge

# Güvenlik ve denetim kayıtları (Orion Bekçi)
opk audit list
opk audit export -f json -o audit.json

# Geliştirici araçları (Paketleme & İmzalama)
opk keygen
opk pack ./my_app -o my_app-1.0.0.opk
opk sign my_app-1.0.0.opk
opk verify my_app-1.0.0.opk
opk publish my_app-1.0.0.opk
```

## Documentation

Kapsamlı teknik dokümantasyon, şartnameler ve geliştirici kılavuzları `docs/` altında yer almaktadır:

### Temel Kılavuzlar ve Şartnameler
- [Mimari (ARCHITECTURE)](docs/ARCHITECTURE.md)
- [CLI Referansı (CLI_REFERENCE)](docs/CLI_REFERENCE.md)
- [Kurulum Kılavuzu (INSTALL)](docs/INSTALL.md)
- [Güvenlik Politikası (SECURITY)](docs/SECURITY.md)
- [Katkıda Bulunma (CONTRIBUTING)](docs/CONTRIBUTING.md)
- [Değişiklik Günlüğü (CHANGELOG)](docs/CHANGELOG.md)
- [Test Rehberi (TESTING)](docs/TESTING.md)
- [Kullanım Şartları (TERMS / ŞARTLAR)](docs/TERMS.md) & [Türkçe Şartlar](docs/SARTLAR.md)

### Özel Entegrasyon Rehberleri
- [Veri Tasarrufu (DATA_SAVING_GUIDE)](docs/DATA_SAVING_GUIDE.md)
- [Flatpak Uyumluluk (FLATPAK_PACKAGING_GUIDE)](docs/FLATPAK_PACKAGING_GUIDE.md)
- [Ödeme Entegrasyonu (PAYMENT_INTEGRATION)](docs/PAYMENT_INTEGRATION.md)
- [Özel Sunucu Barındırma (SELF_HOSTING_GUIDE)](docs/SELF_HOSTING_GUIDE.md)
- [Uygulama Bağlama & IPC (UYGULAMA_BAGLAMA)](docs/UYGULAMA_BAGLAMA.md)
- [Çeviri Rehberi (TRANSLATION_GUIDE)](docs/TRANSLATION_GUIDE.md)
- [Versiyonlama (VERSION)](docs/VERSION.md)

### OPK Teknik Spesifikasyonları (`docs/opk`)
- [Format Şartnamesi](docs/opk/opk-format-spec.md)
- [Kurulum Pipeline Şartnamesi](docs/opk/opk-install-pipeline.md)
- [Sandbox Uygulama Şartnamesi](docs/opk/opk-sandbox-impl.md)
- [Yaşam Döngüsü Şartnamesi](docs/opk/opk-lifecycle-spec.md)
- [Yol Haritası & Eksikler](docs/opk/opk-roadmap.md)
- [Proje İlerlemesi](progress.md)

---
*Developed for the Orion OS ecosystem.*
