# Orion İşletim Sistemi (Orion OS) - Sürüm Raporu

Bu sürüm raporu, Orion İşletim Sistemi projesinin **%100 Pure-Rust Sürümü** (`v1.0.0`) detaylarını, modülleri ve doğrulama süreçlerini içermektedir.

---

## 🏷️ Sürüm Bilgileri
* **Sürüm Numarası:** `v1.0.0`
* **Dil & Ekosistem:** %100 Pure Rust (`crt-static`)
* **Desteklenen Hedef Mimari:** `aarch64` (ARM64 QEMU / PinePhone)
* **Kernel:** Linux ARM64 — Debian netboot precompiled (`out/kernel/Image`, 36 MB)
* **Emülatör:** QEMU `virt` makinesi, `cortex-a72` CPU, 4 çekirdek, 2 GB RAM

---

## 🏗️ Mimari Katmanlar ve Bileşenler

### 1. Önyükleme ve Temel Sistem (Boot & Base)
* **Init Modülü (`core/init`)**: Rust diliyle geliştirilen PID 1 süreci (`crt-static`); sanal dosya sistemlerini mount eder, kernel modüllerini yükler (`e1000.ko`, `virtio_blk.ko`, `ext4.ko`), `init.rc` ayrıştırmasını ve servis süpervizyonunu gerçekleştirir.

### 2. IPC ve Sistem Servisleri (The Nervous System)
* **İletişim Altyapısı (`libipc-rs`)**: Rust dilinde yazılmış Binder/Parcel wire-compatible Unix domain socket IPC mekanizması.
* **Hizmet Yöneticisi (`servicemanager`)**: Servislerin ad ve socket yollarını kaydettiği merkezi rehber.
* **Güç Yöneticisi (`powermanager`)**: Rust ile geliştirilen pil seviyesi ve güç profili yönetim daemon'ı.
* **Giriş Hizmeti (`inputflinger`)**: Rust tabanlı girdi enjeksiyon servisi.
* **API Gateway (`apigateway`)**: REST HTTP -> IPC dönüşümü sağlayan Rust HTTP sunucusu.
* **Ekran Kompozitörü (`surfaceflinger`)**: Çok katmanlı yazılım tabanlı grafik kompozitörü.

### 3. Arayüz ve Uygulama Shell (Pure Rust Apps)
* **Statusbar (`statusbar`)**: Rust diliyle yazılmış üst durum çubuğu ve bildirim paneli.
* **Launcher (`launcher`)**: Rust diliyle yazılmış dokunmatik uyumlu ana ekran grid ve dock uygulaması.
* **Ayarlar (`settings`)**: Rust diliyle yazılmış sistem ayarları ve güç profili kontrol uygulaması.
* **Arama (`dialer`)**: Rust diliyle yazılmış telefon tuş takımı uygulaması.
* **Paket Yöneticisi (`opk`)**: Rust diliyle yazılmış Ed25519 imzalı paket yöneticisi.

---

## 🧪 Entegrasyon ve Doğrulama Testleri

- `test_native_apps_host.sh` entegrasyon testi ile tüm servislerin ve yerel uygulamaların IPC iletişimi doğrulandı.
- `make clean && ./scripts/build.sh` ile tüm sistem paketlerinin ARM64 çapraz derlemesi başarıyla tamamlandı.
- `./scripts/qemu-run.sh --headless` ile QEMU sanal makinesinde %100 Pure-Rust önyüklemesi doğrulandı.

---

*Copyright © 2026 mcsimon. Tüm Hakları Saklıdır.*
