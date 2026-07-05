# Geliştirici İmzalama Rehberi — Teknik Spesifikasyon

> **Versiyon:** 1.0.0-draft  
> **Algoritma:** Ed25519 (RFC 8032)  
> **Araç:** `opk` CLI  
> **Durum:** Tasarım Aşaması

---

## İçindekiler

1. [Neden İmzalama?](#neden-i̇mzalama)
2. [Anahtar Üretimi](#anahtar-üretimi)
3. [Registry'ye Kayıt](#registrye-kayıt)
4. [Paket İmzalama](#paket-i̇mzalama)
5. [İmza Doğrulama](#i̇mza-doğrulama)
6. [Anahtar Yönetimi](#anahtar-yönetimi)
7. [`opk` CLI Referansı](#opk-cli-referansı)
8. [CI/CD Entegrasyonu](#cicd-entegrasyonu)

---

## Neden İmzalama?

İmzalama üç soruyu yanıtlar:

```
1. KİM yaptı?    → Public key ile geliştirici kimliği doğrulanır
2. DEĞİŞTİ mi?   → checksums.sha256 ile dosya bütünlüğü kontrol edilir
3. GÜVENILIR mi? → Orion'un onayıyla güven seviyesi belirlenir
```

**Ed25519 neden?**

| Özellik | RSA-2048 | Ed25519 |
|---|---|---|
| İmza boyutu | 256 byte | **64 byte** |
| Hız | Yavaş | **Çok hızlı** |
| Güvenlik | ~112 bit | **~128 bit** |
| Uygulama kolaylığı | Orta | **Kolay** |
| Yan kanal dayanıklılığı | Zayıf | **Güçlü** |

---

## Anahtar Üretimi

### `opk` CLI ile (Önerilen)

```bash
opk keygen

# Çıktı:
# ✅ Anahtar çifti oluşturuldu:
#   Private key: ~/.config/orion/signing/private.key  (gizli tut!)
#   Public key:  ~/.config/orion/signing/public.key
#   Fingerprint: SHA256:xK9mP2...
```

### Manuel — Rust

```rust
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

fn generate_keypair() -> (SigningKey, [u8; 32]) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_bytes = signing_key.verifying_key().to_bytes();
    (signing_key, verifying_bytes)
}
```

### Dosya Formatı

```
# ~/.config/orion/signing/private.key
# Format: raw binary, 32 byte Ed25519 seed
# Base64 kodlu, tek satır

MCSIMON_PRIVATE_KEY_V1:base64_encoded_32_bytes

# ~/.config/orion/signing/public.key
# Format: raw binary, 32 byte
# Registry'ye bu gönderilir

MCSIMON_PUBLIC_KEY_V1:base64_encoded_32_bytes
```

### Güvenlik Uyarıları

```
⛔ Private key'i asla:
   - Git repository'e commit etme
   - Başkasıyla paylaşma
   - Şifrelenmemiş cloud'a yükleme
   - E-posta ile gönderme

✅ Private key'i:
   - Offline yedekle (USB, kağıt)
   - Şifreli disk veya password manager'da sakla
   - Bir passphrase ile koru (opk destekler)
```

### Passphrase Koruması

```bash
# Anahtar oluşturma sırasında passphrase ekle
opk keygen --passphrase

# İmzalama sırasında passphrase istenir
opk sign myapp.opk
# Passphrase: ****
```

Implementasyon: Private key, `Argon2id` ile türetilmiş anahtarla şifrelenir.

```
şifreli_private_key = AES-256-GCM(
    key = Argon2id(passphrase, salt, m=65536, t=3, p=4),
    plaintext = raw_private_key_bytes
)
```

---

## Registry'ye Kayıt

### Adımlar

```bash
# 1. Geliştirici hesabı oluştur (bir kez)
opk account register

# 2. Public key'i kaydet
opk key add --label "Ana Makine"

# Çıktı:
# ✅ Public key kaydedildi
#   Key ID: key_01HX...
#   Fingerprint: SHA256:xK9mP2...
#   Label: Ana Makine
```

### API Çağrısı (arka planda)

```bash
curl -X POST https://registry.orion.app/v1/developer/keys \
  -H "Authorization: Bearer mcs_live_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "public_key": "base64_encoded_public_key",
    "label": "Ana Makine"
  }'
```

---

## Paket İmzalama

### Komut

```bash
opk sign com.orion.notepad-1.2.3.opk
```

### Ne Olur?

```
1. .opk dosyasını aç (ZIP)
2. Şu dosyaların SHA256'sını hesapla:
   - manifest.toml
   - sandbox.policy
   - checksums.sha256
   - binaries/x86_64/notepad    (varsa)
   - binaries/aarch64/notepad   (varsa)
3. Hash'leri sıraya koy ve birleştir
4. Birleşik veriyi Ed25519 private key ile imzala
5. 64 byte imzayı signature.ed25519 olarak yaz
6. .opk ZIP'i güncelle
```

### Rust Implementasyonu

```rust
use ed25519_dalek::{SigningKey, Signer};
use sha2::{Sha256, Digest};
use std::io::Read;

pub fn sign_package(
    package_path: &Path,
    signing_key: &SigningKey,
) -> Result<(), SignError> {

    let mut archive = open_opk(package_path)?;

    // İmzalanacak dosyaların hash'lerini topla
    let mut message = Vec::new();

    for required_file in &["manifest.toml", "sandbox.policy", "checksums.sha256"] {
        let content = read_from_zip(&mut archive, required_file)?;
        message.extend_from_slice(&sha256(&content));
    }

    // Binary'ler (opsiyonel)
    for arch_binary in &["binaries/x86_64/", "binaries/aarch64/"] {
        if let Ok(content) = read_from_zip(&mut archive, arch_binary) {
            message.extend_from_slice(&sha256(&content));
        }
    }

    // İmzala
    let signature = signing_key.sign(&message);

    // ZIP'e yaz
    write_to_zip(&mut archive, "signature.ed25519", signature.to_bytes().as_ref())?;

    println!("✅ İmzalandı. Fingerprint: SHA256:{}", hex::encode(&sha256(signing_key.verifying_key().as_bytes())[..8]));

    Ok(())
}

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().into()
}
```

---

## İmza Doğrulama

### Komut

```bash
opk verify com.orion.notepad-1.2.3.opk

# Çıktı:
# ✅ Checksum: Tüm dosyalar geçerli
# ✅ İmza: Doğrulandı
#   Geliştirici: Mustafa (dev@orion.app)
#   Key Fingerprint: SHA256:xK9mP2...
#   Orion onayı: ✅ Evet
# 🟢 Güven Seviyesi: Verified
# 📊 Risk Skoru: 10/100 (Düşük)
```

### Doğrulama Akışı

```rust
pub fn verify_package(
    package_path: &Path,
    trusted_keys: &[VerifyingKey],
) -> VerificationResult {

    let archive = open_opk(package_path)?;

    // Adım 1: Checksum doğrula (zorunlu)
    let checksums = parse_checksums(&archive)?;
    for (file_path, expected_hash) in &checksums {
        let actual_hash = sha256_of_file(&archive, file_path)?;
        if actual_hash != *expected_hash {
            return VerificationResult::ChecksumMismatch {
                file: file_path.clone(),
            };
        }
    }

    // Adım 2: İmzayı yeniden oluştur
    let mut message = Vec::new();
    for f in &["manifest.toml", "sandbox.policy", "checksums.sha256"] {
        message.extend_from_slice(&sha256_of_file(&archive, f)?);
    }
    for arch in &["binaries/x86_64/", "binaries/aarch64/"] {
        if let Ok(hash) = sha256_of_file(&archive, arch) {
            message.extend_from_slice(&hash);
        }
    }

    // Adım 3: signature.ed25519 oku
    let sig_bytes = read_from_zip(&archive, "signature.ed25519")?;
    let signature = Signature::from_slice(&sig_bytes)?;

    // Adım 4: Güvenilen keyler arasında ara
    for key in trusted_keys {
        if key.verify(&message, &signature).is_ok() {
            return VerificationResult::Verified {
                fingerprint: hex::encode(&sha256(key.as_bytes())[..8]),
            };
        }
    }

    VerificationResult::SignatureMismatch
}
```

---

## Anahtar Yönetimi

### Birden Fazla Makine

Her makinede ayrı anahtar çifti oluşturup registry'e ekle:

```bash
# Makinede A
opk keygen
opk key add --label "Geliştirme Makinesi"

# Makinede B
opk keygen
opk key add --label "CI Sunucusu"
```

Her iki key ile imzalanan paketler `Verified` seviyesinde görünür.

### Anahtar İptali

```bash
opk key revoke key_01HX...

# Bu key ile imzalanmış mevcut paketler:
# → "Signed" güven seviyesine düşer
# → Yeniden imzalama önerilir
```

Registry API:

```bash
DELETE /v1/developer/keys/key_01HX...
Authorization: Bearer mcs_live_xxx
```

### Anahtar Rotasyonu (Önerilen Periyot: 1 Yıl)

```bash
# 1. Yeni anahtar üret ve kaydet
opk keygen --output new_key
opk key add --key new_key.pub --label "2027 Anahtarı"

# 2. Mevcut paketleri yeni anahtarla yeniden imzala
opk resign --all --new-key new_key

# 3. Eski anahtarı iptal et
opk key revoke key_eski...
```

---

## `opk` CLI Referansı

### Anahtar Komutları

```bash
# Anahtar üret
opk keygen [--passphrase] [--output <path>]

# Anahtarları listele
opk key list

# Registry'ye ekle
opk key add [--key <public_key_file>] [--label <name>]

# İptal et
opk key revoke <key_id>
```

### Paket Komutları

```bash
# Paket oluştur (dizinden .opk üret)
opk pack <dizin> [--output <dosya.opk>]

# İmzala
opk sign <dosya.opk> [--key <private_key_file>]

# Doğrula (yerel)
opk verify <dosya.opk>

# Yükle (build + sign + upload)
opk publish <dosya.opk> [--channel stable|beta|nightly]

# Mevcut paketleri yeniden imzala
opk resign --all
opk resign <dosya.opk> --new-key <private_key_file>

# Belirli versiyonu geri çek
opk yank <package_id> <version> --reason "Açıklama"
```

### Manifest Komutları

```bash
# Yeni proje başlat (manifest.toml + sandbox.policy oluştur)
opk init

# Manifest doğrula
opk manifest check

# Policy doğrula ve risk skoru göster
opk policy check
```

---

## CI/CD Entegrasyonu

### GitHub Actions Örneği

```yaml
# .github/workflows/publish.yml

name: Orion Publish

on:
  push:
    tags: ['v*']

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: opk kur
        run: curl -fsSL https://cli.orion.app/install.sh | bash

      - name: Private key'i ortama yükle
        env:
          MCSIMON_PRIVATE_KEY: ${{ secrets.MCSIMON_PRIVATE_KEY }}
        run: |
          mkdir -p ~/.config/orion/signing
          echo "$MCSIMON_PRIVATE_KEY" > ~/.config/orion/signing/private.key

      - name: Paketle, imzala ve yayınla
        env:
          MCSIMON_API_KEY: ${{ secrets.MCSIMON_API_KEY }}
        run: |
          opk pack ./dist --output myapp.opk
          opk sign myapp.opk
          opk verify myapp.opk
          opk publish myapp.opk --channel stable
```

### Ortam Değişkenleri

```bash
MCSIMON_PRIVATE_KEY_PATH   # Private key dosya yolu
MCSIMON_PRIVATE_KEY        # Private key içeriği (CI için)
MCSIMON_API_KEY            # mcs_live_xxx API anahtarı
MCSIMON_REGISTRY_URL       # Özel registry (varsayılan: registry.orion.app)
```

---

## Özet — Geliştirici İş Akışı

```
İlk kurulum (bir kez):
  opk keygen
  opk account register
  opk key add

Her yayında:
  opk pack ./dist
  opk sign myapp.opk
  opk verify myapp.opk    ← kendi kendini doğrula
  opk publish myapp.opk

Problem olursa:
  opk yank <id> <versiyon> --reason "..."
```

---

*Bu doküman Orion ekosistemi dahilinde geliştirilmektedir.*  
*Tüm belgeler: `opk-format-spec.md`, `opk-registry-api.md`, `opk-bekci-arch.md`, `opk-install-pipeline.md`, `opk-sandbox-impl.md`, `opk-signing-guide.md`*
