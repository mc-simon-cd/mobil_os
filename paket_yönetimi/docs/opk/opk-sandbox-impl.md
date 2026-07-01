# Sandbox Uygulama Detayları — Teknik Spesifikasyon

> **Versiyon:** 1.0.0-draft  
> **Kernel:** Linux 6.x+  
> **Durum:** Tasarım Aşaması  
> **Teknolojiler:** seccomp-bpf, namespaces, cgroups v2, AppArmor

---

## İçindekiler

1. [Katmanlı Savunma Modeli](#katmanlı-savunma-modeli)
2. [Linux Namespaces](#linux-namespaces)
3. [seccomp-bpf Filtreleme](#seccomp-bpf-filtreleme)
4. [AppArmor Profilleri](#apparmor-profilleri)
5. [cgroups v2 Kaynak Limitleri](#cgroups-v2-kaynak-limitleri)
6. [Launcher Mimarisi](#launcher-mimarisi)
7. [İzin → Kural Eşlemesi](#i̇zin--kural-eşlemesi)
8. [Ağ Sandbox'ı](#ağ-sandboxı)

---

## Katmanlı Savunma Modeli

Hiçbir tek mekanizma tam koruma sağlamaz. Katmanlar birbirini tamamlar:

```
┌─────────────────────────────────────────────┐
│  Uygulama (binary)                          │
├─────────────────────────────────────────────┤
│  seccomp-bpf      → sistem çağrısı filtresi │  ← kernel düzeyinde
├─────────────────────────────────────────────┤
│  AppArmor         → dosya/ağ MAC politikası │  ← LSM düzeyinde
├─────────────────────────────────────────────┤
│  Namespaces       → görünüm izolasyonu      │  ← namespace düzeyi
├─────────────────────────────────────────────┤
│  cgroups v2       → kaynak sınırlama        │  ← kernel düzeyi
├─────────────────────────────────────────────┤
│  Linux Kernel                               │
└─────────────────────────────────────────────┘
```

Bir katman aşılırsa diğerleri devreye girer.

---

## Linux Namespaces

### Kullanılan Namespace Türleri

| Namespace | Flag | Ne İzole Eder |
|---|---|---|
| Mount | `CLONE_NEWNS` | Dosya sistemi görünümü |
| PID | `CLONE_NEWPID` | Süreç ID'leri |
| User | `CLONE_NEWUSER` | UID/GID eşlemesi |
| Network | `CLONE_NEWNET` | Ağ arayüzleri |
| IPC | `CLONE_NEWIPC` | Shared memory, message queue |
| UTS | `CLONE_NEWUTS` | Hostname |

### Rust Implementasyonu

```rust
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{fork, ForkResult};

pub fn spawn_sandboxed(
    binary: &Path,
    policy: &SandboxPolicy,
) -> Result<Child, SandboxError> {

    let mut flags = CloneFlags::CLONE_NEWNS    // mount
                  | CloneFlags::CLONE_NEWPID   // pid
                  | CloneFlags::CLONE_NEWUSER  // user
                  | CloneFlags::CLONE_NEWIPC;  // ipc

    // Ağ izolasyonu — policy'e göre
    if policy.network.internet == Permission::Deny
    && policy.network.localhost == Permission::Deny {
        flags |= CloneFlags::CLONE_NEWNET;
    }

    unshare(flags)?;

    // UID/GID eşlemesi — container içinde sahte root, dışarıda normal kullanıcı
    write_uid_map()?;
    write_gid_map()?;

    // Mount namespace kur
    setup_mount_namespace(policy)?;

    // seccomp filtre uygula
    apply_seccomp_filter(policy)?;

    // Binary'yi çalıştır
    exec_binary(binary)
}
```

### Mount Namespace Kurulumu

```rust
fn setup_mount_namespace(policy: &SandboxPolicy) -> Result<(), SandboxError> {
    // Tüm mount'ları private yap (propagasyon engeli)
    mount(None::<&str>, "/", None::<&str>, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None::<&str>)?;

    // Uygulama dizini — okuma/yazma
    let app_data = format!("{}/.local/share/{}", home_dir(), policy.package_id);
    bind_mount(&app_data, &app_data, MsFlags::MS_BIND)?;

    // Sistem kütüphaneleri — salt okunur
    bind_mount("/usr/lib", "/usr/lib", MsFlags::MS_BIND | MsFlags::MS_RDONLY)?;
    bind_mount("/usr/bin", "/usr/bin", MsFlags::MS_BIND | MsFlags::MS_RDONLY)?;

    // /tmp — her uygulama için ayrı
    let tmp = format!("/tmp/orion-{}", policy.package_id);
    std::fs::create_dir_all(&tmp)?;
    bind_mount(&tmp, "/tmp", MsFlags::MS_BIND)?;

    // system_read = deny → /etc bağlanmaz
    if policy.filesystem.system_read == Permission::Allow {
        bind_mount("/etc", "/etc", MsFlags::MS_BIND | MsFlags::MS_RDONLY)?;
    }

    // Proc filesystem
    mount(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), None::<&str>)?;

    Ok(())
}
```

---

## seccomp-bpf Filtreleme

Uygulama hangi sistem çağrılarını yapabileceği kısıtlanır.

### Temel Strateji

```
Allowlist yaklaşımı:
  Beyaz listedeki sistem çağrıları → izin ver
  Listedeki olmayan her şey        → EPERM döndür (öldürme değil)
```

`EPERM` tercih sebebi: Uygulama çökmez, hata kodu alır ve handle edebilir.

### Rust Implementasyonu (libseccomp)

```rust
use seccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};

pub fn apply_seccomp_filter(policy: &SandboxPolicy) -> Result<(), SandboxError> {
    // Varsayılan: izin ver (whitelist değil, kısıtlı syscall'ları engelle)
    let mut ctx = ScmpFilterContext::new_filter(ScmpAction::Allow)?;

    // exec_subprocess = deny → execve ve türevlerini engelle
    if policy.system.exec_subprocess == Permission::Deny {
        ctx.add_rule(ScmpAction::Errno(libc::EPERM), ScmpSyscall::from_name("execve")?)?;
        ctx.add_rule(ScmpAction::Errno(libc::EPERM), ScmpSyscall::from_name("execveat")?)?;
        ctx.add_rule(ScmpAction::Errno(libc::EPERM), ScmpSyscall::from_name("execve")?)?;
    }

    // Her zaman engellenen tehlikeli sistem çağrıları
    let always_blocked = [
        "ptrace",           // Diğer süreçlere debug bağlantısı
        "process_vm_readv", // Başka sürecin belleğini okuma
        "process_vm_writev",// Başka sürecin belleğine yazma
        "kexec_load",       // Kernel yükleme
        "perf_event_open",  // Performans olaylarını izleme (yan kanal)
    ];

    for syscall_name in &always_blocked {
        ctx.add_rule(
            ScmpAction::Errno(libc::EPERM),
            ScmpSyscall::from_name(syscall_name)?
        )?;
    }

    ctx.load()?;
    Ok(())
}
```

### Politikaya Göre Ek Engellemeler

| `sandbox.policy` | Engellenen Syscall'lar |
|---|---|
| `exec_subprocess = deny` | `execve`, `execveat`, `posix_spawn` |
| `network.internet = deny` | `connect`, `bind`, `sendto` (namespace yeterli ama çift katman) |
| `hardware.camera = deny` | `/dev/video*` açma girişimleri AppArmor ile engellenir |
| `system.ipc = deny` | `msgget`, `shmget`, `semget` |

---

## AppArmor Profilleri

Kurulum sırasında `sandbox.policy`'den otomatik üretilir.

### Profil Üretici

```rust
pub fn generate_apparmor_profile(
    package_id: &str,
    policy: &SandboxPolicy,
) -> String {
    let mut rules = Vec::new();

    // Temel — binary çalıştırma izni
    rules.push(format!(
        "/opt/orion/apps/{}/bin/* mr,",
        package_id
    ));

    // Kütüphaneler
    rules.push("/usr/lib/** mr,".to_string());

    // Uygulama veri dizini
    let data_dir = format!("/home/*/.local/share/{}/**", package_id);
    rules.push(format!("{} rw,", data_dir));

    // Geçici dizin
    rules.push(format!("/tmp/orion-{}/** rw,", package_id));

    // Ağ kuralları
    if policy.network.internet == Permission::Allow {
        rules.push("network inet stream,".to_string());
        rules.push("network inet dgram,".to_string());
        rules.push("network inet6 stream,".to_string());
    } else {
        rules.push("deny network inet stream,".to_string());
        rules.push("deny network inet dgram,".to_string());
    }

    // Sistem dosyaları
    if policy.filesystem.system_read == Permission::Deny {
        rules.push("deny /etc/** rw,".to_string());
        rules.push("deny /boot/** rw,".to_string());
        rules.push("deny /proc/sysrq-trigger rw,".to_string());
    }

    // Kamera
    if policy.hardware.camera == Permission::Deny {
        rules.push("deny /dev/video* rw,".to_string());
    }

    // Mikrofon
    if policy.hardware.microphone == Permission::Deny {
        rules.push("deny /dev/snd/pcm* rw,".to_string());
    }

    format!(
        "# Otomatik üretildi — opk v1.0\nprofile {} {{\n  {}\n}}\n",
        package_id,
        rules.join("\n  ")
    )
}
```

### Üretilen Profil Örneği

```apparmor
# Otomatik üretildi — opk v1.0
# Paket: com.orion.notepad v1.2.3

profile com.orion.notepad {

  # Binary
  /opt/orion/apps/com.orion.notepad/bin/* mr,

  # Sistem kütüphaneleri (salt okunur)
  /usr/lib/** mr,
  /lib/** mr,

  # Uygulama verisi
  /home/*/.local/share/com.orion.notepad/** rw,
  /home/*/.config/com.orion.notepad/** rw,

  # Geçici dosyalar
  /tmp/orion-com.orion.notepad/** rw,

  # Ağ: internet = deny
  deny network inet stream,
  deny network inet dgram,
  deny network inet6 stream,

  # Sistem: erişim yok
  deny /etc/** rw,
  deny /boot/** rw,
  deny /sys/** rw,

  # Donanım: kamera ve mikrofon yok
  deny /dev/video* rw,
  deny /dev/snd/pcm* rw,

  # Diğer kullanıcıların dizinleri yok
  deny /root/** rw,
  deny /home/*/** rw,

  # Varsayılan: sinyaller
  signal receive set=(term, kill),
}
```

### Profil Yükleme

```bash
# Kurulum sırasında
apparmor_parser -r /etc/apparmor.d/orion/com.orion.notepad

# Kaldırma sırasında
apparmor_parser -R /etc/apparmor.d/orion/com.orion.notepad
rm /etc/apparmor.d/orion/com.orion.notepad
```

---

## cgroups v2 Kaynak Limitleri

Her uygulama kendi cgroup'unda çalışır.

### Cgroup Hiyerarşisi

```
/sys/fs/cgroup/
└── orion/
    └── apps/
        └── com.orion.notepad/
            ├── cpu.max
            ├── memory.max
            ├── memory.swap.max
            └── io.max
```

### Rust Implementasyonu

```rust
pub fn setup_cgroup(package_id: &str, limits: &ResourceLimits) -> Result<(), SandboxError> {
    let cgroup_path = format!("/sys/fs/cgroup/orion/apps/{}", package_id);
    std::fs::create_dir_all(&cgroup_path)?;

    // CPU limiti: "max quota period" formatı
    // Örnek: "50000 100000" = %50 CPU
    let cpu_max = match limits.cpu_percent {
        Some(pct) => format!("{} 100000", (pct * 1000.0) as u64),
        None => "max 100000".to_string(),  // limitsiz
    };
    std::fs::write(format!("{}/cpu.max", cgroup_path), cpu_max)?;

    // Bellek limiti (byte cinsinden)
    let mem_max = limits.memory_bytes
        .map(|b| b.to_string())
        .unwrap_or_else(|| "max".to_string());
    std::fs::write(format!("{}/memory.max", cgroup_path), mem_max)?;

    // Swap tamamen kapat
    std::fs::write(format!("{}/memory.swap.max", cgroup_path), "0")?;

    // Mevcut süreci bu cgroup'a ekle
    let pid = std::process::id();
    std::fs::write(format!("{}/cgroup.procs", cgroup_path), pid.to_string())?;

    Ok(())
}
```

### Varsayılan Limitler

```toml
# manifest.toml'da geliştirici belirtebilir
[resources]
memory_mb = 512    # varsayılan
cpu_percent = 50   # varsayılan, anlık max
```

Belirtilmezse sistem varsayılanları:

| Kaynak | Varsayılan |
|---|---|
| Bellek | 1 GB |
| Swap | 0 (kapalı) |
| CPU | limitsiz (fair scheduling) |
| IO | limitsiz |

---

## Launcher Mimarisi

Uygulamalar `.desktop` aracılığıyla doğrudan değil, `orion-launcher` üzerinden başlatılır.

```
Kullanıcı ikona çift tıklar
          │
          ▼
  orion-launcher com.orion.notepad
          │
          ▼
  manifest.toml oku
  sandbox.policy oku
          │
          ▼
  Namespace'leri kur
  cgroup oluştur
  AppArmor profilini doğrula
  seccomp filtreyi yükle
          │
          ▼
  execve → /opt/orion/apps/com.orion.notepad/bin/notepad
```

Bu mimarinin avantajı: Sandbox her çalıştırmada taze kurulur — hiçbir escape kalıcı olmaz.

---

## İzin → Kural Eşlemesi

`sandbox.policy` alanının hangi mekanizmaya hangi kuralı ürettiği:

| Policy Alanı | Değer | Mekanizma | Üretilen Kural |
|---|---|---|---|
| `network.internet` | `allow` | AppArmor | `network inet stream,` |
| `network.internet` | `deny` | AppArmor + Namespace | `deny network inet` + `CLONE_NEWNET` |
| `filesystem.system_read` | `deny` | AppArmor + Mount | `deny /etc/**` + `/etc` mount edilmez |
| `filesystem.arbitrary_path` | `deny` | AppArmor | `deny /**` (whitelist dışı) |
| `system.exec_subprocess` | `deny` | seccomp | `execve → EPERM` |
| `system.ipc` | `deny` | seccomp | `msgget, shmget → EPERM` |
| `system.autostart` | `deny` | Launcher | systemd unit oluşturulmaz |
| `hardware.camera` | `deny` | AppArmor | `deny /dev/video*` |
| `hardware.microphone` | `deny` | AppArmor | `deny /dev/snd/pcm*` |
| `hardware.usb_devices` | `deny` | AppArmor | `deny /dev/bus/**` |

---

## Ağ Sandbox'ı

### Senaryo 1: `internet = deny, localhost = deny`

```
Tam ağ izolasyonu:
  CLONE_NEWNET → yeni network namespace
  Yalnızca loopback arayüzü (lo) — up edilmez
  AppArmor → deny network inet
```

### Senaryo 2: `internet = deny, localhost = allow`

```
Yalnızca yerel iletişim:
  Network namespace paylaşılır (ana namespace)
  AppArmor → deny network inet (sadece 127.0.0.1'e izin)
  iptables kural: OUTPUT -d 127.0.0.1 -j ACCEPT, gerisi DROP
```

### Senaryo 3: `internet = allow`

```
Tam erişim:
  Network namespace paylaşılır
  AppArmor → network inet stream/dgram
  Kısıtlama yok
```

### Senaryo 4: `local_network = ask`

```
Kullanıcı ilk bağlantı girişiminde sorulur:
  Uygulama 192.168.x.x adresine bağlanmaya çalışır
  → Launcher interceptor → kullanıcıya bildir
  → [İzin Ver] [Her Zaman İzin Ver] [Reddet]
  Karar denetim günlüğüne yazılır
```

---

*Bu doküman Orion ekosistemi dahilinde geliştirilmektedir.*  
*Sonraki: `opk-signing-guide.md`*
