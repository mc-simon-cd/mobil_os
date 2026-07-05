# Bekçi (Gatekeeper) Mimarisi — Teknik Spesifikasyon

> **Versiyon:** 1.0.0-draft  
> **Crate:** `bekci` (Rust)  
> **Durum:** Tasarım Aşaması  
> **Felsefe:** İnform, block değil — kullanıcı karar verir.

---

## İçindekiler

1. [Felsefe ve Tasarım Kararları](#felsefe-ve-tasarım-kararları)
2. [Bileşen Mimarisi](#bileşen-mimarisi)
3. [Ed25519 İmza Doğrulama](#ed25519-i̇mza-doğrulama)
4. [Güven Seviyeleri](#güven-seviyeleri)
5. [Politika Motoru](#politika-motoru)
6. [Uyarı Sistemi](#uyarı-sistemi)
7. [Denetim Günlüğü](#denetim-günlüğü)
8. [Rust Crate Yapısı](#rust-crate-yapısı)
9. [CLI Arayüzü](#cli-arayüzü)
10. [IPC Protokolü](#ipc-protokolü)

---

## Felsefe ve Tasarım Kararları

### Temel Prensip: İnform, Block Değil

Bekçi bir kapı bekçisi değil, bir **danışman**dır.

```
Apple/Google modeli:          Bekçi modeli:
─────────────────────         ─────────────
Şüpheli paket → BLOCK         Şüpheli paket → UYAR → kullanıcı seçer
Kurallar gizli                Kurallar şeffaf
Geliştirici başvurur          Geliştirici kodu açık inceleyebilir
Hata yoksa bile reddedebilir  Sadece teknik sorunları raporlar
```

### Ne Zaman Bloklar?

Bekçi yalnızca **kriptografik bütünlük ihlali** durumunda zorla durdurur:

```
checksums.sha256 uyuşmazlığı → ZORUNLU DURDUR (dosya değiştirilmiş)
```

Diğer her durumda kullanıcıya sorar. Kullanıcı "devam et" derse kurulum ilerler ve bu karar loglanır.

---

## Bileşen Mimarisi

```
┌─────────────────────────────────────────────────────┐
│                    bekci crate                      │
│                                                     │
│  ┌────────────┐   ┌──────────────┐   ┌───────────┐ │
│  │  Verifier  │   │Policy Engine │   │  Logger   │ │
│  │            │   │              │   │           │ │
│  │ Ed25519    │   │ Permission   │   │ Audit log │ │
│  │ Checksum   │   │ Risk scoring │   │ SQLite    │ │
│  │ Manifest   │   │ Warn levels  │   │           │ │
│  └─────┬──────┘   └──────┬───────┘   └─────┬─────┘ │
│        └────────────┬────┘                 │       │
│                     ▼                      │       │
│              ┌─────────────┐               │       │
│              │  Bekci Core │───────────────┘       │
│              │             │                       │
│              │  Result<>   │                       │
│              │  Decision   │                       │
│              └──────┬──────┘                       │
└─────────────────────┼───────────────────────────── ┘
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
      [CLI tool]  [IPC daemon] [GUI widget]
      bekci-cli   bekci-d      Store UI
```

---

## Ed25519 İmza Doğrulama

### Bağımlılık

```toml
# Cargo.toml
[dependencies]
ed25519-dalek = { version = "2", features = ["std"] }
sha2 = "0.10"
```

### İmzalanan Veri Yapısı

```
imza_girdisi = SHA256(manifest.toml)
             ‖ SHA256(sandbox.policy)
             ‖ SHA256(checksums.sha256)
             ‖ SHA256(binary_x86_64)     # varsa
             ‖ SHA256(binary_aarch64)    # varsa
```

`‖` = byte dizisi olarak birleştirme (concatenation), sıra sabit.

### Rust Implementasyonu

```rust
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use sha2::{Sha256, Digest};

pub struct SignatureVerifier {
    trusted_keys: Vec<VerifyingKey>,
}

impl SignatureVerifier {
    /// Registry'den indirilen public key'leri yükler.
    pub fn from_registry(keys: Vec<[u8; 32]>) -> Result<Self, BekciError> {
        let trusted_keys = keys
            .into_iter()
            .map(|bytes| VerifyingKey::from_bytes(&bytes))
            .collect::<Result<Vec<_>, _>>()
            .map_err(BekciError::InvalidKey)?;
        Ok(Self { trusted_keys })
    }

    /// .opk dosyasındaki imzayı doğrular.
    pub fn verify(&self, package: &McpkgPackage) -> VerificationResult {
        let message = self.build_message(package);
        let sig_bytes = &package.signature_bytes; // 64 byte

        let signature = match Signature::from_slice(sig_bytes) {
            Ok(s) => s,
            Err(_) => return VerificationResult::InvalidSignatureFormat,
        };

        for key in &self.trusted_keys {
            if key.verify(&message, &signature).is_ok() {
                return VerificationResult::Verified {
                    key_fingerprint: hex::encode(key.as_bytes()),
                };
            }
        }

        VerificationResult::SignatureMismatch
    }

    fn build_message(&self, pkg: &McpkgPackage) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&sha256(&pkg.manifest_bytes));
        msg.extend_from_slice(&sha256(&pkg.policy_bytes));
        msg.extend_from_slice(&sha256(&pkg.checksums_bytes));
        if let Some(bin) = &pkg.binary_x86_64 {
            msg.extend_from_slice(&sha256(bin));
        }
        if let Some(bin) = &pkg.binary_aarch64 {
            msg.extend_from_slice(&sha256(bin));
        }
        msg
    }
}

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
```

---

## Güven Seviyeleri

### Seviye Belirleme Algoritması

```rust
pub enum TrustLevel {
    Verified,   // Orion + geliştirici imzası, her ikisi geçerli
    Signed,     // Geliştirici imzası geçerli, Orion doğrulamamış
    Unsigned,   // İmza yok veya geçersiz
}

impl TrustLevel {
    pub fn determine(
        developer_sig: VerificationResult,
        orion_approved: bool,
    ) -> Self {
        match (developer_sig, orion_approved) {
            (VerificationResult::Verified { .. }, true)  => TrustLevel::Verified,
            (VerificationResult::Verified { .. }, false) => TrustLevel::Signed,
            _ => TrustLevel::Unsigned,
        }
    }
}
```

### Kullanıcıya Gösterilen Bilgi

```
┌─────────────────────────────────────────────┐
│  🟢 Doğrulanmış Uygulama                    │
│  "com.orion.notepad" v1.2.3               │
│  Geliştirici: Mustafa                       │
│  Orion Store tarafından doğrulanmıştır.   │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│  🟡 İmzalı Uygulama                         │
│  "com.example.tool" v0.5.0                  │
│  Geliştirici: Bilinmeyen                    │
│  ⚠️ Orion Store tarafından henüz          │
│  incelenmemiştir. Devam etmek ister misiniz?│
│  [İptal]  [Devam Et]                        │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│  🔴 İmzasız Uygulama                        │
│  "unknown-app" v?                           │
│  ⛔ Bu uygulamanın kim tarafından           │
│  hazırlandığı doğrulanamıyor.               │
│  [İptal]  [Riski Anlıyorum, Devam Et]       │
└─────────────────────────────────────────────┘
```

---

## Politika Motoru

### Risk Puanlama

Her `sandbox.policy` dosyası bir risk skoru üretir.

```rust
pub struct RiskScore {
    pub total: u32,           // 0–100 arası
    pub level: RiskLevel,
    pub flags: Vec<RiskFlag>,
}

pub enum RiskLevel {
    Low,        // 0–25
    Medium,     // 26–50
    High,       // 51–75
    Critical,   // 76–100
}

pub struct RiskFlag {
    pub permission: String,
    pub value: String,
    pub weight: u32,
    pub message: String,
}
```

### Puanlama Tablosu

| İzin | Değer | Puan | Mesaj |
|---|---|---|---|
| `network.internet` | `allow` | +10 | İnternet erişimi istiyor |
| `filesystem.system_read` | `allow` | +25 | Sistem dosyalarına erişim |
| `filesystem.arbitrary_path` | `allow` | +30 | Rastgele dosya erişimi |
| `system.exec_subprocess` | `allow` | +20 | Alt süreç başlatabilir |
| `system.autostart` | `allow` | +15 | Sistem başlangıcında çalışır |
| `hardware.camera` | `allow` | +10 | Kameraya erişim |
| `hardware.microphone` | `allow` | +10 | Mikrofona erişim |
| `system.ipc` | `allow` | +15 | Diğer uygulamalarla iletişim |
| `network.internet` | `deny` | -5 | İnternet yok, daha güvenli |

### Rust Implementasyonu

```rust
pub fn score_policy(policy: &SandboxPolicy) -> RiskScore {
    let mut flags = Vec::new();
    let mut total: u32 = 0;

    if policy.network.internet == Permission::Allow {
        total += 10;
        flags.push(RiskFlag {
            permission: "network.internet".into(),
            value: "allow".into(),
            weight: 10,
            message: "İnternet erişimi istiyor".into(),
        });
    }

    if policy.filesystem.arbitrary_path == Permission::Allow {
        total += 30;
        flags.push(RiskFlag {
            permission: "filesystem.arbitrary_path".into(),
            value: "allow".into(),
            weight: 30,
            message: "Rastgele dosya sistemi erişimi — dikkatli olun".into(),
        });
    }

    // ... diğer izinler

    let level = match total {
        0..=25   => RiskLevel::Low,
        26..=50  => RiskLevel::Medium,
        51..=75  => RiskLevel::High,
        _        => RiskLevel::Critical,
    };

    RiskScore { total, level, flags }
}
```

---

## Uyarı Sistemi

### Uyarı Seviyeleri

```rust
pub enum WarnLevel {
    Info,       // Bilgi amaçlı, onay gerektirmez
    Warning,    // Kullanıcıya sorar, devam edebilir
    Critical,   // Güçlü uyarı, "riski anlıyorum" onayı
    Block,      // Yalnızca checksum uyuşmazlığında — durdurulamaz
}
```

### Uyarı Tetikleyicileri

| Durum | Seviye | Mesaj |
|---|---|---|
| `TrustLevel::Signed` | `Warning` | Orion tarafından incelenmemiş |
| `TrustLevel::Unsigned` | `Critical` | Kimlik doğrulanamıyor |
| `RiskLevel::High` | `Warning` | Yüksek riskli izinler var |
| `RiskLevel::Critical` | `Critical` | Kritik izinler isteniyor |
| Checksum uyuşmazlığı | `Block` | Dosya bütünlüğü bozuk |
| `rating == "18+"` | `Warning` | Yetişkin içeriği |
| `min_os > current_os` | `Info` | OS versiyonu eski |
| Yank edilmiş versiyon | `Warning` | Bu versiyon geri çekilmiş |

---

## Denetim Günlüğü

Her bekçi kararı yerel SQLite veritabanına yazılır.

```
~/.local/share/orion/bekci/audit.db
```

### Şema

```sql
CREATE TABLE audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   TEXT NOT NULL,           -- ISO 8601
    action      TEXT NOT NULL,           -- install | update | remove | yank_bypass
    package_id  TEXT NOT NULL,
    version     TEXT NOT NULL,
    trust_level TEXT NOT NULL,           -- verified | signed | unsigned
    risk_level  TEXT NOT NULL,           -- low | medium | high | critical
    risk_score  INTEGER NOT NULL,
    user_choice TEXT NOT NULL,           -- approved | cancelled | forced
    warnings    TEXT,                    -- JSON array
    checksum_ok INTEGER NOT NULL,        -- 0 | 1
    sig_ok      INTEGER NOT NULL,        -- 0 | 1
    key_fp      TEXT                     -- imza parmak izi
);
```

### Örnek Kayıt

```json
{
  "id": 42,
  "timestamp": "2026-04-15T12:30:00Z",
  "action": "install",
  "package_id": "com.orion.notepad",
  "version": "1.2.3",
  "trust_level": "verified",
  "risk_level": "low",
  "risk_score": 10,
  "user_choice": "approved",
  "warnings": ["İnternet erişimi istiyor"],
  "checksum_ok": 1,
  "sig_ok": 1,
  "key_fp": "SHA256:abc123..."
}
```

---

## Rust Crate Yapısı

```
bekci/
├── Cargo.toml
├── src/
│   ├── lib.rs              ← Public API
│   ├── verifier/
│   │   ├── mod.rs
│   │   ├── signature.rs    ← Ed25519 doğrulama
│   │   └── checksum.rs     ← SHA256 bütünlük kontrolü
│   ├── policy/
│   │   ├── mod.rs
│   │   ├── parser.rs       ← sandbox.policy parse
│   │   └── scorer.rs       ← Risk puanlama
│   ├── manifest/
│   │   ├── mod.rs
│   │   └── parser.rs       ← manifest.toml parse + validate
│   ├── decision/
│   │   ├── mod.rs
│   │   └── engine.rs       ← Karar motoru
│   ├── audit/
│   │   ├── mod.rs
│   │   └── sqlite.rs       ← Denetim günlüğü
│   └── error.rs            ← BekciError enum
├── benches/
│   └── verify_bench.rs
└── tests/
    ├── signature_tests.rs
    └── policy_tests.rs
```

### Public API

```rust
// lib.rs
pub use crate::verifier::{SignatureVerifier, VerificationResult};
pub use crate::policy::{PolicyEngine, RiskScore, RiskLevel};
pub use crate::decision::{BekciDecision, WarnLevel};
pub use crate::error::BekciError;

/// Ana bekçi değerlendirme fonksiyonu.
/// Registry API'den gelen public key'lerle paketi doğrular ve karar üretir.
pub fn evaluate(
    package_path: &Path,
    trusted_keys: Vec<[u8; 32]>,
    orion_approved: bool,
) -> Result<BekciDecision, BekciError> {
    // 1. Paketi aç (ZIP)
    // 2. Checksum doğrula (zorunlu)
    // 3. İmza doğrula
    // 4. Manifest parse et
    // 5. Policy parse et ve puanla
    // 6. Güven seviyesi belirle
    // 7. Uyarıları derle
    // 8. BekciDecision döndür
    todo!()
}

pub struct BekciDecision {
    pub trust_level: TrustLevel,
    pub risk_score: RiskScore,
    pub warnings: Vec<Warning>,
    pub require_user_confirmation: bool,
    pub hard_block: bool,       // Yalnızca checksum ihlalinde true
}
```

---

## CLI Arayüzü

`bekci-cli` — geliştirici ve ileri kullanıcılar için.

```bash
# Bir paketi doğrula (kurmadan)
bekci verify com.orion.notepad-1.2.3.opk

# Çıktı:
# ✅ Checksum: Geçerli
# ✅ İmza: Doğrulandı (SHA256:abc123...)
# 🟢 Güven Seviyesi: Verified
# 📊 Risk Skoru: 10/100 (Düşük)
# ⚠️  Uyarılar:
#    - İnternet erişimi istiyor

# Ayrıntılı JSON çıktı
bekci verify --json com.orion.notepad-1.2.3.opk

# Denetim günlüğünü görüntüle
bekci audit list
bekci audit list --package com.orion.notepad
bekci audit export --format csv > audit.csv

# Policy dosyasını bağımsız değerlendir
bekci policy check sandbox.policy

# Güven seviyelerini sorgula
bekci trust list
```

---

## IPC Protokolü

`bekci-d` daemon, Store UI ve paket yöneticisiyle Unix domain socket üzerinden haberleşir.

```
~/.local/run/orion/bekci.sock
```

### Mesaj Formatı

```json
// İstek
{
  "id": "req_01HX...",
  "action": "evaluate",
  "package_path": "/tmp/com.orion.notepad-1.2.3.opk"
}

// Yanıt
{
  "id": "req_01HX...",
  "decision": {
    "trust_level": "verified",
    "risk_score": 10,
    "risk_level": "low",
    "warnings": ["İnternet erişimi istiyor"],
    "require_user_confirmation": false,
    "hard_block": false
  }
}
```

---

*Bu doküman Orion ekosistemi dahilinde geliştirilmektedir.*  
*Sonraki: `opk-install-pipeline.md`*
