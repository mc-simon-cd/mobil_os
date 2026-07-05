# 📦 opk — Kullanım Kılavuzu & Geliştirici Rehberi

`opk`, Orion OS (Arch Linux tabanlı) işletim sistemi için tasarlanmış modern, güvenlik öncelikli (sandbox-first), atomik güncelleme desteğine sahip ve Ed25519 dijital imzalı bir uygulama paket yöneticisidir.

Bu kılavuz, hem son kullanıcıların paket yükleme/çalıştırma adımlarını hem de geliştiricilerin paket hazırlama, imzalama ve yayınlama süreçlerini detaylandırmaktadır.

---

## 🚀 Hızlı Başlangıç

### Temel Komut Yapısı
```bash
opk <ALT_KOMUT> [SEÇENEKLER]
```

| Alt Komut | Açıklama | Örnek Kullanım |
| :--- | :--- | :--- |
| **`install`** | Yerel dosyadan veya registry'den paket kurar | `opk install com.example.app` |
| **`launch`** | Kurulu bir uygulamayı izole sandbox içinde başlatır | `opk launch com.example.app` |
| **`keygen`** | Geliştirici için yeni bir Ed25519 imza anahtarı üretir | `opk keygen` |
| **`pack`** | Proje dizininden `.opk` paket dosyası oluşturur | `opk pack ./my_app_dir` |
| **`sign`** | Oluşturulan paketi özel anahtarla imzalar | `opk sign my_app.opk` |
| **`verify`** | Paketin bütünlüğünü ve imzasını doğrular | `opk verify my_app.opk` |
| **`publish`** | İmzalanmış paketi registry sunucusuna yükler | `opk publish my_app.opk` |

---

## 🛡️ Kullanıcı Rehberi

### 1. Paket Kurulumu (`install`)
Uygulamalar yerel bir `.opk` arşiv dosyasından veya uzak paket kütüphanesinden kurulabilir. 

> [!NOTE]
> Kurulum sırasında arka planda geçici bir klasörde (`staging`) dosyalar açılır, checksum ve imza doğrulamaları yapılır ve `pre-install.sh` betiği çalıştırılır. Hata alınması durumunda atomik rollback mekanizması devreye girerek sistemi eski haline geri döndürür.

```bash
# A. Uzak kütüphaneden (Registry) otomatik indirme ve kurma:
opk install com.orion.notepad

# Belirli bir versiyonu kurmak için:
opk install com.orion.notepad --version 1.0.0

# B. Yerel bir paket dosyasından kurmak için:
opk install ./downloads/com.orion.notepad-1.0.0.opk
```

### 2. Uygulama Çalıştırma (`launch`)
Kurulan tüm uygulamalar varsayılan olarak izole edilmiş bir sandbox içinde çalıştırılır. Sandbox izinleri paketin içerisindeki `sandbox.policy` dosyasına göre şekillenir.

```bash
# Uygulamayı sandbox içinde güvenli şekilde başlatır:
opk launch com.orion.notepad
```

---

## 🛠️ Geliştirici Rehberi

### 1. Proje ve Paket Dosya Yapısı
Geçerli bir `.opk` paketi, arka planda bir ZIP arşividir. Proje dizininizin aşağıdaki şemaya uygun olması gerekir:

```
com.example.myapp/
├── manifest.toml          # [Zorunlu] Paket metadata ve bağımlılık bilgileri.
├── sandbox.policy         # [Zorunlu] Güvenlik ve donanım izin tanımları.
├── binaries/
│   └── x86_64/
│       └── myapp          # [Zorunlu] Derlenmiş ELF binary dosyası.
├── assets/
│   └── icon.png           # [Zorunlu] Uygulama ikonu (512x512).
└── scripts/               # [İsteğe Bağlı]
    ├── pre-install.sh     # Kurulum öncesi çalışacak betik.
    └── post-install.sh    # Kurulum sonrası çalışacak betik.
```

#### `manifest.toml` Örneği:
```toml
[package]
id = "com.example.myapp"
name = "My App"
version = "1.0.0"
description = "Harika bir Orion uygulaması."
author = "Geliştirici"
author_email = "dev@example.com"
min_os = "1.0.0"

[build]
entry_x86_64 = "binaries/x86_64/myapp"
exec_type = "elf"

[store]
category = "productivity"
rating = "ALL"
price_usd = 0.00
tier = "free"
```

#### `sandbox.policy` Örneği:
```toml
[meta]
policy_version = "1.0"
strict_mode = false

[filesystem]
home_read = "allow"
home_write = "allow"
temp_access = "allow"
documents_read = "ask"
documents_write = "deny"

[network]
internet = "allow"
localhost = "deny"
```

---

### 2. Adım Adım Yayınlama Süreci

#### Adım A: İmza Anahtarı Üretimi (`keygen`)
Paketlerinizi imzalamak için bir Ed25519 anahtar çifti oluşturun.

```bash
# Varsayılan konuma (~/.config/orion/signing/) anahtar üretir:
opk keygen

# Özel bir klasöre üretmek için:
opk keygen --output ./my_keys/
```
> [!IMPORTANT]
> - `private.key` dosyasını asla paylaşmayın veya git repolarına yüklemeyin.
> - `public.key` dosyanızı registry hesabınıza eklemeniz gerekir.

#### Adım B: Paketleme (`pack`)
Proje dizinindeki dosyaları tarar, bütünlük kontrol listesini (`checksums.sha256`) otomatik oluşturur ve `.opk` paketini paketler.

```bash
# Dizini paketler (otomatik olarak manifest.toml'a göre com.example.myapp-1.0.0.opk adını verir):
opk pack ./com.example.myapp

# Özel bir çıktı dosyası belirtmek için:
opk pack ./com.example.myapp --output ./builds/app.opk
```

#### Adım C: İmzalama (`sign`)
Paketinizin değiştirilmediğini ve güvenilir olduğunu kanıtlamak için özel anahtarınızla imzalayın.

```bash
# Varsayılan private.key'i kullanarak paketi imzalar:
opk sign ./com.example.myapp-1.0.0.opk

# Farklı bir özel anahtar ile imzalamak için:
opk sign ./com.example.myapp-1.0.0.opk --key ./my_keys/private.key
```

#### Adım D: Doğrulama (`verify`)
Paketinizin imzasını ve dosya bütünlüğünü kurmadan önce test edin:

```bash
# Paketi kendi ürettiğiniz public key ile doğrulamak için:
opk verify ./com.example.myapp-1.0.0.opk --key ./my_keys/public.key
```

#### Adım E: Yayınlama (`publish`)
İmzalı paketi registry sunucusuna yükleyin.

```bash
# Yüklemeden önce API anahtarınızı tanımlayın:
export ORION_API_KEY="orion_live_your_secret_api_key"

# Paketi stable kanalına yayınlar:
opk publish ./com.example.myapp-1.0.0.opk --channel stable
```

---

## 🔒 Güvenlik & İzolasyon Mimarisi

`opk` uygulamaları çalıştırırken Linux Çekirdeği (Kernel) özelliklerini kullanan çok katmanlı bir izolasyon uygular:

1. **User Namespaces:** Kök yetkileri (rootless execution) olmadan çalışmayı sağlar. Kullanıcı UID/GID eşlemeleri yapılarak uygulamanın host sisteme zarar vermesi engellenir.
2. **Mount Namespaces:** Uygulamanın sadece kendi kurulu olduğu dizine ve izin verilen sistem yollarına erişmesini sağlar, disk geneline erişim kapatılır.
3. **Network Namespaces:** `sandbox.policy` dosyasında `internet = "deny"` ise uygulama için sanal bir ağ soyutlaması yapılarak internet erişimi tamamen kesilir.
4. **Seccomp BPF:** Çekirdeğe yapılan tehlikeli sistem çağrıları (system calls) sınırlandırılarak uygulamanın işletim sisteminin çekirdeğine sızması önlenir.

---

## 💡 İpuçları ve Sorun Giderme

### Çevrimdışı / Geliştirici (Mock) Modu
Geliştirme aşamasında gerçek bir sunucuya ihtiyaç duymadan test yapmak için `mock://` protokolünü kullanabilirsiniz:
```bash
# Mock registry kullanarak paket kurma testi:
opk install com.example.mockapp --registry "mock://offline_dev"
```

### Kurulum Hatası ve Rollback Durumu
Eğer yüklediğiniz bir güncellemenin `post-install.sh` betiğinde hata oluşursa endişelenmeyin! `opk` otomatik olarak:
1. Hatalı yüklenen yeni dosyaları siler.
2. `/tmp/orion/apps/{app_id}.backup` adresindeki eski çalışan sürümü geri yükler.
3. Masaüstü kısayollarını eski haline getirir.
Uygulamanız kesintisiz olarak çalışmaya devam eder.
