# Sürüm Notları (Changelog) — opk (Orion Package Kit)

## v0.2.0 (2026-10-13)
### Eklendi
- **Paket Yaşam Döngüsü Yönetimi (Milestone 6):**
  - `opk uninstall <app_id>`: Uygulama süreçlerini sonlandırma (`SIGTERM`), AppArmor profilini kaldırma, masaüstü kısayolu, ikon ve dosya temizliği desteği (`--purge` ile kullanıcı verilerini temizleme seçeneği).
  - `opk list`: Kurulu tüm paketleri tablo ve JSON (`--json`) olarak listeleme.
  - `opk info <id|opk>`: Paket metadata, deklaratif güvenlik izinleri ve kurulum durumu sorgulama.
  - `opk update [id]`: Registry üzerinden yeni sürüm denetimi ve otomatik güncelleme indirme.
- **Sistem & Dizin İyileştirmeleri (Milestone 3):**
  - Kurulum hedefi `/tmp` yerine kalıcı `~/.local/share/orion/apps` (rootless) veya `/opt/orion/apps` (root) dizinine bağlandı.
  - `assets/icon.png` dosyasının `~/.local/share/icons/orion/` dizinine çıkarılması ve masaüstü başlatıcı entegrasyonu.
  - `tempfile::into_path` uyarısı `TempDir::keep()` ile güncellendi.
- **Orion Bekçi (Gatekeeper) & SQLite Audit Log (Milestone 7):**
  - `~/.local/share/orion/bekci/audit.db` SQLite veritabanı şeması ve `rusqlite` entegrasyonu.
  - Kurulum, kaldırma ve güncelleme eylemlerinin otomatik denetim günlüğü kaydı (`opk audit`, `opk audit list`, `opk audit show <id>`, `opk audit export`).
  - Politika risk puanlama motoru (0-100 puan, `Low`, `Medium`, `High`, `Critical`) ve `TrustLevel` (`Verified`, `Signed`, `Unsigned`) değerlendirmesi.
  - Etkileşimli terminal onay ekranı (`[e/H]`) ve `--yes` (`-y`) otomatik onay parametresi.
  - `bekci-d` IPC Daemon Unix domain socket desteği (`opk daemon`).
- **Kapsamlı Dokümantasyon Paketi:**
  - `docs/` altında 16 adet teknik şartname ve geliştirici rehberi (`ARCHITECTURE.md`, `CLI_REFERENCE.md`, `INSTALL.md`, `SECURITY.md` vb.).

## v0.1.0 (2026-04-28)
### Eklendi
- Rust Cargo Workspace modüler mimarisi (`mcpkg-core`, `mcpkg-crypto`, `mcpkg-sandbox`, `mcpkg-pipeline`, `mcpkg-cli`).
- `manifest.toml` ve `sandbox.policy` TOML ayrıştırıcıları.
- Ed25519 kriptografik ilkel fonksiyonları (`sign`, `verify`, `keygen`).
- Linux namespace yönetimi ve AppArmor profil üretimi.
- Kurulum durum makinesi (`InstallState`) ve atomik geri alma (rollback).
- Temel komut satırı arayüzü (`install`, `launch`, `pack`, `sign`, `verify`, `publish`).
