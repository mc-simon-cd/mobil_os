# `.opk` Paket Formatı — Teknik Spesifikasyon

> **Versiyon:** 1.0.0-draft  
> **Durum:** Tasarım Aşaması  
> **Hedef Platform:** Orion OS (Arch Linux tabanlı)  
> **Son Güncelleme:** 2026

---

## İçindekiler

1. [Genel Bakış](#genel-bakış)
2. [Dosya Yapısı](#dosya-yapısı)
3. [manifest.toml Şeması](#manifesttoml-şeması)
4. [sandbox.policy Şeması](#sandboxpolicy-şeması)
5. [İmzalama ve Doğrulama](#i̇mzalama-ve-doğrulama)
6. [Checksum Sistemi](#checksum-sistemi)
7. [Versiyon Kuralları](#versiyon-kuralları)
8. [Kısıtlamalar ve Limitler](#kısıtlamalar-ve-limitler)
9. [Örnek Paket](#örnek-paket)

---

## Genel Bakış

`.opk` formatı, Orion ekosistemi için tasarlanmış bir uygulama paket formatıdır. Teknik olarak bir **ZIP arşivi**dir — herhangi bir ZIP aracıyla açılabilir, ancak geçerliliği için Orion Bekçi (gatekeeper) tarafından doğrulanmış bir Ed25519 imzası zorunludur.

### Temel Prensipler

| Prensip | Açıklama |
|---|---|
| **Açık format** | ZIP tabanlı, özel binary format değil |
| **İmza zorunlu** | İmzasız paket kurulmaz (kullanıcı uyarıyla override edebilir) |
| **Sandbox-first** | Her uygulama izolasyonla çalışır |
| **İnform, block değil** | Bekçi bloklamaz, kullanıcıyı bilgilendirir |
| **Çoklu mimari** | x86_64, aarch64 aynı pakette |

---

## Dosya Yapısı

```
com.orion.myapp-1.2.3.opk
│
├── manifest.toml          ← Zorunlu. Paket metadata ve bağımlılıklar.
├── signature.ed25519      ← Zorunlu. Bekçi imzası (binary, 64 byte).
├── checksums.sha256       ← Zorunlu. Tüm dosyaların hash listesi.
│
├── binaries/
│   ├── x86_64/
│   │   └── myapp          ← ELF binary
│   └── aarch64/
│       └── myapp          ← ELF binary (opsiyonel ama önerilen)
│
├── assets/
│   ├── icon.png           ← 512x512 PNG (zorunlu)
│   ├── icon.svg           ← Vektör (opsiyonel ama önerilen)
│   └── screenshots/
│       ├── 01.webp
│       └── 02.webp
│
├── sandbox.policy         ← Zorunlu. İzin tanımları.
│
├── locales/               ← Opsiyonel. i18n dosyaları.
│   ├── tr.json
│   └── en.json
│
└── scripts/               ← Opsiyonel.
    ├── pre-install.sh     ← Kurulum öncesi
    └── post-install.sh    ← Kurulum sonrası
```

### Dosya İsimlendirme Kuralı

```
{reverse-domain-id}-{versiyon}.opk

Örnekler:
  com.orion.browser-2.1.0.opk
  app.mycompany.tool-0.9.1-beta.1.opk
```

---

## `manifest.toml` Şeması

Tüm alanlar TOML formatındadır. Zorunlu alanlar `*` ile işaretlenmiştir.

```toml
# ─────────────────────────────────────────
# [package] — Temel Kimlik Bilgileri
# ─────────────────────────────────────────

[package]
id          = "com.orion.myapp"   # * Reverse domain. Değiştirilemez sonradan.
name        = "My App"              # * Görünen ad (max 64 karakter)
version     = "1.2.3"              # * SemVer (bkz. Versiyon Kuralları)
description = "Kısa açıklama"      # * Max 256 karakter
author      = "Geliştirici Adı"    # * Gerçek ad veya takma ad
author_email = "dev@example.com"   # * İletişim (gizli tutulabilir, store'a gönderilir)
homepage    = "https://example.com" # Opsiyonel
source_url  = "https://github.com/..." # Opsiyonel. Açık kaynak ise.
license     = "MIT"                # Opsiyonel. SPDX identifier.

min_os      = "1.0.0"              # * Minimum Orion OS versiyonu
max_os      = ""                   # Opsiyonel. Boşsa sınırsız.


# ─────────────────────────────────────────
# [build] — Binary Bilgileri
# ─────────────────────────────────────────

[build]
entry_x86_64   = "binaries/x86_64/myapp"   # * x86_64 binary yolu
entry_aarch64  = "binaries/aarch64/myapp"  # Opsiyonel

# Desteklenen exec tipler: elf | script | electron | flatpak-compat
exec_type = "elf"


# ─────────────────────────────────────────
# [store] — Mağaza Bilgileri
# ─────────────────────────────────────────

[store]
category    = "productivity"        # * Kategori (bkz. kategori listesi)
rating      = "ALL"                 # * ALL | 13+ | 17+ | 18+
price_usd   = 0.00                  # * 0.00 = ücretsiz
tier        = "free"                # * free | standard | premium
tags        = ["editor", "notes"]   # Opsiyonel, max 10 adet

# Abonelik modeli (opsiyonel)
[store.subscription]
enabled         = false
price_monthly   = 2.99
price_yearly    = 24.99
trial_days      = 7


# ─────────────────────────────────────────
# [dependencies] — Bağımlılıklar
# ─────────────────────────────────────────

[dependencies]
# Sistem kütüphaneleri — paket yöneticisi bunları kontrol eder
system = [
  "libssl >= 3.0",
  "libsqlite3 >= 3.40"
]

# Orion paket bağımlılıkları
packages = [
  "com.orion.runtime >= 2.0.0"
]


# ─────────────────────────────────────────
# [update] — Güncelleme Politikası
# ─────────────────────────────────────────

[update]
channel         = "stable"          # stable | beta | nightly
auto_update     = true              # Kullanıcı override edebilir
changelog_url   = "https://example.com/changelog"
```

### Geçerli Kategori Listesi

```
productivity | utilities | development | education | entertainment
media | graphics | games | security | networking | system | finance
```

---

## `sandbox.policy` Şeması

TOML formatında. Her izin üç değer alır: `"allow"` | `"deny"` | `"ask"`.

```toml
# sandbox.policy — Orion Sandbox İzin Tanımları
# Versiyon: 1.0

[meta]
policy_version = "1.0"
strict_mode    = false   # true: tanımlanmamış her şey deny

# ─────────────────────────────────────────
# Dosya Sistemi Erişimi
# ─────────────────────────────────────────

[filesystem]
home_read       = "allow"   # ~/myapp/ altına otomatik kısıtlanır
home_write      = "allow"
temp_access     = "allow"
documents_read  = "ask"     # ~/Documents/
documents_write = "deny"
downloads_read  = "ask"
system_read     = "deny"    # /etc, /usr vb.
arbitrary_path  = "deny"    # Rastgele yol erişimi

# ─────────────────────────────────────────
# Ağ Erişimi
# ─────────────────────────────────────────

[network]
internet        = "allow"
localhost       = "allow"
local_network   = "ask"     # 192.168.x.x, 10.x.x.x

# ─────────────────────────────────────────
# Donanım
# ─────────────────────────────────────────

[hardware]
camera          = "deny"
microphone      = "deny"
gpu_access      = "allow"
usb_devices     = "deny"
bluetooth       = "deny"
location        = "deny"

# ─────────────────────────────────────────
# Sistem
# ─────────────────────────────────────────

[system]
notifications   = "allow"
clipboard_read  = "ask"
clipboard_write = "allow"
autostart       = "deny"    # Önyükleme ile başlatma
background_run  = "deny"    # Arka planda çalışma
ipc             = "deny"    # Diğer süreçlerle IPC
exec_subprocess = "deny"    # Alt süreç başlatma
```

### Sandbox Uygulama Mekanizması

Orion OS, Linux kernel özelliklerini kullanır:

```
seccomp-bpf    → sistem çağrısı filtreleme
namespaces     → dosya sistemi, ağ, süreç izolasyonu
cgroups v2     → CPU/bellek/IO limitleri
AppArmor       → profil tabanlı MAC (zorunlu erişim kontrolü)
```

---

## İmzalama ve Doğrulama

### Algoritma

**Ed25519** (RFC 8032) — 64 byte imza, 32 byte public key.

### İmza Kapsamı

`signature.ed25519`, şu dosyaların SHA-256 hash'lerinin sıralı birleşiminin imzasıdır:

```
imzalanan = SHA256(manifest.toml)
          + SHA256(sandbox.policy)
          + SHA256(checksums.sha256)
          + SHA256(binaries/x86_64/myapp)   # varsa
          + SHA256(binaries/aarch64/myapp)  # varsa
```

> **Not:** `assets/` ve `scripts/` dosyaları `checksums.sha256` üzerinden dolaylı olarak imza kapsamına girer.

### Doğrulama Akışı

```
1. signature.ed25519 oku (64 byte)
2. Orion Registry'den geliştirici public key'ini al
3. İmzayı doğrula (ed25519_verify)
   └─ FAIL → kullanıcıya uyarı göster, devam etmek için onay iste
   └─ OK   → checksums.sha256'yı doğrula
              └─ FAIL → kurulumu durdur (bütünlük ihlali)
              └─ OK   → kurulum devam eder
```

### Güven Seviyeleri

| Seviye | Açıklama | Görsel |
|---|---|---|
| **Verified** | Orion imzalı, geliştirici doğrulanmış | 🟢 Yeşil rozet |
| **Signed** | Geliştirici imzalı, Orion doğrulamamış | 🟡 Sarı rozet |
| **Unsigned** | İmza yok | 🔴 Kırmızı uyarı |

---

## Checksum Sistemi

`checksums.sha256` — paket içindeki her dosyanın SHA-256 hash'ini tutar.

### Format

```
# checksums.sha256
# Üretilme zamanı: 2026-01-15T10:30:00Z

e3b0c44298fc1c149afb...  manifest.toml
a87ff679a2f3e71d9181...  sandbox.policy
fcab2ce88c0e4a758b9a...  binaries/x86_64/myapp
d41d8cd98f00b204e980...  assets/icon.png
```

### Format Kuralı

```
{sha256_hex}  {göreli_dosya_yolu}
```

İki boşluk ile ayrılır (`sha256sum` çıktısıyla uyumlu).

---

## Versiyon Kuralları

**Semantic Versioning 2.0.0** (semver.org) zorunludur.

```
MAJOR.MINOR.PATCH[-pre_release][+build_metadata]

1.2.3
1.2.3-beta.1
1.2.3-alpha.2+build.20260115
0.9.0-rc.1
```

### Kurallar

| Durum | Değişiklik |
|---|---|
| Geriye dönük uyumsuz API değişikliği | MAJOR artır |
| Geriye dönük uyumlu yeni özellik | MINOR artır |
| Hata düzeltmesi | PATCH artır |
| Yayın öncesi | `-alpha`, `-beta`, `-rc` ekle |

### Registry Kabul Kuralları

- `0.x.x` — geliştirici önizleme, store'da açıkça işaretlenir
- `1.0.0` ve üzeri — kararlı yayın
- Aynı versiyon tekrar yüklenemez (immutable releases)
- Bir versiyon geri çekilebilir ama silinemez (audit trail)

---

## Kısıtlamalar ve Limitler

| Alan | Limit |
|---|---|
| Maksimum paket boyutu | 4 GB |
| Maksimum binary boyutu (tek mimari) | 2 GB |
| `manifest.toml` boyutu | 64 KB |
| `sandbox.policy` boyutu | 16 KB |
| Paket ismi (display name) | 64 karakter |
| Açıklama | 256 karakter |
| Tag sayısı | 10 |
| Screenshot sayısı | 8 |
| Desteklenen görsel formatları | PNG, WebP, AVIF |
| Minimum ikon boyutu | 512×512 PNG |
| `pre/post-install.sh` boyutu | 1 MB |
| `pre/post-install.sh` çalışma süresi | 60 saniye |

---

## Örnek Paket

Minimal, gerçek dünya benzeri örnek:

### Dizin yapısı

```
com.orion.notepad-1.0.0.opk
├── manifest.toml
├── signature.ed25519
├── checksums.sha256
├── binaries/
│   └── x86_64/
│       └── notepad
├── assets/
│   ├── icon.png
│   └── screenshots/
│       └── 01.webp
└── sandbox.policy
```

### `manifest.toml`

```toml
[package]
id           = "com.orion.notepad"
name         = "Orion Notepad"
version      = "1.0.0"
description  = "Sade ve hızlı not alma uygulaması."
author       = "Mustafa"
author_email = "dev@orion.app"
license      = "MIT"
min_os       = "1.0.0"

[build]
entry_x86_64 = "binaries/x86_64/notepad"
exec_type    = "elf"

[store]
category  = "productivity"
rating    = "ALL"
price_usd = 0.00
tier      = "free"
tags      = ["notes", "text", "editor"]

[update]
channel    = "stable"
auto_update = true
```

### `sandbox.policy`

```toml
[meta]
policy_version = "1.0"
strict_mode    = false

[filesystem]
home_read       = "allow"
home_write      = "allow"
documents_read  = "ask"
documents_write = "ask"

[network]
internet    = "deny"
localhost   = "deny"

[hardware]
camera      = "deny"
microphone  = "deny"

[system]
notifications   = "allow"
clipboard_read  = "allow"
clipboard_write = "allow"
autostart       = "deny"
```

---

## Ek Belgeler

| Belge | Açıklama |
|---|---|
| `opk-registry-api.md` | Registry backend API spesifikasyonu |
| `opk-bekci-arch.md` | Bekçi (gatekeeper) mimari detayları |
| `opk-install-pipeline.md` | Kurulum pipeline adımları |
| `opk-sandbox-impl.md` | seccomp/namespace uygulama detayları |
| `opk-signing-guide.md` | Geliştirici imzalama rehberi |

---

*Bu doküman Orion ekosistemi dahilinde geliştirilmektedir.*  
*Katkı, geri bildirim ve soru için: `dev@orion.app`*
