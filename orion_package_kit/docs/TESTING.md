# Test ve Kalite Güvence Rehberi (Testing Guide)

> **Konum:** `docs/TESTING.md`  
> **Kapsam:** Birim Testler, Entegrasyon Testleri ve Çevrimdışı Test Stratejileri

---

## 1. Genel Bakış

Orion Package Kit (`opk`), işletim sisteminin çekirdek süreçlerini ve paket yaşam döngüsünü yönettiği için yüksek güvenilirlik ve test kapsamı gerektirir. Workspace genelinde birim testleri, kriptografik doğrulama testleri ve uçtan uca kurulum/rollback testleri yer alır.

---

## 2. Testleri Çalıştırma

### Tüm Workspace Testleri
```bash
cargo test
```

### Belirli Bir Crate'in Testleri
```bash
# Sadece pipeline ve durum makinesi testleri:
cargo test -p mcpkg-pipeline

# Sadece sandbox ve AppArmor testleri:
cargo test -p mcpkg-sandbox

# Geliştirici araçları testleri:
cargo test -p mcpkg-pipeline -- developer::tests
```

### Test Çıktısını Detaylı İnceleme
```bash
cargo test -- --nocapture
```

---

## 3. Mevcut Test Kategorileri

| Test Alanı | Konum | Açıklama |
|---|---|---|
| **Kriptografi & İmza** | `mcpkg-pipeline/src/developer.rs` | Anahtar üretimi, imzalama, bozuk anahtar reddi ve Ed25519 doğrulaması. |
| **Bütünlük Kontrolü** | `mcpkg-pipeline/src/lib.rs` | Bozulmuş dosya tespiti, `checksums.sha256` uyuşmazlığı ve zorunlu durdurma. |
| **Geri Alma (Rollback)** | `mcpkg-pipeline/src/lib.rs` | `pre-install.sh` veya `post-install.sh` hatasında sistemin eski haline geri dönmesi. |
| **Registry & Önbellek** | `mcpkg-pipeline/src/registry.rs` | `mock://` protokolü üzerinden ağ gerektirmeyen indirme ve önbellek testleri. |
| **AppArmor Üretimi** | `mcpkg-sandbox/src/apparmor.rs` | `sandbox.policy` kurallarının AppArmor profil sözdizimine dönüştürülmesi. |

---

## 4. Çevrimdışı Test için Mock Protokolü (`mock://`)

Ağ bağlantısı olmayan ortamlarda veya CI/CD iş akışlarında registry entegrasyonunu test etmek için dahili `mock://` şeması kullanılır:

```rust
// Test kodu örneği:
let client = RegistryClient::new(Some("mock://test_api".to_string()));
let pkg_info = client.get_package("com.mock.test")?;
assert_eq!(pkg_info["version"], "1.0.0");
```

Bu mekanizma sayesinde gerçek internet bağlantısı gerekmeden sahte bir paket dinamik olarak bellekte üretilir, hash'lenir ve kurulur.

---

## 5. Yeni Test Yazma Kuralları

1. **Geçici Dizin Kullanımı:** Testlerdeki dosya işlemleri daima `tempfile::tempdir()` içinde yapılmalıdır; sistem diskine doğrudan kalıcı dosya yazılmamalıdır.
2. **Determinizm:** Testler ağ gecikmesinden bağımsız ve birbirini etkilemeyecek şekilde izole çalışmalıdır.
3. **Rollback Doğrulaması:** Başarısız kurulum simülasyonlarında geçici dizinlerin (`staging`) temizlendiği `assert!(!staging_path.exists())` ile doğrulanmalıdır.
