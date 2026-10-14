# `.opk` Paket Yaşam Döngüsü — Teknik Spesifikasyon

> **Versiyon:** 1.0.0-draft  
> **Modül:** `mcpkg-pipeline`, `mcpkg-cli`  
> **Durum:** Geliştirme Aşaması  
> **Hedef Platform:** Orion OS (Arch Linux tabanlı)

---

## İçindekiler

1. [Genel Bakış](#genel-bakış)
2. [Yerel Paket Kayıt Defteri (`installed.db`)](#yerel-paket-kayıt-defteri-installeddb)
3. [Kaldırma Akışı (`opk uninstall`)](#kaldırma-akışı-opk-uninstall)
4. [Listeleme Akışı (`opk list`)](#listeleme-akışı-opk-list)
5. [Bilgi Görüntüleme (`opk info`)](#bilgi-görüntüleme-opk-info)
6. [Güncelleme Akışı (`opk update`)](#güncelleme-akışı-opk-update)
7. [Hata ve Rollback Durumları](#hata-ve-rollback-durumları)

---

## Genel Bakış

Bir `.opk` uygulamasının yaşam döngüsü kurulum ile bitmez. Sistem kararlılığı ve kullanıcı güvenliği için uygulamaların kaldırılması, güncellenmesi, durumlarının sorgulanması ve kaynaklarının temizlenmesi standart kurallara bağlanmalıdır.

```
       ┌───────────┐
       │   Store   │ (veya yerel .opk)
       └─────┬─────┘
             │ opk install
             ▼
       ┌───────────┐         opk update
       │  Kurulu   │ ◄─────────────────────────┐
       │ (Active)  │ ─────────────────────────┘
       └─────┬─────┘
             │ opk uninstall
             ▼
       ┌───────────┐
       │ Silindi   │ (Kullanıcı verileri korunur)
       └───────────┘
```

---

## Yerel Paket Kayıt Defteri (`installed.db`)

Sistemde kurulu olan tüm uygulamaların metadata ve durum bilgileri yerel SQLite veritabanında saklanır.

* **Konum:** `/opt/orion/registry/installed.db` (root) veya `~/.local/share/orion/registry/installed.db` (rootless)

### Veritabanı Şeması

```sql
CREATE TABLE IF NOT EXISTS installed_packages (
    id                  TEXT PRIMARY KEY,       -- örn: com.orion.notepad
    name                TEXT NOT NULL,          -- Orion Notepad
    version             TEXT NOT NULL,          -- 1.2.3
    installed_at        TEXT NOT NULL,          -- ISO 8601 zaman damgası
    updated_at          TEXT,                   -- Son güncelleme zamanı
    install_path        TEXT NOT NULL,          -- /opt/orion/apps/com.orion.notepad
    desktop_file        TEXT,                   -- ~/.local/share/applications/com.orion.notepad.desktop
    icon_file           TEXT,                   -- ~/.local/share/icons/orion/com.orion.notepad.png
    channel             TEXT DEFAULT 'stable',  -- stable | beta | nightly
    status              TEXT DEFAULT 'active',  -- active | broken | updating
    size_bytes          INTEGER NOT NULL,       -- Toplam disk boyutu
    sha256              TEXT NOT NULL           -- Paket sağlama toplamı
);
```

---

## Kaldırma Akışı (`opk uninstall`)

Kaldırma işlemi atomik olmalı ve sistemde yetkisiz artık bırakmamalıdır.

```bash
opk uninstall <app_id> [--purge]
```

### Kaldırma Adımları

1. **Paket Varlık Kontrolü:** `installed.db` ve disk üzerindeki uygulama dizini kontrol edilir.
2. **Çalışan Süreçleri Sonlandırma:**
   - İlgili uygulamanın PID namespace veya cgroup altındaki tüm süreçleri tespit edilir.
   - Önce `SIGTERM` gönderilir, 5 saniye içinde kapanmazsa `SIGKILL` ile zorla sonlandırılır.
3. **AppArmor Profilini Kaldırma:**
   - `apparmor_parser -R /etc/apparmor.d/orion/{app_id}` çağrısı ile çekirdek LSM profili kaldırılır.
4. **cgroups Temizliği:**
   - `/sys/fs/cgroup/orion/apps/{app_id}` dizini silinir.
5. **Uygulama Dosyalarını Silme:**
   - `/opt/orion/apps/{app_id}/` (veya ilgili kullanıcı dizini) tamamen silinir.
6. **Masaüstü ve İkon Temizliği:**
   - `~/.local/share/applications/{app_id}.desktop` silinir.
   - `~/.local/share/icons/orion/{app_id}.png` silinir.
7. **Veritabanı Güncelleme:**
   - `installed.db` tablosundan paket kaydı silinir.
8. **Denetim Günlüğü Yazma:**
   - `audit.db` tablosuna `action = "remove"` kaydı eklenir.
9. **Kullanıcı Verisi Politikası:**
   - `~/.local/share/{app_id}` ve `~/.config/{app_id}` dizinlerine varsayılan olarak **DOKUNULMAZ**.
   - Sadece `--purge` bayrağı verilmişse ve kullanıcı onaylarsa bu dizinler de silinir.

---

## Listeleme Akışı (`opk list`)

Sistemde kurulu olan tüm uygulamaları okunabilir bir tabloda listeler.

```bash
opk list [--json] [--updates]
```

### Örnek Çıktı

```
ID                   AD             VERSİYON   KANAL    BOYUT    KURULUM TARİHİ
com.orion.notepad    Orion Notepad  1.2.3      stable   14.2 MB  2026-04-10
app.mycompany.tool   Dev Tool       0.9.1      beta     38.0 MB  2026-04-12
```

`--updates` bayrağı verildiğinde her paket için Registry API sorgulanır ve güncellenebilir sürümler sarı ile işaretlenir.

---

## Bilgi Görüntüleme (`opk info`)

Yerel bir `.opk` arşivinin veya kurulu bir uygulamanın detaylarını gösterir.

```bash
opk info com.orion.notepad
opk info ./downloads/app-1.0.0.opk
```

### Gösterilecek Bilgiler

* **Kimlik:** Ad, ID, Sürüm, Kategori, Lisans, Yazar ve İletişim
* **İzinler (Policy):** Dosya sistemi, ağ ve donanım izinleri (izin/red/sor)
* **Risk Analizi:** Bekçi risk puanı ve risk düzeyi
* **İmza:** Ed25519 parmak izi ve güven seviyesi (`Verified` / `Signed` / `Unsigned`)
* **Mimari:** Desteklenen platformlar (`x86_64`, `aarch64`)

---

## Güncelleme Akışı (`opk update`)

Güncellemeler veri kaybını önlemek için otomatik yedekleme ve geri alma ile çalışır.

```bash
opk update [app_id]
```

### Güncelleme Durum Makinesi

```
Mevcut Sürüm
     │
     ▼
Yedekle: /opt/orion/apps/{id}/.backup/
     │
     ▼
Yeni .opk'yı indir ve staging'e aç
     │
     ▼
İmza + Checksum Doğrula ──(HATA)──► Geri Yükle ve Durdur
     │
     ▼
pre-install.sh çalıştır
     │
     ▼
Atomik rename: staging → target
     │
     ▼
post-install.sh çalıştır ──(HATA)──► .backup'ı geri taşı
     │
     ▼
Başarılı ──► .backup'ı temizle, installed.db güncelle, audit log yaz
```

---

## Hata ve Rollback Durumları

| Hata Noktası | Yapılacak İşlem |
|---|---|
| Yeni ikili bozuk | Staging silinir, eski sürüm çalışmaya devam eder |
| `post-install.sh` çöktü | Yeni dizin silinir, `.backup` hedef konuma geri taşınır |
| Süreç durdurulamadı | Zorla `SIGKILL` denenir, 10 saniye sonra başarısız sayılır |
