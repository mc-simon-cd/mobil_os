# Orion OS (mcsimon) - Kullanım ve Geliştirici Kılavuzu 📱

Bu kılavuz, **mcsimon** özel mobil işletim sisteminin mimarisini, derleme süreçlerini, QEMU emülatörü üzerinde çalıştırılmasını, sistem servislerini, Inter-Process Communication (IPC) yapısını, ağ yapılandırmasını, çoklu dil kütüphanesini (`libi18n`) ve test doğrulama aşamalarını detaylandırmaktadır.

---

## 🏛️ 1. Sistem Mimarisi ve Katmanları

Orion OS, sıfırdan geliştirilen düşük seviyeli kullanıcı alanı (user-space) servisleri ile minimal bir Linux çekirdeği (kernel) üzerinde çalışan modüler bir işletim sistemidir.

```
┌────────────────────────────────────────────────────────┐
│                   Sistem Uygulamaları                  │
│     (launcher, settings, dialer, messaging, browser)   │
└───────────────────────────┬────────────────────────────┘
                            │ (libui / libgraphics)
┌────────────────────────────────────────────────────────┐
│                    Mobil Shell UI                      │
│        (statusbar, quicksettings, notifications)       │
└───────────────────────────┬────────────────────────────┘
                            │ (libipc)
┌────────────────────────────────────────────────────────┐
│                    Sistem Servisleri                     │
│   (surfaceflinger, inputflinger, powermanager, etc.)   │
└───────────────────────────┬────────────────────────────┘
                            │ (Unix Domain Sockets)
┌────────────────────────────────────────────────────────┐
│                servicemanager (Registry)               │
└───────────────────────────┬────────────────────────────┘
                            │ (Özel init.rc Yapılandırması)
┌────────────────────────────────────────────────────────┐
│                    core/init (PID 1)                   │
└───────────────────────────┬────────────────────────────┘
                            │ (VFS Mounts / ioctl / modules)
┌────────────────────────────────────────────────────────┐
│                      Linux Kernel                      │
└───────────────────────────┬────────────────────────────┘
                            │ (ARM64 / QEMU Sanal Donanımı)
┌────────────────────────────────────────────────────────┐
│                      Sanal Cihaz                       │
└────────────────────────────────────────────────────────┘
```

### Temel Bileşenler
1. **`core/init` (PID 1)**: C diliyle yazılmış önyükleme yöneticisi. VFS mount, `e1000.ko` yükleme, ağ yapılandırması, `init.rc` ayrıştırma (`class core/main`, `args`, `respawn`) ve servis süpervizyonu yapar.
2. **`servicemanager` (IPC Kayıt Defteri)**: İletişim kurmak isteyen tüm servislerin uç noktalarını (Unix Domain Socket yolları) kaydettiği ve sorguladığı merkezi rehber sistemidir.
3. **`powermanager`**: Rust diliyle yazılmış, pil durumunu ve güç profillerini (balanced, performance, powersave) takip eden sistem servisidir.
4. **`inputflinger`**: Rust tabanlı girdi enjeksiyon servisidir.
5. **`apigateway`**: Dış dünya (örneğin ana makine) ile HTTP REST üzerinden haberleşmeyi sağlayan, gelen istekleri IPC üzerinden sistem servislerine ileten Rust tabanlı sunucudur.
6. **`surfaceflinger`**: C++ tabanlı kompozitör (ekran yöneticisi) daemon'ıdır. Uygulamaların çizdiği PPM pencerelerini koordinatlarına göre birleştirip nihai ekranı oluşturur.
7. **`statusbar`**: Pil durumunu, aktif güç profilini ve sisteme gönderilen bildirim mesajlarını üst panelde çizen C tabanlı görsel servistir.

---

## 🛠️ 2. Geliştirme Ortamı ve Bağımlılıklar

Tüm bağımlılıklar merkezi manifest dosyasında tanımlıdır: [`deps/deps.yml`](deps/deps.yml)

### Otomatik Kurulum (Önerilen)

```bash
# Ortamı kontrol et
./scripts/update-deps.sh --check

# Eksik paketleri kur (Arch: pacman, Debian/Ubuntu: apt)
./scripts/update-deps.sh --install

# Tam pipeline: kurulum + cargo update + kernel + derleme + initramfs
./scripts/update-deps.sh --all

# veya Makefile üzerinden:
make check-deps
make update-deps
```

### Manuel Kurulum (Debian/Ubuntu)
```bash
sudo apt update
sudo apt install build-essential rustc cargo make python3 nodejs curl qemu-system-arm -y
```

### ARM64 Çapraz Derleme Toolchain
QEMU ARM64 sanal makinesinde çalıştırmak üzere çapraz derleme yapabilmek için:
```bash
sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu -y
```

Rust için ARM64 hedefini ekleyin:
```bash
rustup target add aarch64-unknown-linux-gnu
```

---

## 🏗️ 3. Derleme Sistemi (Build System)

Orion OS hem yerel makinede (x86_64 üzerinde hızlı test için) hem de hedef ARM64 mimarisi için derlenebilir.

### 3.1. Yerel Makine (Host) Derlemesi
Tüm servisleri ve testleri hızlıca yerel makinede denemek için:
```bash
make clean
make CC=gcc CXX=g++
./scripts/build.sh
```

### 3.2. ARM64 Target Derlemesi
QEMU emulatoründe çalışacak statik binary'leri çapraz derlemek için:
```bash
make clean
CC=aarch64-linux-gnu-gcc CXX=aarch64-linux-gnu-g++ ./scripts/build.sh
```

> [!IMPORTANT]
> QEMU içinde çalışan minimal `initramfs` ortamında dinamik kütüphaneler (`glibc` / `ld.so`) bulunmadığından, derleme sistemi tüm C/C++ ve Rust projelerini **statically linked** (-static) olarak derler.
> - C/C++ için: Link aşamasında `-static` bayrağı ile derlenir.
> - Rust için: `RUSTFLAGS="-C target-feature=+crt-static"` kullanılarak statik glibc derlemesi sağlanır.

Derleme sonrasında üretilen tüm binary'ler ve kök dizin ağacı `out/rootfs/` içerisinde toplanır.

---

## 📦 4. Rootfs ve Initramfs İmajının Hazırlanması

QEMU'nun disk sürücüsü gerektirmeden belleğe yükleyip doğrudan çalıştırabileceği sıkıştırılmış `initramfs.cpio.gz` dosyasını oluşturmak için:

```bash
./scripts/make-rootfs.sh
```

Bu script şunları yapar:
1. `out/rootfs_stage` adında geçici bir staging dizini oluşturur.
2. `core/init` kodunu statik olarak derleyip `/sbin/init` ve kök dizindeki `/init` konumuna kopyalar.
3. Derlenen sistem servislerini (`servicemanager`, `powermanager`, `apigateway`, `inputflinger`, `apprunner`, `statusbar`, `surfaceflinger`, `launcher`) `/system/bin/` dizinine taşır.
4. `/system/apps` (control_center, sys_reporter, sys_monitor) ve locale dosyalarını kopyalar.
5. Ağ sürücüsü `e1000.ko` dosyasını `/lib/modules/` dizinine yerleştirir (`out/e1000.ko` — `download-kernel.sh` ile üretilir).
6. Compositor çıktısı için `out/` dizinini oluşturur.
7. `/etc/init.rc`, `/etc/passwd`, `/etc/group` ve `/etc/fstab` yapılandırmalarını yazar.
8. Tüm yapıyı `cpio -H newc` formatında paketleyip `out/initramfs.cpio.gz` olarak sıkıştırır.

---

## 💻 5. QEMU Emülasyonu ve Önyükleme (Boot)

Orion OS'i sanal makinede çalıştırmak için sırasıyla aşağıdaki adımları uygulayın.

### 5.1. Çekirdek ve Ağ Modülü İndirme
QEMU ARM64 kernel görüntüsü ve eşleşen `e1000` ağ sürücüsünü indirin:
```bash
./scripts/download-kernel.sh
```
Bu script şunları üretir:
- `out/kernel/Image` (~37 MB, Debian netboot ARM64)
- `out/e1000.ko` (kernel sürümüne uygun nic-modules paketinden otomatik çıkarılır)

### 5.2. QEMU Çalıştırma Seçenekleri
Sistem iki farklı modda çalıştırılabilir:

* **Grafik Arayüz Modu (Varsayılan)**:
  Eğer X11/Wayland ekran kartı desteği olan bir masaüstü ortamındaysanız:
  ```bash
  ./scripts/qemu-run.sh
  ```
* **Konsol / Headless Mod (Önerilen)**:
  Terminal üzerinden doğrudan logları izlemek ve etkileşime geçmek için:
  ```bash
  ./scripts/qemu-run.sh --headless
  ```

> [!TIP]
> QEMU headless moddan çıkmak için terminalinizde **`Ctrl + A`** tuş kombinasyonuna bastıktan sonra **`X`** tuşuna basmanız yeterlidir.

### 5.3. Boot Sırası (`init.rc`)

`core/init/init.rc` servisleri `class` önceliğine göre başlatır:

| Sınıf | Servisler |
|:---|:---|
| **core** | `servicemanager`, `inputflinger` |
| **main** | `powermanager`, `apigateway`, `surfaceflinger`, `statusbar`, `apprunner`, `launcher` |

Desteklenen `init.rc` direktifleri: `on boot`, `service`, `class`, `args`, `respawn`, `mkdir`.

---

## 🔌 6. Programatik Ağ Yapılandırması ve Kernel Modülü Yükleme

Minimal işletim sistemi çekirdeğinde varsayılan olarak ağ sürücüleri gömülü gelmediği için `core/init` (PID 1) süreci boot aşamasında dinamik sürücü yüklemesi ve IP atama işlemlerini programatik olarak yönetir.

### Ağ Kurulum Aşamaları:
1. **Loopback Arayüzü**: `lo` arayüzü ayağa kaldırılır ve `127.0.0.1` IP'si atanır.
2. **Sürücü Yükleme**: `/lib/modules/e1000.ko` (Intel PRO/1000 ağ kartı sürücüsü) kernel içerisine `init_module` sistem çağrısı ile dinamik olarak yüklenir.
3. **Aygıt Algılama**: Sistem 500 ms bekleyerek ağ kartının işletim sistemi tarafından tanınmasını sağlar.
4. **IP Adres Atama**: `/sys/class/net` taranarak algılanan ethernet arayüzü (örneğin `eth0`) `ioctl` yardımıyla ayağa kaldırılır (UP & RUNNING) ve QEMU varsayılan IP adresi olan `10.0.2.15` atanır.

### Port Yönlendirme (Port Forwarding):
QEMU başlatılırken host işletim sistemi ile sanal makine arasında tünel kurulur:
- **Host Port:** `9595`  ➡️  **Guest Port:** `8080` (API Gateway HTTP Portu)

---

## 🌐 7. API Gateway Uç Noktaları (REST API)

Dış dünyadan veya uygulamalardan HTTP protokolü üzerinden işletim sistemi servislerini kontrol etmek ve durum bilgisi almak için kullanılan uç noktalar aşağıda listelenmiştir.

Host makineden istek göndermek için `http://localhost:9595` adresini kullanabilirsiniz:

### 1. Sistem Durumu
* **İstek:** `GET /api/status`
* **Yanıt:**
  ```json
  {"status":"ok","service":"apigateway","version":"1.0"}
  ```

### 2. Güç Durumu Sorgulama
* **İstek:** `GET /api/power`
* **Yanıt:**
  ```json
  {"battery_level":85,"power_mode":"balanced"}
  ```

### 3. Güç Profilini Değiştirme
* **İstek:** `POST /api/power/mode?mode=<balanced|performance|powersave>`
* **Örnek:** `curl -X POST "http://localhost:9595/api/power/mode?mode=performance"`
* **Yanıt:**
  ```json
  {"status":"success","power_mode":"performance"}
  ```

### 4. Girdi Enjekte Etme (Key / Touch Events)
* **İstek:** `POST /api/input/inject?type=<type>&code=<code>&value=<value>`
* **Örnek (Güç Tuşuna Basma):** `curl -X POST "http://localhost:9595/api/input/inject?type=1&code=116&value=1"`
* **Yanıt:**
  ```json
  {"status":"success","type":1,"code":116,"value":1}
  ```

### 5. Son Girdi Olayını Sorgulama
* **İstek:** `GET /api/input/last`
* **Yanıt:**
  ```json
  {"status":"success","type":1,"code":116,"value":1}
  ```

### 6. Ekran Kompozit Tetikleme
* **İstek:** `POST /api/graphics/composite`
* **Açıklama:** `surfaceflinger` üzerinde PPM katmanlarını birleştirir → `out/display_composited.ppm`
* **Yanıt:** `{"status":"success","composited":true}`

### 7. Canlı Ekran Önizlemesi (Control Center sync)
* **İstek:** `GET /api/display/info`
* **Yanıt:**
  ```json
  {"status":"ok","available":true,"width":1080,"height":2400,"format":"ppm","size":7776015}
  ```
* **İstek:** `GET /api/display/frame?composite=1`
* **Açıklama:** `composite=1` önce kompozit çalıştırır, ardından `display_composited.ppm` ikili gövde olarak döner (`Content-Type: image/x-portable-pixmap`). Control Center mock cihaz canvas’ı bu endpoint’i ~2.5 sn aralıkla poll eder; canvas tıklaması touch inject (`EV_ABS` + `BTN_TOUCH`) gönderir.
* **Örnek:**
  ```bash
  curl -s http://localhost:9595/api/display/info
  curl -s http://localhost:9595/api/display/frame?composite=1 -o screen.ppm
  ```

> **Not:** Bu API PPM yazılım kompozitörü içindir. Wayland geçişi sonrası aynı endpoint screencopy ile beslenecek; ayrıntılar: [`docs/wayland-migration.md`](docs/wayland-migration.md).

---

## 🔄 8. Inter-Process Communication (IPC) Geliştirici Kılavuzu

Orion OS içindeki tüm uygulamalar ve daemonlar Unix Domain Socket tabanlı, Android'in Binder ve Parcel modellerine benzeyen yüksek performanslı bir IPC mekanizması kullanır. Hem C hem de Rust için kütüphaneler mevcuttur.

### 8.1. C Geliştirici API (`libipc`)
Veri paketlemek için `parcel_t` yapısı kullanılır.

#### Yazma (Serialization) Örneği:
```c
#include <ipc/parcel.h>
#include <ipc/binder.h>

parcel_t p;
parcel_init(&p);

parcel_write_int32(&p, 42);
parcel_write_string(&p, "Merhaba Orion OS");
```

#### Bağlantı Kurma ve Gönderme:
```c
int client_fd = ipc_connect("/tmp/powermanager.sock");
if (client_fd >= 0) {
    parcel_t reply;
    parcel_init(&reply);
    
    // 101 transaction kodu ile veriyi gönder
    int ret = ipc_send_transaction(client_fd, 101, &p, &reply);
    if (ret == IPC_SUCCESS) {
        int32_t result;
        parcel_read_int32(&reply, &result);
        printf("Yanıt alındı: %d\n", result);
    }
    parcel_free(&reply);
    close(client_fd);
}
parcel_free(&p);
```

### 8.2. Rust Geliştirici API (`libipc-rs`)
Rust ile yazılan servislerde serialization tamamen güvenli veri sarmalayıcıları ile gerçekleştirilir. `powermanager` ve `apigateway` bu kütüphaneyi kullanır.

---

## 🌍 9. Çoklu Dil Desteği (`libi18n`)

Sistemde bulunan tüm pencereler, C uygulamaları ve Python/JS scriptleri dinamik dil paketleri üzerinden yerelleştirilmiştir.

### Dil Dosyaları Konumu
Dil dosyaları kök dizinde `/system/usr/share/locale/` altında yer alır:
- `tr.txt` (Türkçe)
- `en.txt` (İngilizce)
- `de.txt` (Almanca)
- vb.

### Dosya Formatı (`key=value`):
```properties
# Türkçe Locale (tr.txt)
launcher.title=Uygulama Başlatıcı
settings.title=Ayarlar
power.battery=Pil
power.mode=Güç Modu
power.mode.performance=Yüksek Performans
```

### C Kodunda Kullanım:
```c
#include <i18n.h>

// Dil paketini yükle
i18n_init("/system/usr/share/locale", "tr");

// Çeviriyi al
const char* app_title = i18n_get("launcher.title");
printf("Başlık: %s\n", app_title); // Çıktı: Uygulama Başlatıcı

// Belleği serbest bırak
i18n_free();
```

---

## 📱 10. Çoklu Dil Uygulama Runtimeları ve Kontrol Paneli

Sistem, manifest tabanlı `apprunner` modülüne sahiptir. **Önemli:** `apprunner` tek başına bir daemon değildir; bir uygulama dizini argüman olarak verilmelidir:

```bash
./system/bin/apprunner /system/apps/control_center
```

QEMU boot sırasında `init.rc` Control Center'ı otomatik başlatır:

```
service apprunner /system/bin/apprunner
    args /system/apps/control_center
    respawn
```

### 10.1. Python Reporter (`sys_reporter`)
Pil verilerini okuyarak lokalize edilmiş bir biçimde raporlayan Python uygulaması:
```bash
API_PORT=8085 LANG=tr ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/sys_reporter
```

### 10.2. JavaScript Monitor (`sys_monitor`)
Node.js üzerinde API durumlarını belirli aralıklarla sorgulayan izleme uygulaması:
```bash
API_PORT=8085 ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/sys_monitor
```

### 10.3. Web Kontrol Merkezi Dashboard (`control_center`)
Premium Glassmorphism tasarıma sahip, gerçek zamanlı durum izleme, güç modu değiştirme ve klavye girdisi enjekte etme özelliklerine sahip web kontrol paneli:
```bash
APP_PORT=8090 ./out/rootfs/system/bin/apprunner ./out/rootfs/system/apps/control_center
```
> Host tarayıcınızdan `http://localhost:8090` adresini açarak kontrol merkezine erişebilirsiniz.

---

## 🧪 11. Test ve Doğrulama Süreçleri

Sistem kalitesini garanti altına almak için `tests/` dizininde hem birim (unit) hem de uçtan uca (integration) testler yer almaktadır.

### Birim Testlerini Çalıştırma (Unit Tests)
* **IPC Serileştirme Testi**:
  ```bash
  gcc -Wall -Wextra -std=c99 -Ilibs/libipc/include tests/unit/test_ipc.c out/rootfs/system/lib/libipc.a -o test_ipc && ./test_ipc
  ```
* **Dil Kütüphanesi Testi**:
  ```bash
  gcc -Wall -Wextra -std=c99 -Ilibs/libi18n/include tests/unit/test_i18n.c out/rootfs/system/lib/libi18n.a -o test_i18n && ./test_i18n
  ```

### Entegrasyon Testlerini Çalıştırma (Integration Tests)
Aşağıdaki betikler sistem servislerini arka planda ayağa kaldırıp senaryoları simüle eder:
```bash
# REST API ve CORS testleri
./tests/integration/test_api_host.sh

# Python/JS/Web uygulama çalışma testleri
./tests/integration/test_apprunner_host.sh

# Durum çubuğu ve bildirim mekanizması testleri
./tests/integration/test_statusbar_host.sh
```

---

## 🔍 12. Sorun Giderme ve İpuçları (Troubleshooting)

### 1. `Exec format error` Hatası
* **Neden**: QEMU sanal makinesi içerisinde x86_64 için derlenmiş bir binary çalıştırılmaya çalışılıyor veya binary dinamik linklenmiş ancak sanal makinede dynamic linker bulunamıyor.
* **Çözüm**: Derleme yaparken mutlaka `aarch64-linux-gnu-gcc` ile çapraz derleme yapın ve static linklemeyi (`-static`) zorunlu tutun. `file out/rootfs/system/bin/apigateway` komutu ile mimarinin `ARM aarch64` ve `statically linked` olduğunu doğrulayın.

### 2. Port Çakışması (`Port already in use`)
* **Neden**: Host üzerindeki `9595` veya `8085` portları başka bir QEMU veya web servisi tarafından kullanılıyor.
* **Çözüm**: Arka planda çalışan eski servisleri temizleyin:
  ```bash
  killall qemu-system-aarch64
  killall apigateway powermanager servicemanager statusbar surfaceflinger
  rm -f /tmp/*.sock
  ```

### 3. Ekran Görüntüsü Alamama (`surfaceflinger` PPM Oluşturmuyor)
* **Neden**: Uygulama surface ayırmış ancak `out/surface_<id>.ppm` dosyasını yazmamış olabilir (launcher bilinen bir eksiklik). Veya eski/stale surface kayıtları birikmiş olabilir.
* **Çözüm**: `out/` dizininin initramfs'te mevcut olduğundan emin olun (`make-rootfs.sh` bunu oluşturur). Statusbar surface 1 için PPM üretir; launcher henüz PPM yazmıyorsa compositor uyarıları normaldir.

### 4. `Usage: apprunner <app_directory_path>` Hatası
* **Neden**: `apprunner` argümansız başlatılmış (daemon olarak değil, uygulama çalıştırıcı olarak tasarlanmıştır).
* **Çözüm**: `init.rc` içinde `args /system/apps/control_center` satırının olduğundan emin olun ve initramfs'i yeniden oluşturun.

### 5. Kernel İndirme Zaman Aşımı
* **Neden**: Debian FTP yavaş veya bağlantı kesildi; `out/kernel/Image` eksik kalabilir (~37 MB olmalı).
* **Çözüm**: `./scripts/download-kernel.sh` komutunu tekrar çalıştırın veya `./scripts/update-deps.sh --kernel` kullanın.

---

*Copyright © 2026 mcsimon. Tüm Hakları Saklıdır.*
