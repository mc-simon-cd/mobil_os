# Kurulum Pipeline — Teknik Spesifikasyon

> **Versiyon:** 1.0.0-draft  
> **Hedef:** Orion OS (Arch Linux tabanlı, kernel 6.x+)  
> **Durum:** Tasarım Aşaması

---

## İçindekiler

1. [Pipeline Genel Bakış](#pipeline-genel-bakış)
2. [Aşamalar](#aşamalar)
3. [Hata ve Geri Alma](#hata-ve-geri-alma)
4. [Güncelleme ve Kaldırma](#güncelleme-ve-kaldırma)
5. [Dosya Sistemi Düzeni](#dosya-sistemi-düzeni)
6. [State Makinesi](#state-makinesi)

---

## Pipeline Genel Bakış

```
Kullanıcı "Kur" der
        │
        ▼
  ① İndir + Önbellek Kontrolü
        │
        ▼
  ② Checksum Doğrula
        │ FAIL → DURDUR (bütünlük ihlali)
        ▼
  ③ Bekçi Değerlendirmesi
        │ WARN → Kullanıcıya Sor → İptal veya Devam
        ▼
  ④ İzin Ekranı Göster
        │ Kullanıcı Onayı
        ▼
  ⑤ Sandbox Container Hazırla
        │
        ▼
  ⑥ pre-install.sh Çalıştır (varsa)
        │
        ▼
  ⑦ Dosyaları Yerleştir
        │
        ▼
  ⑧ post-install.sh Çalıştır (varsa)
        │
        ▼
  ⑨ Denetim Kaydı Yaz
        │
        ▼
  ⑩ ✅ Kurulum Tamamlandı
```

---

## Aşamalar

---

### ① İndir + Önbellek Kontrolü

```
Önce cache kontrol et:
  ~/.cache/orion/packages/{id}/{version}.opk → varsa atla

Yoksa:
  CDN'den indir → geçici dosyaya yaz
  /tmp/orion-install-{id}-{version}.opk

İndirme sırasında:
  - İlerleme çubuğu göster (bayt bazlı)
  - Timeout: 5 dakika
  - Yeniden deneme: 3 kez (exponential backoff)
```

**İndirme URL şeması:**

```
https://cdn.orion.app/packages/{id}/{version}/{id}-{version}.opk
```

---

### ② Checksum Doğrula

```
checksums.sha256 dosyasını oku
Her dosya için:
  SHA256 hesapla → beklenenle karşılaştır
  UYUŞMAZLIK → pipeline DURDUR, kullanıcıya hata göster

Bu adım bypass edilemez.
```

---

### ③ Bekçi Değerlendirmesi

`bekci evaluate` çağrısı. Karar:

```
hard_block = true   → pipeline DURDUR
warning(lar) var    → kullanıcıya uyarı ekranı göster
                      [İptal] [Devam Et]
temiz               → doğrudan devam
```

Kullanıcı kararı denetim günlüğüne yazılır.

---

### ④ İzin Ekranı

Her kurulumda gösterilir. Basit, anlaşılır dil.

```
┌──────────────────────────────────────────────────┐
│  Orion Notepad bu izinleri istiyor:            │
│                                                  │
│  ✅ İnternet erişimi                            │
│  ✅ Ev klasörüne okuma/yazma (~/ altında)       │
│  ❓ Belgeler klasörüne erişim (sizi soracak)    │
│  ❌ Kamera — İstemez                            │
│  ❌ Mikrofon — İstemez                          │
│                                                  │
│  Risk Seviyesi: 🟡 Orta (35/100)               │
│                                                  │
│               [İptal]   [Kur]                   │
└──────────────────────────────────────────────────┘
```

---

### ⑤ Sandbox Container Hazırla

Linux kernel özellikleri kullanılarak izolasyon kurulur.

**Namespace oluşturma:**

```bash
# Mount namespace — kendi dosya sistemi görünümü
# Network namespace — ağ izolasyonu (policy'e göre)
# PID namespace — süreç izolasyonu
# User namespace — kök ayrıcalığı olmadan

unshare --mount --pid --user --net ...
```

**AppArmor profili oluşturma:**

```
/etc/apparmor.d/orion/{package_id}
```

`sandbox.policy` içeriğinden otomatik üretilir:

```
# /etc/apparmor.d/orion/com.orion.notepad
profile com.orion.notepad {
  /home/*/.local/share/com.orion.notepad/** rw,
  /tmp/orion-com.orion.notepad/** rw,
  network inet stream,       # internet = allow ise
  deny /etc/** rw,           # system_read = deny ise
  deny /usr/** w,
}
```

**seccomp filtresi:**

```
exec_subprocess = deny → execve, execveat sistem çağrıları engellenir
arbitrary_path = deny → open() çağrıları whitelist dışında reddedilir
```

**cgroups v2 limitleri:**

```
# İlk versiyon için basit limitler
cpu.max     = "50% üst sınır yok"
memory.max  = manifest'ten okunur (varsayılan: 1GB)
io.max      = sınırsız (ileride kısıtlanabilir)
```

---

### ⑥ pre-install.sh

Varsa çalıştırılır. Kısıtlamalar:

```
- Maksimum 60 saniye
- Sandbox içinde çalışır (kısıtlı izinlerle)
- Root erişimi yok
- Ağ erişimi yok (güvenlik)
- Çıkış kodu ≠ 0 → pipeline durdurur
```

Tipik kullanım:

```bash
#!/bin/bash
# pre-install.sh — Gerekli dizinleri oluştur
mkdir -p ~/.local/share/com.orion.notepad
mkdir -p ~/.config/com.orion.notepad
```

---

### ⑦ Dosyaları Yerleştir

```
binary       → /opt/orion/apps/{id}/bin/{binary}
assets       → /opt/orion/apps/{id}/assets/
manifest     → /opt/orion/apps/{id}/manifest.toml
policy       → /opt/orion/apps/{id}/sandbox.policy
desktop      → ~/.local/share/applications/{id}.desktop
icon         → ~/.local/share/icons/orion/{id}.png
```

**`.desktop` dosyası otomatik üretilir:**

```ini
[Desktop Entry]
Name=Orion Notepad
Exec=/opt/orion/launcher %u com.orion.notepad
Icon=orion-com.orion.notepad
Type=Application
Categories=Utility;
```

Uygulama doğrudan binary olarak değil, **launcher** aracılığıyla başlatılır — her çalıştırmada sandbox kurulur.

---

### ⑧ post-install.sh

Varsa çalıştırılır. Aynı kısıtlamalar geçerlidir.

Tipik kullanım:

```bash
#!/bin/bash
# post-install.sh — Veritabanı şemasını başlat
/opt/orion/apps/com.orion.notepad/bin/notepad --init-db
```

---

### ⑨ Denetim Kaydı

`bekci audit` SQLite'a yazılır:

```json
{
  "action": "install",
  "package_id": "com.orion.notepad",
  "version": "1.2.3",
  "trust_level": "verified",
  "risk_score": 35,
  "user_choice": "approved",
  "checksum_ok": true,
  "sig_ok": true
}
```

---

## Hata ve Geri Alma

Her aşamanın başında mevcut durum snapshot'ı alınır. Hata durumunda geri alınır.

### Geri Alma Sırası

```
Hata aşaması → Geri alma adımları:

post-install   → dosyaları kaldır, AppArmor profilini sil
Dosya yerleştir→ kopyalanan dosyaları sil
Container      → namespace ve cgroup'ları temizle
pre-install    → oluşturulan dizinleri sil (manifest'ten okur)
```

### Atomik Kurulum

Dosyalar önce geçici dizine yazılır, sonra tek seferde taşınır:

```
/tmp/orion-staging-{id}-{version}/  → kurulum hazır
    ↓ rename (atomic)
/opt/orion/apps/{id}/
```

`rename()` sistem çağrısı atomiktir — yarım kurulum olmaz.

---

## Güncelleme ve Kaldırma

### Güncelleme

```
Mevcut versiyon → /opt/orion/apps/{id}/.backup/ arşivle
Yeni versiyonu kur (aynı pipeline)
Başarılı → backup'ı sil
Başarısız → backup'tan geri yükle
```

Kullanıcı verisi (`~/.local/share/{id}/`) güncelleme sırasında dokunulmaz.

### Kaldırma

```
1. Çalışan süreçleri durdur
2. AppArmor profilini kaldır
3. cgroup'ları temizle
4. /opt/orion/apps/{id}/ sil
5. .desktop dosyasını sil
6. İkonu sil
7. Denetim kaydı yaz (action: "remove")
```

**Kullanıcı verisi silinmez** — kullanıcı ayrıca onay vermediği sürece:

```
~/.local/share/com.orion.notepad/   ← kalır
~/.config/com.orion.notepad/        ← kalır
```

---

## Dosya Sistemi Düzeni

```
/opt/orion/
├── apps/
│   └── com.orion.notepad/
│       ├── bin/
│       │   └── notepad
│       ├── assets/
│       │   └── icon.png
│       ├── manifest.toml
│       ├── sandbox.policy
│       └── .backup/            ← güncelleme sırasında geçici
│
├── launcher                    ← tüm uygulamaları sandbox'ta başlatır
└── registry/                   ← yerel paket veritabanı (SQLite)
    └── installed.db

~/.local/share/
└── com.orion.notepad/        ← uygulama kullanıcı verisi (dokunulmaz)

~/.config/
└── com.orion.notepad/        ← uygulama konfigürasyonu (dokunulmaz)

~/.cache/orion/
└── packages/
    └── com.orion.notepad/
        └── 1.2.3.opk         ← indirilen paket cache'i
```

---

## State Makinesi

```rust
pub enum InstallState {
    Idle,
    Downloading { progress: f32 },
    Verifying,
    AwaitingUserConfirmation { warnings: Vec<Warning> },
    PreparingContainer,
    RunningPreInstall,
    PlacingFiles,
    RunningPostInstall,
    WritingAuditLog,
    Completed,
    Failed { stage: String, error: String },
    RollingBack,
}
```

Her state geçişi UI'a bildirilir (IPC üzerinden). Store UI, progress ekranını bu state'e göre günceller.

---

*Bu doküman Orion ekosistemi dahilinde geliştirilmektedir.*  
*Sonraki: `opk-sandbox-impl.md`*
