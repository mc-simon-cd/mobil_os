# Orion OS — Sistem Mimarisi

> Versiyon: 0.5.x · Platform: ARM64 (QEMU virt) · Grafik: PPM / Wayland (çift mod)

---

## İçindekiler

1. [Genel Bakış](#genel-bakış)
2. [Katman Modeli](#katman-modeli)
3. [Önyükleme Akışı](#önyükleme-akışı)
4. [Çekirdek Bileşenler](#çekirdek-bileşenler)
5. [Servisler Katmanı](#servisler-katmanı)
6. [Uygulama Katmanı](#uygulama-katmanı)
7. [Grafik Alt Sistemi](#grafik-alt-sistemi)
8. [Girdi Alt Sistemi](#girdi-alt-sistemi)
9. [Wayland Geçiş Mimarisi](#wayland-geçiş-mimarisi)
10. [Derleme ve Hedef Platform](#derleme-ve-hedef-platform)

---

## Genel Bakış

Orion OS, ARM64 donanım hedefini (ve QEMU `virt` emülasyonunu) desteklemek üzere tasarlanmış minimal bir mobil işletim sistemi çekirdeğidir. Android'in temel tasarım ilkelerinden (katmanlı servis mimarisi, IPC tabanlı iletişim, `init` süreci tabanlı önyükleme) esinlenerek sıfırdan yazılmıştır.

**Temel tasarım hedefleri:**

- **Minimal footprint** — Kernel + initramfs < 50 MB
- **Modüler servisler** — Her servis bağımsız bir ikili, Unix socket üzerinden IPC
- **Çift mod grafik** — PPM (legacy) ve Wayland (compositor) desteği aynı anda
- **Çapraz derleme** — Host (x86_64) geliştirme, hedef (aarch64) deployment

---

## Katman Modeli

```
┌─────────────────────────────────────────────────┐
│                 UYGULAMA KATMANI                 │
│   launcher │ dialer │ settings │ statusbar       │
├─────────────────────────────────────────────────┤
│               SERVİS KATMANI                     │
│  surfaceflinger │ inputflinger │ apigateway      │
│  servicemanager │ powermanager │ apprunner       │
├─────────────────────────────────────────────────┤
│               KÜTÜPHANELer                       │
│        libipc │ libgraphics │ libi18n            │
├─────────────────────────────────────────────────┤
│               ÇEKİRDEK (core/)                   │
│             init │ servicemanager                │
├─────────────────────────────────────────────────┤
│            LINUX ÇEKİRDEĞİ (6.x ARM64)          │
└─────────────────────────────────────────────────┘
```

---

## Önyükleme Akışı

```
QEMU virt (ARM64)
    │
    ▼
Linux Kernel (Image.gz) ──── virtio-blk / virtio-net
    │
    ▼ rdinit=/init
core/init  ─────────────────────────────────────────┐
    │  1. mount_essential_filesystems()              │
    │     /proc /sys /dev /tmp                      │
    │  2. configure_network() (lo + eth0)           │
    │  3. property_init()                            │
    │  4. /proc/cmdline okuma                       │
    │     wayland=1 → WAYLAND_ENABLED=1             │
    │  5. parse_init_rc("/etc/init.rc")             │
    │  6. launch_services()                          │
    │                                                │
    ├──► servicemanager   (/tmp/servicemanager.sock) │
    ├──► surfaceflinger   (/tmp/surfaceflinger.sock) │
    ├──► inputflinger     (/tmp/inputflinger.sock)   │
    ├──► powermanager                                │
    ├──► apigateway       (/tmp/apigateway.sock)     │
    ├──► apprunner                                   │
    ├──► statusbar                                   │
    └──► launcher                                    │
                                                     │
         SIGCHLD handler: respawn ─────────────────-─┘
```

### init.rc Örnek Yapısı

```ini
on boot
    mkdir /tmp 0777

service servicemanager /system/bin/servicemanager
    class core
    user root
    respawn

service surfaceflinger /system/bin/surfaceflinger
    class core
    user root
    respawn

service launcher /system/bin/launcher
    class main
    user root
```

---

## Çekirdek Bileşenler

### `core/init`

| Dosya | Sorumluluk |
|---|---|
| `src/main.c` | PID 1 giriş noktası, filesystem mount, ağ, `/proc/cmdline` okuma |
| `src/init.c` | `init.rc` parser, servis kayıt ve başlatma motoru |
| `src/properties.c` | Anahtar-değer sistem property deposu (max 128 kayıt) |
| `src/mount.c` | `/proc`, `/sys`, `/dev`, `/tmp` sanal fs mount işlemleri |
| `init.rc` | Sistem boot konfigürasyon script'i |

**Servis Lifecycle:**

```
parse_init_rc() → service_t[] dizisi dolduruluyor
launch_services() → fork() + execve() her servis için
SIGCHLD → handle_sigchld() → respawn_service() (respawn=1 ise)
```

### `libs/libipc`

Tüm servisler arası iletişimi sağlayan senkron Unix socket kütüphanesi.

```
┌────────────┐    ┌─────────────┐
│  binder.h  │    │  parcel.h   │
│            │    │             │
│ ipc_connect│    │ parcel_init │
│ ipc_send   │    │ write_int32 │
│ _transaction│   │ write_string│
└────────────┘    │ read_int32  │
                  └─────────────┘
```

### `libs/libgraphics`

Software rasterizer — piksel tabanlı çizim kütüphanesi.

| Fonksiyon | Açıklama |
|---|---|
| `canvas_init()` | Dahili buffer allocate |
| `canvas_init_external()` | SHM buffer'a bind (Wayland modu) |
| `canvas_fill_rect()` | Dikdörtgen fill |
| `canvas_draw_text()` | Bitmap font rendering |
| `canvas_save_ppm()` | PPM dosyasına yaz |

---

## Servisler Katmanı

Detaylar için → [docs/services.md](services.md)

### Hızlı Referans

| Servis | Dil | Socket | Temel Görev |
|---|---|---|---|
| `servicemanager` | C | `/tmp/servicemanager.sock` | Servis adı → socket yolu çözümleme |
| `surfaceflinger` | C++ | `/tmp/surfaceflinger.sock` | Grafik compositor, frame birleştirme |
| `inputflinger` | Rust | `/tmp/inputflinger.sock` | Dokunma olayı routing |
| `apigateway` | Rust | `/tmp/apigateway.sock` | REST HTTP → IPC köprüsü |
| `powermanager` | Rust | — | Ekran ve batarya yönetimi |
| `apprunner` | Rust | — | Uygulama başlatma ve sandbox |

---

## Uygulama Katmanı

| Uygulama | Dil | Açıklama |
|---|---|---|
| `launcher` | C | Ana ekran, uygulama grid, dock |
| `statusbar` | C | Üst durum çubuğu (saat, batarya, sinyal) |
| `dialer` | C | Arama ekranı |
| `settings` | C | Sistem ayarları |

Tüm uygulamalar aynı pattern'i kullanır:

```c
// 1. Servicemanager'dan surfaceflinger yolunu al
resolve_service_path("mobile.graphics", sf_path, sizeof(sf_path));

// 2. Surface kaydet (PPM modu)
ipc_send_transaction(sf_fd, CMD_REGISTER_SURFACE, &data, &reply);

// 3. Canvas'a çiz
canvas_fill_rect(&canvas, ...);
canvas_draw_text(&canvas, ...);

// 4. Frame'i commit et
ipc_send_transaction(sf_fd, CMD_COMMIT_FRAME, &data, &reply);
```

---

## Grafik Alt Sistemi

### PPM Modu (Legacy)

```
app → CMD_COMMIT_FRAME → surfaceflinger
    surfaceflinger: tüm surface buffer'ları PPM olarak birleştir
    → out/display_composited.ppm
```

### Wayland Modu

```
QEMU boot: wayland=1
  → init: WAYLAND_ENABLED=1 env

surfaceflinger --wayland
  ├── wlroots headless backend
  ├── xdg_shell, wl_shm, wl_seat
  └── IPC thread: CMD_SEND_INPUT_EVENT=4

app (launcher, statusbar, ...)
  ├── wl_display_connect()
  ├── wl_compositor_create_surface()
  ├── xdg_wm_base_get_xdg_surface()
  └── SHM buffer → wl_surface_commit()
```

---

## Girdi Alt Sistemi

```
Dokunma Kaynağı (QEMU evdev / test)
         │
         ▼
   inputflinger  (/tmp/inputflinger.sock)
         │
    ┌────┴────────────────────────┐
    │  WAYLAND_ENABLED=1?        │
    │  Evet → surfaceflinger IPC │
    │  (CMD=4, wlr_seat inject)  │
    │  Hayır → listener dispatch │
    └────────────────────────────┘
         │
    ┌────┴──────────┐
    │   listeners   │
    │  launcher     │
    │  statusbar    │
    └───────────────┘
```

---

## Wayland Geçiş Mimarisi

Detaylar için → [docs/wayland-migration.md](wayland-migration.md)

| Kriter | PPM Modu | Wayland Modu |
|---|---|---|
| Compositor | surfaceflinger (custom) | wlroots headless |
| Frame output | `out/display_composited.ppm` | SHM buffer |
| Touch input | IPC listener dispatch | `wlr_seat_touch_notify_*` |
| Boot flag | — | `wayland=1` kernel cmdline |
| Cross-compile | ✅ | ❌ (host-only) |

---

## Derleme ve Hedef Platform

### Host Build (x86_64)

```bash
make all                    # Tüm bileşenleri derle
make -C services/surfaceflinger all USE_WAYLAND=1  # Wayland ile
```

### Cross-compile (aarch64)

```bash
make all CXX=aarch64-linux-gnu-g++ CC=aarch64-linux-gnu-gcc
# IS_CROSS=1 otomatik algılanır → USE_WAYLAND=0
```

### QEMU Çalıştırma

```bash
./scripts/qemu-run.sh            # PPM modu
./scripts/qemu-run.sh --wayland  # Wayland modu
./scripts/qemu-run.sh --headless # Ekransız
./scripts/qemu-run.sh --disk     # Disk boot (virtio-blk)
```

### Çıktı Dizini

```
out/
├── rootfs/
│   ├── sbin/init
│   ├── system/bin/     ← tüm servis ve uygulama ikilileri
│   ├── system/lib/     ← libipc.a, libgraphics.a, libi18n.a
│   └── etc/init.rc
├── initramfs.cpio.gz
└── display_composited.ppm
```

---

*© 2026 mcsimon — Apache License 2.0*
