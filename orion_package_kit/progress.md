# Project Progress — opk (Orion Package Kit)

## Milestone 1: Modular Foundation ✅
- [x] Workspace yapısı kuruldu (`core`, `crypto`, `sandbox`, `pipeline`, `cli`)
- [x] Temel TOML ayrıştırma (`manifest.toml`, `sandbox.policy`)
- [x] Ed25519 kriptografik ilkel fonksiyonları (`sign`, `verify`, `keygen`)
- [x] Linux namespace tespit ve temel çağrıları (`unshare`)
- [x] Temel kurulum state machine yapısı (`InstallState`)

## Milestone 2: Functional Sandbox (Tam Seviye) ✅
- [x] AppArmor profil üreticisi ve `apparmor_parser` çağrısı
- [x] Rootless kullanıcı namespace (UID/GID) eşlemesi
- [x] **Seccomp BPF Filtresi Entegrasyonu:** `libseccomp` bağımlılığı varsayılan (`default`) derleme özelliğine dahil edildi
- [x] **cgroups v2 Entegrasyonu:** CPU (`cpu.max`) ve RAM (`memory.max`) limitlerinin uygulanması ve yaşam döngüsü temizliği
- [x] **Gelişmiş Mount İzolasyonu:** Özel `/tmp/orion-{id}` bağlama ve salt-okunur (read-only) dosya sistemi kuralları

## Milestone 3: Production Pipeline & Kurulum ✅
- [x] ZIP arşiv açma ve dahili SHA-256 doğrulama
- [x] Atomik dosya taşıma ve hata anında rollback desteği
- [x] Temel `.desktop` dosyası üretimi
- [x] Kalıcı Kurulum Dizini: Standart yol (`/opt/orion/apps` veya `~/.local/share/orion/apps`)
- [x] İkon Entegrasyonu: `assets/icon.png` dosyasının `~/.local/share/icons/orion/` dizinine yerleştirilmesi
- [x] `TempDir::into_path` uyarısının `TempDir::keep()` ile düzeltilmesi
- [x] **Çoklu Mimari Desteği:** Host CPU mimarisine (`x86_64` vs `aarch64`) göre uygun binary'yi çalıştırma
- [x] **Bağımlılık ve Sürüm Denetimi:** `min_os`, `max_os` ve `dependencies` alanlarının kurulum anında doğrulanması

## Milestone 4: Store & Registry Entegrasyonu ✅
- [x] Registry API istemcisi (`get_package`, `download_package`, `publish_package`)
- [x] İndirme ve yerel önbellek yönetimi (`CacheManager`)
- [x] Çevrimdışı testler için `mock://` protokolü
- [x] **Paket Arama (`opk search`):** Registry arama API'si ve tablo / JSON çıktı desteği
- [x] **Geliştirici Anahtar Zinciri (Keyring):** Paket doğrulaması için geliştirici public key'lerinin Registry'den çözümlenmesi

## Milestone 5: Developer Tooling ✅
- [x] `opk pack` — dizinden `.opk` arşivi ve `checksums.sha256` üretme
- [x] `opk sign` — paketi özel anahtarla imzalama (`signature.ed25519`)
- [x] `opk verify` — paketin imza, Bekçi risk puanı ve sağlama toplamlarını doğrulama
- [x] `opk publish` — registry'ye paket yükleme
- [x] `opk keygen` — Ed25519 anahtar çifti üretimi

---

## 🚀 Paket Yaşam Döngüsü & Güvenlik Mimarisi

## Milestone 6: Paket Yaşam Döngüsü & Yönetimi (Lifecycle Management) ✅
- [x] **`opk uninstall <app_id>`:**
  - [x] Çalışan uygulama süreçlerinin tespit edilip sonlandırılması (`SIGTERM`)
  - [x] AppArmor profilinin çekirdekten kaldırılması
  - [x] cgroups v2 kaynak dizininin silinmesi
  - [x] Uygulama dizininin (`/opt/orion/apps/{id}` veya `~/.local/share/orion/apps/{id}`) silinmesi (kullanıcı verilerine dokunulmadan)
  - [x] `.desktop` ve ikon dosyalarının temizlenmesi
  - [x] `--purge` seçeneği ile kullanıcı verilerini (`~/.local/share/`, `~/.config/`) temizleme
- [x] **`opk list`:**
  - [x] Kurulu uygulamaların ID, isim, sürüm, kategori ve boyut bilgilerini listeleyen komut
  - [x] JSON formatında çıktı desteği (`--json`)
- [x] **`opk info <id | dosya.opk>`:**
  - [x] Paket metadata, izin listesi (`sandbox.policy`), mimari, giriş noktası ve kurulu olma durumunun gösterimi
  - [x] JSON formatında çıktı desteği (`--json`)
- [x] **`opk update [id]` & `opk upgrade`:**
  - [x] Registry üzerinden yeni sürüm denetimi ve otomatik indirme/kurulum mekanizması

## Milestone 7: Orion Bekçi (Gatekeeper) & SQLite Audit Log ✅
- [x] **SQLite Denetim Günlüğü:**
  - [x] `~/.local/share/orion/bekci/audit.db` veritabanı şeması ve `rusqlite` entegrasyonu
  - [x] Kurulum, güncelleme, kaldırma ve doğrulama eylemlerinin loglanması
  - [x] `opk audit list`, `opk audit show <id>` ve `opk audit export` komutları
- [x] **Politika ve Risk Puanlama Motoru (Risk Scoring Engine):**
  - [x] `SandboxPolicy` risk puanı hesaplama (0-100 puan, `Low`, `Medium`, `High`, `Critical`)
  - [x] Risk faktörleri ve uyarı mesajları derleyicisi (`RiskFlag`, `RiskScore`)
- [x] **Güven Seviyesi Sınıflandırması:**
  - [x] `TrustLevel` motoru (`Verified`, `Signed`, `Unsigned`)
- [x] **Etkileşimli Kullanıcı Onay Ekranı:**
  - [x] Kurulum esnasında istenen izinlerin risk seviyeleriyle konsolda sunulup onay istenmesi (`[e/H]`), `--yes` ile sessiz onay
- [x] **Bekçi IPC Daemon (`bekci-d`):**
  - [x] `~/.local/run/orion/bekci.sock` Unix Domain Socket üzerinden IPC mesajlaşma protokolü ve `opk daemon` komutu

## Milestone 8: Kod Kalitesi, İsimlendirme ve Test Kapsamı ✅
- [x] **İsimlendirme Standardizasyonu:**
  - [x] Dizin yolları, yapılandırma ve ortam değişkenleri `orion` standardına çekildi (`~/.local/share/orion/apps`, `/opt/orion/apps`, `~/.config/orion/signing`, `ORION_APPS_ROOT`)
- [x] **Derleme ve Uyarı Temizliği:**
  - [x] `tempfile::TempDir::into_path` uyarısı `TempDir::keep()` ile güncellendi
- [x] **Kapsamlı Test Kapsamı:**
  - [x] `mcpkg-core`: Manifest ve Policy sınır/hata testleri, SHA-256 sağlama testleri (6 test)
  - [x] `mcpkg-crypto`: Ed25519 anahtar üretimi, imzalama, doğrulama, bozulma ve serileştirme testleri (5 test)
  - [x] `mcpkg-sandbox`: AppArmor profil üretimi, Seccomp BPF filtreleme ve cgroups v2 sınır testleri (3 test)
  - [x] `mcpkg-pipeline`: Kurulum, rollback, ön/son betikler, SQLite denetim günlüğü, Bekçi puanlama, IPC daemon ve yaşam döngüsü testleri (18 test)
  - [x] `mcpkg-cli`: CLI komutları uçtan uca test edildi (`install`, `verify`, `info`, `list`, `uninstall`, `update`, `audit`, `search`, `daemon`)

---

## 🔮 Gelecek Fazlar & Sonraki Adımlar (Next Steps / Roadmap)

### Milestone 9: Sistem & Grafik Arayüz (GUI) Entegrasyonu ⏳
- [ ] **Orion Shell & Launcher Entegrasyonu:**
  - [ ] `opk daemon` IPC soketi (`bekci.sock`) üzerinden paket kurulum, kaldırma ve güncelleme olay akışı (event broadcasting)
  - [ ] Grafiksel izin onay ekranı (GUI Gatekeeper Prompt): Terminal `[e/H]` onayı yerine Orion Shell üzerinden grafiksel modal penceresi
  - [ ] Uygulama başlatıcı (`Launcher`) arayüzünde yeni kurulan paketlerin `.desktop` ve ikonlarının anlık dinamik yenilenmesi
- [ ] **Grafik Sunucusu & Wayland İzolasyonu:**
  - [ ] Sandbox içindeki uygulamalara Wayland soketi (`$XDG_RUNTIME_DIR/wayland-0`) ve DRI/GPU erişiminin `sandbox.policy` iznine göre izole bağlanması

### Milestone 10: Gelişmiş Dağıtım, Delta Güncelleme & PKI ⏳
- [ ] **Delta Güncelleme (Differential Updates):**
  - [ ] Büyük `.opk` paketleri için yalnızca değişen dosyaları indiren ikili fark (binary delta/patch) mekanizması
- [ ] **Güven Zinciri & Sertifika Hiyerarşisi (PKI):**
  - [ ] Orion OS kök güven sertifikası (Root of Trust) ve geliştirici sertifika yetkilendirme zinciri
  - [ ] Geliştirici anahtar iptal listesi (CRL - Certificate Revocation List) denetimi
- [ ] **Dinamik İzin Yönetimi (Dynamic Permission Revocation):**
  - [ ] Kullanıcının kurulu bir uygulamanın izinlerini (kamera, mikrofon, ağ vb.) sonradan çalışma zamanında tek tek açıp kapatabilmesi
- [ ] **Uçtan Uca CI/CD & E2E Entegrasyon Test Paketi:**
  - [ ] Otomatik test senaryosu (`mcpkg-cli/tests/e2e.rs`): Sıfırdan paket üretme, imzalama, registry'ye yükleme, indirme, kurma, sandbox'ta çalıştırma ve temiz kaldırma döngüsü


