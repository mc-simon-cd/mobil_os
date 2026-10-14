# Orion OS — Sistem Mimarisi

> Versiyon: 1.0.0 · Platform: ARM64 (QEMU virt / PinePhone) · Çekirdek & Ekosistem: %100 Pure Rust

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
9. [Derleme ve Hedef Platform](#derleme-ve-hedef-platform)

---

## Genel Bakış

Orion OS, ARM64 donanım hedefini (ve QEMU `virt` emülasyonunu) desteklemek üzere tasarlanmış **%100 Rust** dilinde yazılmış bağımsız bir mobil işletim sistemidir.

**Temel Mimarisi:**

- **Tam Güvenli ve Bağımsız Katmanlar** — Bellek güvenliği ve `crt-static` ile sıfır dış kütüphane bağımlılığı.
- **Modüler Servisler** — Her servis bağımsız bir Rust binary'si, Unix Domain Socket üzerinden `libipc-rs` mesajlaşması.
- **Yazılım Kompozitörü (`surfaceflinger`)** — Çok katmanlı (Statusbar + Uygulama katmanları) frame birleştirme.
- **Statik Çapraz Derleme** — `aarch64-unknown-linux-gnu` hedefi için Cargo ile tek komutla derleme.

---

## Katman Modeli

```
┌─────────────────────────────────────────────────────────┐
│                    UYGULAMA KATMANI                     │
│    launcher │ dialer │ settings │ opk package manager   │
├─────────────────────────────────────────────────────────┤
│                    SERVİS KATMANI                       │
│    surfaceflinger │ inputflinger │ apigateway │ statusbar│
│    servicemanager │ powermanager │ apprunner            │
├─────────────────────────────────────────────────────────┤
│                    KÜTÜPHANELER                         │
│         libipc-rs │ libgraphics │ libi18n               │
├─────────────────────────────────────────────────────────┤
│                    ÇEKİRDEK (core/)                     │
│               init (Rust Static Bin - PID 1)            │
├─────────────────────────────────────────────────────────┤
│               LINUX ÇEKİRDEĞİ (6.x ARM64)               │
└─────────────────────────────────────────────────────────┘
```

---

## Önyükleme Akışı

```
QEMU virt (ARM64)
    │
    ▼
Linux Kernel (Image) ──── e1000 / virtio-blk / ext4
    │
    ▼ rdinit=/init
core/init (Rust PID 1)
    │  1. Sanal dosya sistemleri (proc, sys, dev, tmp) mount edilir
    │  2. Ağ arayüzleri yapılandırılır (lo, eth0)
    │  3. Sistem property deposu başlatılır
    │  4. /etc/init.rc yapılandırması ayrıştırılır
    │  5. Servisler başlatılır (fork + execve)
    │
    ├──► servicemanager   (/tmp/servicemanager.sock)
    ├──► surfaceflinger   (/tmp/surfaceflinger.sock)
    ├──► inputflinger     (/tmp/inputflinger.sock)
    ├──► powermanager     (/tmp/powermanager.sock)
    ├──► apigateway       (/tmp/apigateway.sock)
    ├──► apprunner        (/tmp/apprunner.sock)
    ├──► statusbar        (/tmp/statusbar.sock)
    └──► launcher         (/tmp/launcher.sock)
```

---

## Çekirdek Bileşenler

### `core/init` (Rust PID 1)
- **Sorumluluk**: Sanal dosya sistemlerini mount eder, kernel modüllerini yükler, `init.rc` komutlarını çalıştırır ve arka plan servislerini izler (respawn).

### `libs/libipc-rs`
- **Sorumluluk**: Parcel tabanlı ikili veri serileştirme ve Unix Domain Socket işlemleri.

### `libs/libgraphics`
- **Sorumluluk**: Çizim ilkel elemanları (çizgi, daire, dikdörtgen, degrade, gömülü font ve PPM kaydetme).

---

## Servisler Katmanı

| Servis | Dil | Socket Path | Görev |
| :--- | :--- | :--- | :--- |
| `servicemanager` | Rust | `/tmp/servicemanager.sock` | Servis adı çözümleme rehberi |
| `surfaceflinger` | Rust | `/tmp/surfaceflinger.sock` | Çok katmanlı ekran birleştirme kompozitörü |
| `inputflinger` | Rust | `/tmp/inputflinger.sock` | Dokunma ve girdi olay yönlendirme |
| `powermanager` | Rust | `/tmp/powermanager.sock` | Pil durumu ve güç profili yönetimi |
| `apigateway` | Rust | `/tmp/apigateway.sock` | REST HTTP -> IPC dönüştürücü |
| `apprunner` | Rust | `/tmp/apprunner.sock` | Uygulama yaşam döngüsü yöneticisi |

---

## Uygulama Katmanı

| Uygulama | Dil | Görev |
| :--- | :--- | :--- |
| `launcher` | Rust | Ana ekran, uygulama grid ve dock |
| `statusbar` | Rust | Saat, pil durumu ve bildirim çubuğu |
| `dialer` | Rust | Telefon arama ekranı |
| `settings` | Rust | Sistem ayarları ve güç modu yönetimi |
| `opk` | Rust | Paket yöneticisi (kurulum, doğrulama, çalıştırma) |

---

## Derleme ve Hedef Platform

### Çapraz Derleme (ARM64)
```bash
./scripts/build.sh
```

### Rootfs ve Paketleme
```bash
./scripts/download-kernel.sh
./scripts/make-rootfs.sh
```

### QEMU Simülasyonu
```bash
./scripts/qemu-run.sh --headless
```

---

*© 2026 mcsimon — Apache License 2.0*
