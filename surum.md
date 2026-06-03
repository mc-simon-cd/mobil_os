# Mobil İşletim Sistemi (Mobile OS) - Sürüm Raporu

Bu sürüm raporu, Mobil İşletim Sistemi projesinin ilk alfa sürümüne (`v1.0.0-alpha`) ait detayları, geliştirilen modülleri, mimari bileşenleri ve doğrulama süreçlerini içermektedir.

---

## 🏷️ Sürüm Bilgileri
* **Sürüm Numarası:** `v1.0.0-alpha`
* **Yayınlanma Tarihi:** 3 Haziran 2026
* **Desteklenen Hedef Mimari:** `aarch64` (ARM64 QEMU)
* **Desteklenen Simülasyon Mimarisi:** `x86_64` (Local Linux Host)

---

## 🏗️ Mimari Katmanlar ve Bileşenler

### 1. Önyükleme ve Temel Sistem (Boot & Base)
* **Init Modülü (`core/init`)**: C diliyle sıfırdan geliştirilen PID 1 `init` süreci, `/proc`, `/sys`, `/dev` ve `/tmp` dosya sistemlerini otomatik olarak bağlar ve `etc/inittab` dosyasını okuyarak sistem daemon'larını başlatır.
* **Kök Dosya Sistemi (Rootfs Overlay)**: Güvenli kullanıcı yapılandırmaları (`etc/passwd`, `etc/group`), `etc/fstab`, önyükleme betikleri (`etc/init.d/rcS`) ve yerel dil yapılandırmalarını içerir.

### 2. IPC ve Sistem Servisleri (The Nervous System)
* **İletişim Altyapısı (`libipc` / `libipc-rs`)**: C ve Rust dillerinde veri serileştirme/paketleme (`Parcel` ve `Binder` modelleri) ile Unix domain socketleri üzerinden çalışan yüksek performanslı ve güvenli IPC mekanizması.
* **Hizmet Yöneticisi (`servicemanager`)**: Servislerin sistem genelinde kayıt altına alınmasını, sorgulanmasını ve ad çözümlemesini (ad çözümleme Unix soketi üzerinden) yöneten ana kayıt defteri.
* **Güç Yöneticisi (`powermanager`)**: Rust ile geliştirilen, pil seviyesini izleyen, güç profillerini (balanced/performance/powersave) yöneten ve durum değişikliklerini statusbar'a bildiren sistem daemon'ı.
* **Giriş Hizmeti (`inputflinger`)**: Rust tabanlı girdi olayları dinleme ve API Gateway üzerinden gelen Key/Touch girdilerini sisteme enjekte etme servisi.
* **Uygulama Çalıştırıcı (`apprunner`)**: JSON manifest dosyaları üzerinden farklı dillerle (Native C, Python, JavaScript, Web) yazılmış uygulamaları izole ve dinamik bir biçimde çalıştıran birleşik Rust uygulaması.
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
