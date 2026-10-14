# CLI Referansı — Orion Package Kit (`opk`)

> **Komut Adı:** `opk`  
> **Açıklama:** Orion OS Resmi Paket Yöneticisi ve Geliştirici CLI Aracı  
> **Konum:** `docs/CLI_REFERENCE.md`

---

## Genel Kullanım Sözdizimi

```bash
opk <ALT_KOMUT> [SEÇENEKLER] [ARGÜMANLAR]
```

### Genel Bayraklar

* `-h, --help`: Yardım bilgilerini görüntüler.
* `-V, --version`: `opk` sürümünü görüntüler.

---

## Komut Listesi

### 1. `install` — Paket Kurulumu

Yerel bir `.opk` arşiv dosyasından veya uzak paket deposundan (Registry) uygulama kurar.

```bash
opk install <path_or_id> [SEÇENEKLER]
```

* **Argümanlar:**
  * `<path_or_id>`: Yerel dosya yolu (örn. `./com.orion.notepad-1.0.0.opk`) ya da paket ID'si (örn. `com.orion.notepad`).
* **Seçenekler:**
  * `-y, --yes`: İzin onaylarını ve Bekçi risk uyarılarını otomatik onaylar (etkileşimsiz mod).
  * `-v, --version <VERSION>`: Kurulacak belirli sürümü belirtir (Varsayılan: En son kararlı sürüm).
  * `-r, --registry <URL>`: Özel bir paket kütüphanesi URL'si belirtir (Test için: `mock://test_api`).
* **Örnekler:**
  ```bash
  opk install ./downloads/com.orion.notepad-1.0.0.opk
  opk install com.orion.notepad -y
  opk install com.orion.notepad --version 1.2.0
  ```

---

### 2. `launch` — Uygulama Başlatma

Kurulu bir uygulamayı izole Linux sandbox'ı içinde çalıştırır.

```bash
opk launch <app_id>
```

* **Argümanlar:**
  * `<app_id>`: Çalıştırılacak uygulamanın kimliği (örn. `com.orion.notepad`).
* **Örnek:**
  ```bash
  opk launch com.orion.notepad
  ```

---

### 3. `uninstall` — Uygulama Kaldırma

Kurulu bir uygulamayı ve sistemdeki tüm kısayollarını güvenle kaldırır.

```bash
opk uninstall <app_id> [SEÇENEKLER]
```

* **Argümanlar:**
  * `<app_id>`: Kaldırılacak uygulamanın kimliği.
* **Seçenekler:**
  * `--purge`: Uygulamanın `~/.local/share/` ve `~/.config/` altındaki kullanıcı verilerini de siler.
* **Örnek:**
  ```bash
  opk uninstall com.orion.notepad
  opk uninstall com.orion.notepad --purge
  ```

---

### 4. `list` — Kurulu Uygulamaları Listeleme

Sistemde kurulu olan tüm `.opk` uygulamalarını listeler.

```bash
opk list [SEÇENEKLER]
```

* **Seçenekler:**
  * `--json`: Çıktıyı makine tarafından okunabilir JSON formatında verir.
  * `--updates`: Güncellemesi bulunan uygulamaları kontrol eder.
* **Örnek:**
  ```bash
  opk list
  opk list --json
  ```

---

### 5. `info` — Paket Detaylarını İnceleme

Kurulu bir uygulamanın veya yerel bir `.opk` arşivinin metadata, izin ve imza detaylarını görüntüler.

```bash
opk info <path_or_id>
```

* **Örnek:**
  ```bash
  opk info com.orion.notepad
  opk info ./builds/myapp-1.0.0.opk
  ```

---

### 6. `update` — Uygulama Güncelleme

Kurulu uygulamaların yeni sürümlerini denetler ve günceller.

```bash
opk update [app_id]
```

* **Örnek:**
  ```bash
  opk update                     # Tüm uygulamaları günceller
  opk update com.orion.notepad   # Belirli bir uygulamayı günceller
  ```

---

### 7. `pack` — Paket Arşivi Oluşturma

Bir proje kaynak dizinini tarar, `checksums.sha256` üretir ve `.opk` paketi olarak paketler.

```bash
opk pack <source_dir> [SEÇENEKLER]
```

* **Argümanlar:**
  * `<source_dir>`: `manifest.toml` ve `sandbox.policy` içeren kaynak dizin.
* **Seçenekler:**
  * `-o, --output <PATH>`: Çıktı `.opk` dosyasının kaydedileceği yol (Varsayılan: `{id}-{version}.opk`).
* **Örnek:**
  ```bash
  opk pack ./my_project
  opk pack ./my_project -o ./builds/myapp.opk
  ```

---

### 8. `sign` — Paket İmzalama

Oluşturulmuş bir `.opk` paketini geliştirici özel anahtarı ile dijital olarak imzalar.

```bash
opk sign <package> [SEÇENEKLER]
```

* **Argümanlar:**
  * `<package>`: İmzalanacak `.opk` dosyasının yolu.
* **Seçenekler:**
  * `-k, --key <PATH>`: Özel anahtar yolu (Varsayılan: `~/.config/orion/signing/private.key`).
* **Örnek:**
  ```bash
  opk sign ./myapp-1.0.0.opk
  opk sign ./myapp-1.0.0.opk -k ./keys/custom_private.key
  ```

---

### 9. `verify` — Bütünlük ve İmza Doğrulama

Bir paketin SHA-256 sağlama toplamlarını ve Ed25519 imzasını kurmadan önce yerel olarak doğrular.

```bash
opk verify <path> [SEÇENEKLER]
```

* **Argümanlar:**
  * `<path>`: Doğrulanacak `.opk` dosyası.
* **Seçenekler:**
  * `-k, --key <PATH>`: Doğrulama için kullanılacak açık anahtar yolu.
* **Örnek:**
  ```bash
  opk verify ./myapp-1.0.0.opk
  opk verify ./myapp-1.0.0.opk -k ./keys/custom_public.key
  ```

---

### 10. `keygen` — İmza Anahtarı Üretimi

Paketleri imzalamak için yeni bir Ed25519 anahtar çifti oluşturur.

```bash
opk keygen [SEÇENEKLER]
```

* **Seçenekler:**
  * `-o, --output <DIR>`: Anahtarların yazılacağı dizin (Varsayılan: `~/.config/orion/signing/`).
* **Örnek:**
  ```bash
  opk keygen
  opk keygen -o ./my_keys
  ```

---

### 11. `publish` — Paketi Mağazaya / Registry'ye Yükleme

İmzalanmış bir paketi uzak Orion Registry sunucusuna yayınlar.

```bash
opk publish <package> [SEÇENEKLER]
```

* **Ortam Değişkeni:** `ORION_API_KEY` (veya `MCSIMON_API_KEY`)
* **Seçenekler:**
  * `-c, --channel <CHANNEL>`: Yayın kanalı (`stable`, `beta`, `nightly`) (Varsayılan: `stable`).
  * `-r, --registry <URL>`: Özel registry URL'si.
* **Örnek:**
  ```bash
  export ORION_API_KEY="orion_live_secret_key"
  opk publish ./myapp-1.0.0.opk --channel stable
  ```

---

### 12. `search` — Paket Arama

Orion Registry üzerinde anahtar kelime veya paket adına göre arama yapar.

```bash
opk search <query> [SEÇENEKLER]
```

* **Argümanlar:**
  * `<query>`: Aranacak paket adı, etiket veya açıklama kelimesi.
* **Seçenekler:**
  * `--json`: Sonuçları makine tarafından okunabilir JSON formatında listeler.
  * `-r, --registry <URL>`: Özel paket deposu URL'si (Varsayılan: `https://registry.orion.org` veya `mock://test_api`).
* **Örnek:**
  ```bash
  opk search editor
  opk search editor --json
  ```

---

### 13. `audit` — Orion Bekçi Denetim Günlüğü

Sistemdeki tüm kurulum, güncelleme, kaldırma ve doğrulama eylemlerinin güvenlik kayıtlarını inceler.

```bash
opk audit <ALT_KOMUT> [SEÇENEKLER]
```

* **Alt Komutlar:**
  * `list`: Son denetim kayıtlarını listeler (`-n, --limit <LIMIT>` seçeneğiyle sınırlandırılabilir).
  * `show <id>`: Belirtilen kayıt kimliğinin detaylı güvenlik ve risk analiz raporunu görüntüler.
  * `export`: Denetim günlüğünü dışa aktarır (`-f, --format <csv|json>`, `-o, --output <DOSYA>`).
* **Örnekler:**
  ```bash
  opk audit list
  opk audit list -n 20
  opk audit show 1
  opk audit export -f json -o audit_export.json
  opk audit export -f csv -o audit_export.csv
  ```

---

### 14. `daemon` — Bekçi IPC Arka Plan Servisi

Orion masaüstü ortamı veya sistem servisleriyle IPC üzerinden haberleşen Bekçi (`bekci-d`) daemon'ını başlatır.

```bash
opk daemon [SEÇENEKLER]
```

* **Seçenekler:**
  * `-s, --socket <PATH>`: Özel Unix Domain Socket yolu (Varsayılan: `~/.local/run/orion/bekci.sock`).
* **Örnek:**
  ```bash
  opk daemon
  opk daemon --socket /tmp/bekci.sock
  ```

