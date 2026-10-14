# Kurulum ve Derleme Kılavuzu (Installation Guide) — `opk`

> **Konum:** `docs/INSTALL.md`  
> **Hedef:** Orion OS ve Linux Sistemleri

---

## 1. Sistem Gereksinimleri

* **İşletim Sistemi:** Linux Kernel 6.0+
* **İşlemci Mimarisi:** `x86_64` veya `aarch64` (ARM64)
* **Kernel Özellikleri:**
  * User Namespaces (`CONFIG_USER_NS=y`)
  * cgroups v2 (`systemd.unified_cgroup_hierarchy=1`)
  * AppArmor LSM
  * Seccomp BPF (`CONFIG_SECCOMP=y`, `CONFIG_SECCOMP_FILTER=y`)
* **Derleme Araçları:**
  * Rust & Cargo 1.80+
  * GCC / Clang
  * `libseccomp` kütüphanesi
  * `apparmor_parser` aracı

---

## 2. Gerekli Bağımlılıkların Kurulması

### Arch Linux / Orion OS:
```bash
sudo pacman -S rust cargo libseccomp apparmor base-devel
```

### Ubuntu / Debian:
```bash
sudo apt-get update
sudo apt-get install -y build-essential cargo rustc libseccomp-dev apparmor-utils
```

### Fedora:
```bash
sudo dnf install -y rust cargo libseccomp-devel apparmor-parser make gcc
```

---

## 3. Kaynaktan Derleme

Workspace kök dizininde ikiliyi derleyin:

```bash
cd orion_package_kit

# Hata ayıklama sürümü (Debug)
cargo build

# Üretim sürümü (Release - optimize edilmiş)
cargo build --release -p opk
```

Derlenen ikili dosya `target/release/opk` konumunda yer alır.

---

## 4. Sisteme Yükleme

İkiliyi sistem genelinde kullanılabilir hale getirmek için:

```bash
# Sistem genelinde /usr/local/bin altına kopyalama:
sudo cp target/release/opk /usr/local/bin/

# Veya kullanıcı dizini altına (~/.local/bin):
mkdir -p ~/.local/bin
cp target/release/opk ~/.local/bin/
```

Kabuğunuzda çalıştığını doğrulayın:
```bash
opk --version
# Çıktı: opk 0.1.0
```

---

## 5. Çekirdek İzinleri ve Rootless Ayarları

`opk` uygulamaları kök yetkisi olmadan (rootless) çalıştırmak için User Namespaces kullanır. Eğer sisteminizde unprivileged user namespaces kapalıysa açın:

```bash
# Geçici olarak açmak için:
sudo sysctl -w kernel.unprivileged_userns_clone=1

# Kalıcı yapmak için (/etc/sysctl.d/00-local-userns.conf):
echo "kernel.unprivileged_userns_clone=1" | sudo tee /etc/sysctl.d/00-local-userns.conf
```
