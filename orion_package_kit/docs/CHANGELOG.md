# Değişiklik Günlüğü (Changelog) — Orion Package Kit (`opk`)

Tüm önemli değişiklikler bu dosyada belgelenir.  
Format [Keep a Changelog](https://keepachangelog.com/tr/1.0.0/) standardına uygundur ve [Semantic Versioning 2.0.0](https://semver.org/lang/tr/) kurallarını takip eder.

---

## [v0.2.0] — 2026-10-13

### Eklendi
- **Paket Yaşam Döngüsü Yönetimi (Milestone 6):**
  - `opk uninstall <app_id>`: Uygulama süreçlerini sonlandırma (`SIGTERM`), AppArmor profilini kaldırma, cgroups v2 temizliği, masaüstü kısayolu, ikon ve dosya temizliği desteği (`--purge` ile kullanıcı verilerini temizleme seçeneği).
  - `opk list`: Kurulu tüm paketleri tablo ve JSON (`--json`) olarak listeleme.
  - `opk info <id|opk>`: Paket metadata, deklaratif güvenlik izinleri (`sandbox.policy`) ve kurulum durumu sorgulama.
  - `opk update [id]`: Registry üzerinden yeni sürüm denetimi ve otomatik güncelleme indirme.
- **Güçlendirilmiş İzolasyon (Milestone 2):**
  - `libseccomp` bağımlılığı `mcpkg-sandbox` içinde varsayılan (default feature) haline getirildi.
  - `cgroups v2` kaynak sınırlandırma yöneticisi (`CgroupManager`): `cpu.max`, `memory.max`, `memory.swap.max` ve işlem atama/temizliği.
  - Özel `/tmp/orion-{id}` bağlama ve salt-okunur (read-only) kök bağlama kuralları.
- **Sistem & Kurulum İyileştirmeleri (Milestone 3):**
  - Kurulum hedefi `/tmp` yerine kalıcı `~/.local/share/orion/apps` (rootless) veya `/opt/orion/apps` (root) dizinine bağlandı.
  - `assets/icon.png` dosyasının `~/.local/share/icons/orion/` dizinine çıkarılması ve masaüstü başlatıcı entegrasyonu.
  - Host işlemci mimarisine göre çoklu mimari seçimi (`entry_x86_64` vs `entry_aarch64`).
  - `min_os`, `max_os` ve bağımlılık (`dependencies`) doğrulama mekanizması.
- **Store & Registry Entegrasyonu (Milestone 4):**
  - `opk search <query>` komutu ile paket arama (tablo ve JSON desteği).
  - Registry üzerinden geliştirici public key'lerini sorgulama desteği.
- **Orion Bekçi (Gatekeeper) & SQLite Audit Log (Milestone 7):**
  - `~/.local/share/orion/bekci/audit.db` SQLite veritabanı şeması ve `rusqlite` entegrasyonu.
  - Kurulum, kaldırma ve güncelleme eylemlerinin otomatik denetim günlüğü kaydı (`opk audit`, `opk audit list`, `opk audit show <id>`, `opk audit export`).
  - Politika risk puanlama motoru (0-100 puan, `Low`, `Medium`, `High`, `Critical`) ve `TrustLevel` (`Verified`, `Signed`, `Unsigned`) değerlendirmesi.
  - Etkileşimli terminal onay ekranı (`[e/H]`) ve `-y, --yes` otomatik onay parametresi.
  - `bekci-d` IPC Daemon Unix Domain Socket desteği (`opk daemon`).
- **Geliştirici Araçları (Milestone 5):**
  - `opk pack`: Proje klasöründen otomatik `checksums.sha256` ve `.opk` ZIP arşivi üretme.
  - `opk sign`: Özel Ed25519 anahtarı ile paketi imzalama ve `signature.ed25519` bloğunu güncelleme.
  - `opk verify`: Paketin hem SHA-256 sağlama toplamlarını, Bekçi risk puanını hem de dijital imzasını doğrulama.
  - `opk keygen`: Base64 formatında `private.key` ve `public.key` anahtar çifti üretme.
  - `opk publish`: İmzalı paketleri registry sunucusuna yükleme.
- **Kapsamlı Test Kapsamı (Milestone 8):**
  - 32 adet birim ve entegrasyon testi ile tüm bileşenlerin doğrulanması.

---

## [v0.1.0] — 2026-04-28

### Eklendi
- Rust Cargo Workspace modüler mimarisi (`mcpkg-core`, `mcpkg-crypto`, `mcpkg-sandbox`, `mcpkg-pipeline`, `mcpkg-cli`).
- `manifest.toml` ve `sandbox.policy` TOML ayrıştırıcıları.
- Ed25519 imzalama ve doğrulama ilkel fonksiyonları (`CryptoManager`).
- Linux mount, user ve network namespace temel entegrasyonu (`NamespaceManager`).
- AppArmor profil şablonu oluşturucu ve `apparmor_parser` çağrısı.
- Temel kurulum durum makinesi (`InstallState`) ve ZIP çıkarma altyapısı.
- `opk` komut satırı arayüzü temeli (`install`, `launch`, `verify`).
