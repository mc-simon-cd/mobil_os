# Flatpak Uyumluluk ve Paketleme Rehberi (Flatpak Packaging Guide)

> **Konum:** `docs/FLATPAK_PACKAGING_GUIDE.md`  
> **Destek:** `manifest.toml -> exec_type = "flatpak-compat"`

---

## 1. Genel Bakış

Flatpak, masaüstü Linux ekosisteminde geniş bir uygulama kütüphanesine sahiptir. Orion OS, yerel ELF ikililerinin yanı sıra mevcut Flatpak uygulamalarını `.opk` ekosistemine entegre etmek için **`flatpak-compat`** çalışma zamanı köprüsünü destekler.

Bu sayede geliştiriciler, sıfırdan derleme yapmadan var olan Flatpak uygulamalarını Orion OS'in güvenlik modeliyle uyumlu hale getirebilirler.

---

## 2. Flatpak vs. Orion Sandbox Karşılaştırması

| Özellik | Flatpak | Orion (`opk`) |
|---|---|---|
| **Paket Formatı** | OSTree / OCI katmanı | Standart Ed25519 İmzalı ZIP (`.opk`) |
| **İzolasyon Motoru** | Bubblewrap (`bwrap`) | Linux Namespaces + Seccomp BPF + AppArmor |
| **İzin Tanımı** | Manifest metadata bayrakları | TOML tabanlı `sandbox.policy` |
| **Hedef Donanım** | PC / Masaüstü | Mobil Cihazlar & Dokunmatik Ekranlar |
| **Yönetici Yetkisi** | Rootless veya System | Tamamen Rootless |

---

## 3. `manifest.toml` Yapılandırması

Bir Flatpak paketini `.opk` olarak dağıtmak için `build.exec_type` alanı `flatpak-compat` olarak ayarlanmalıdır:

```toml
[package]
id = "org.mozilla.firefox"
name = "Firefox"
version = "125.0.0"
description = "Web Tarayıcısı"
author = "Mozilla"
author_email = "support@mozilla.org"
min_os = "1.0.0"

[build]
entry_x86_64 = "binaries/x86_64/firefox"
entry_aarch64 = "binaries/aarch64/firefox"
exec_type = "flatpak-compat"

[store]
category = "networking"
rating = "ALL"
price_usd = 0.00
tier = "free"
```

---

## 4. İzin Dönüşüm Haritası (Permissions Mapping)

Flatpak izinleri, Orion `sandbox.policy` formatına şu şekilde eşlenir:

| Flatpak İzni | Orion `sandbox.policy` Eşleniği |
|---|---|
| `--share=network` | `[network] internet = "allow"` |
| `--filesystem=home` | `[filesystem] home_read = "allow"`, `home_write = "allow"` |
| `--device=all` | `[hardware] camera = "ask"`, `microphone = "ask"` |
| `--talk-name=org.freedesktop.Notifications` | `[system] notifications = "allow"` |

---

## 5. Paketleme Adımları

1. Flatpak ikililerini ve runtime bağımlılıklarını export edin:
   ```bash
   flatpak run --command=bash org.example.app
   ```
2. İlgili dosyaları bir dizin altına toplayın (`manifest.toml`, `sandbox.policy`, `assets/icon.png`, `binaries/`).
3. Paketi `opk pack` ile oluşturun ve imzalayın:
   ```bash
   opk pack ./firefox_flatpak_dir -o firefox-125.0.0.opk
   opk sign firefox-125.0.0.opk
   ```
