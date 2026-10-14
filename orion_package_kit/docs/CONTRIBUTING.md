# Katkıda Bulunma Kılavuzu (Contributing Guide)

Orion Package Kit (`opk`) projesine katkıda bulunmak istediğiniz için teşekkür ederiz!  
Bu belge, geliştirme ortamının hazırlanması, kodlama standartları ve Pull Request (PR) süreçlerini açıklamaktadır.

---

## 1. Geliştirme Ortamının Hazırlanması

### Gereksinimler

* **Rust:** 1.80+ (Cargo ile birlikte)
* **Linux Dağıtımı:** Çekirdek 6.x+ (Arch Linux, Ubuntu 24.04 vb.)
* **Sistem Kütüphaneleri:** `libseccomp-dev`, `apparmor-utils`, `build-essential`

```bash
# Arch Linux
sudo pacman -S rust base-devel libseccomp apparmor

# Ubuntu / Debian
sudo apt-get install rustc cargo libseccomp-dev apparmor-utils build-essential
```

### Projeyi Klonlama ve Derleme

```bash
git clone https://github.com/mc-simon-cd/mobil_os.git
cd mobil_os/orion_package_kit

# Tüm workspace'i derle
cargo build

# Birim testlerini çalıştır
cargo test
```

---

## 2. Kodlama Kuralları ve Standartlar

1. **Rustfmt ve Clippy:** Tüm kodlar standart `rustfmt` ile formatlanmalı ve hiçbir `clippy` uyarısı içermemelidir:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   ```
2. **Hata Yönetimi (Error Handling):** `panic!`, `unwrap()` veya `expect()` üretim kodunda (`src/`) kullanılmamalıdır; `Result<T, E>` ve anlamlı hata mesajları döndürülmelidir.
3. **Güvenlik İlkeleri:**
   * Çekirdek sistem çağrıları ve namespace işlemleri `unsafe` blokları içerisinde asgari düzeyde tutulmalıdır.
   * `rootless` çalışmayı bozacak ayrıcalıklı varsayımlardan kaçınılmalıdır.

---

## 3. Commit Mesaj Standartları

[Conventional Commits](https://www.conventionalcommits.org/) standardı zorunludur:

* `feat: add opk uninstall command`
* `fix: correct desktop exec path in pipeline`
* `docs: update cli reference for info command`
* `refactor: standardize orion paths across modules`
* `test: add unit tests for manifest parsing`

---

## 4. Pull Request (PR) Süreci

1. Projeyi kendi hesabınıza forklayın.
2. Anlamlı bir branch açın (`feature/uninstall-support` veya `bugfix/tempdir-warning`).
3. Değişikliklerinizi yapın ve ilgili testleri ekleyin.
4. `cargo test` komutunun başarıyla geçtiğinden emin olun.
5. PR açarken açıklama kısmında çözülen sorunu ve test sonuçlarını belirtin.

---

## 5. Güvenlik Açığı Bildirimi

Bir güvenlik açığı tespit ettiyseniz lütfen bunu genel bir GitHub Issue'su olarak **AÇMAYIN**.  
Detayları [SECURITY.md](SECURITY.md) dosyasındaki prosedürü izleyerek `security@orion.app` adresine iletiniz.
