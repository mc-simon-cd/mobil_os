# Orion İşletim Sistemi (Orion OS) - Sürüm Raporu

Bu sürüm raporu, Orion İşletim Sistemi projesinin ilk alfa sürümüne (`v1.0.0-alpha`) ait detayları, geliştirilen modülleri, mimari bileşenleri ve doğrulama süreçlerini içermektedir.

---

## 🏷️ Sürüm Bilgileri
* **Sürüm Numarası:** `v1.0.0-alpha`
* **Yayınlanma Tarihi:** 6 Haziran 2026
* **Desteklenen Hedef Mimari:** `aarch64` (ARM64 QEMU)
* **Desteklenen Simülasyon Mimarisi:** `x86_64` (Local Linux Host)
* **Kernel:** Linux ARM64 — Debian netboot precompiled (`out/kernel/Image`, 36 MB)
* **Emülatör:** QEMU `virt` makinesi, `cortex-a72` CPU, 4 çekirdek, 2 GB RAM

---

## 🏗️ Mimari Katmanlar ve Bileşenler

### 1. Önyükleme ve Temel Sistem (Boot & Base)
* **Init Modülü (`core/init`)**: C diliyle geliştirilen PID 1 süreci; `/proc`, `/sys`, `/dev`, `/tmp` mount, `e1000.ko` modül yükleme, ağ yapılandırması (`ioctl`), `init.rc` ayrıştırma (`class core/main`, `args`, `respawn`) ve servis süpervizyonu.
* **Boot Manifest (`init.rc`)**: `servicemanager` → `inputflinger` (core), ardından `powermanager`, `apigateway`, `surfaceflinger`, `statusbar`, `apprunner`, `launcher` (main) sırasıyla başlatılır.
* **Initramfs Boot**: `scripts/make-rootfs.sh` ile `out/initramfs.cpio.gz` oluşturulur; disk sürücüsü gerektirmez.
* **Bağımlılık Otomasyonu (`deps/deps.yml`)**: apt/pacman paketleri, Rust workspace güncellemeleri, kernel/modül indirme ve derleme pipeline'ı `scripts/update-deps.sh` ile otomatikleştirildi.

### 2. IPC ve Sistem Servisleri (The Nervous System)
* **İletişim Altyapısı (`libipc` / `libipc-rs`)**: C ve Rust dillerinde veri serileştirme/paketleme (`Parcel` ve `Binder` modelleri) ile Unix domain socketleri üzerinden çalışan yüksek performanslı ve güvenli IPC mekanizması.
* **Hizmet Yöneticisi (`servicemanager`)**: Servislerin sistem genelinde kayıt altına alınmasını, sorgulanmasını ve ad çözümlemesini (ad çözümleme Unix soketi üzerinden) yöneten ana kayıt defteri.
* **Güç Yöneticisi (`powermanager`)**: Rust ile geliştirilen, pil seviyesini izleyen, güç profillerini (balanced/performance/powersave) yöneten ve durum değişikliklerini statusbar'a bildiren sistem daemon'ı.
* **Giriş Hizmeti (`inputflinger`)**: Rust tabanlı girdi olayları dinleme ve API Gateway üzerinden gelen Key/Touch girdilerini sisteme enjekte etme servisi.
* **Uygulama Çalıştırıcı (`apprunner`)**: JSON manifest tabanlı birleşik Rust uygulaması. Tek başına daemon değildir; `apprunner <app_dir>` şeklinde çağrılır. QEMU boot'ta `init.rc` üzerinden Control Center web sunucusu otomatik başlatılır.
* **API Gateway (`apigateway`)**: Android/iOS cihazlarla entegrasyon sağlayan, giriş olayları enjeksiyonu ve güç profili kontrolü sunan Rust HTTP REST sunucusu.

### 3. Grafik ve Arayüz Shell (Graphics & Compositing)
* **Grafik Kütüphanesi (`libgraphics`)**: C ile yazılmış; çizgi, dikdörtgen, çember, karakter çizimleri ve yazı tipi işlemeyi bellek tamponuna (RGB24 canvas) işleyen temel çizim motoru.
* **Ekran Kompozitörü (`surfaceflinger`)**: İstemci uygulamalardan gelen tampon PPM verilerini (`out/surface_<id>.ppm`) yükseklik ve y-başlangıç offsetlerine göre arka plandan öne doğru üst üste bindirerek tek bir birleşik ekran görüntüsü (`out/display_composited.ppm`) üreten C++ grafik motoru.
* **Durum Çubuğu Daemon'ı (`statusbar`)**: `powermanager` servisinden pil ve profil bilgilerini alıp ekrana yazan, ayrıca `mobile.statusbar` Unix soketini dinleyerek sisteme enjekte edilen bildirim yazılarını anlık olarak üst panelde gösteren C daemon'ı.
* **Launcher (`launcher`)**: 2D grafik motoru ile çizilmiş, dokunmatik uyumlu ızgara uygulama listesi ve alt bar dock içeren masaüstü kabuk uygulaması.
* **Sistem Uygulamaları (Settings & Dialer)**: 
  - **Settings (Ayarlar)**: Dil desteğine göre yerelleştirilmiş arayüz sunan, pil göstergesini grafik bar olarak çizen ve güç profilini değiştirebilen C uygulaması.
  - **Dialer (Telefon Arayüzü)**: Tuş takımı arayüzünü çizip arama işlemlerini tetikleyen ve statusbar'a bildirim yollayan C uygulaması.
  - **Kontrol Merkezi (Web App)**: Glassmorphism tasarım diliyle yazılmış, API Gateway üzerinden girdi yollayabilen ve sistemi uzaktan izleyebilen gerçek zamanlı web uygulaması.

---

## 🌍 Dil ve Yerelleştirme Desteği (`libi18n`)
Sistemdeki tüm C uygulamaları ve Python/JS betikleri dinamik çoklu dil desteğine sahiptir.
* Sistem dizini `/system/usr/share/locale` altındaki yerelleştirilmiş dil paketlerini yükler.
* Türkçe (`tr`) ve İngilizce (`en`) dilleri ana diller olarak modüler şekilde desteklenmektedir.

---

## 🧪 Entegrasyon ve Doğrulama Testleri
Sistem host mimarisinde (`x86_64`) tüm bileşenlerin iletişimini ve çalışmasını doğrulayan 5 adet otomatik entegrasyon testi içermektedir:
1. **`test_api_host.sh`**: API Gateway HTTP uç noktalarını, CORS protokolünü ve girdi/güç işlemlerini test eder.
2. **`test_apprunner_host.sh`**: Python (`reporter.py`), JS (`monitor.js`) ve Web Control Center uygulamalarının `apprunner` altındaki çalışma durumunu ve dil dosyası çevirilerini doğrular.
3. **`test_statusbar_host.sh`**: Statusbar daemon'ının bildirim soketine enjekte edilen mesajları anlık alıp ekranda render ettiğini test eder.
4. **`test_native_apps_host.sh`**: Native Settings ve Dialer uygulamalarının bağımsız arayüz tamponları ürettiğini ve IPC üzerinden durum çubuğuna sinyal gönderdiğini doğrular.
5. **`test_compositor_host.sh`**: `surfaceflinger` kompozitörünün istemci PPM tamponlarını doğru koordinat ve katman sırasında üst üste bindirerek nihai ekranı oluşturduğunu doğrular.

---

## 🖥️ QEMU Emülasyon ve Kernel

### Kernel Edinimi
* **Kaynak:** Debian netboot ARM64 precompiled kernel
* **Script:** `scripts/download-kernel.sh`
* **Çıktılar:** `out/kernel/Image` (~37 MB) + `out/e1000.ko` (Debian nic-modules paketinden otomatik çıkarılır)

### QEMU Boot (Initramfs)
* ARM64 Linux kernel, initramfs ile QEMU `virt` makinesinde başarıyla boot edildi.
* PID 1 init tüm core/main servislerini ayağa kaldırır; ağ `10.0.2.15` olarak yapılandırılır.
* Host'tan `curl http://localhost:9595/api/status` ile API Gateway doğrulanabilir.

### Port Konfigürasyonu
| Host Port | Misafir Port | Protokol | Açıklama |
|-----------|-------------|----------|---------|
| `9595` | `8080` | TCP/HTTP | API Gateway REST |
| — | `8000` | TCP/HTTP | Control Center (apprunner, guest içi) |

### Çalıştırma
```bash
# Bağımlılıkları kur ve doğrula
./scripts/update-deps.sh --all

# veya adım adım:
./scripts/download-kernel.sh
./scripts/make-rootfs.sh
./scripts/qemu-run.sh --headless
```

---

## 🔜 Sonraki Adımlar
* [ ] Launcher'ın `out/surface_<id>.ppm` dosyası üretmesi (compositor uyarılarının giderilmesi)
* [x] Rootfs ext4 disk image'ı oluştur (`out/rootfs.ext4`)
* [x] ext4 tabanlı tam disk boot (`./scripts/qemu-run.sh --disk`)
