# Orion OS (mcsimon) - Kullanım ve Geliştirici Kılavuzu 📱

Bu kılavuz, **mcsimon** özel mobil işletim sisteminin %100 Rust mimarisini, derleme süreçlerini, QEMU emülatörü üzerinde çalıştırılmasını, sistem servislerini, Inter-Process Communication (IPC) yapısını, ağ yapılandırmasını, çoklu dil kütüphanesini (`libi18n`) ve test doğrulama aşamalarını detaylandırmaktadır.

---

## 🏛️ 1. Sistem Mimarisi ve Katmanları

Orion OS, sıfırdan geliştirilen düşük seviyeli kullanıcı alanı (user-space) servisleri ile minimal bir Linux çekirdeği (kernel) üzerinde çalışan **%100 Rust** dilinde modüler bir işletim sistemidir.

```
┌────────────────────────────────────────────────────────┐
│                   Sistem Uygulamaları                  │
│             (launcher, settings, dialer, opk)          │
└───────────────────────────┬────────────────────────────┘
                            │ (libi18n / libgraphics)
┌────────────────────────────────────────────────────────┐
│                    Mobil Shell UI                      │
│               (statusbar, notifications)               │
└───────────────────────────┬────────────────────────────┘
                            │ (libipc-rs)
┌────────────────────────────────────────────────────────┐
│                    Sistem Servisleri                   │
│   (surfaceflinger, inputflinger, powermanager, etc.)   │
└───────────────────────────┬────────────────────────────┘
                            │ (Unix Domain Sockets)
┌────────────────────────────────────────────────────────┐
│                servicemanager (Registry)               │
└───────────────────────────┬────────────────────────────┘
                            │ (Özel init.rc Yapılandırması)
┌────────────────────────────────────────────────────────┐
│               core/init (Rust Static Bin - PID 1)      │
└───────────────────────────┬────────────────────────────┘
                            │ (VFS Mounts / modules)
┌────────────────────────────────────────────────────────┐
│                      Linux Kernel                      │
└───────────────────────────┬────────────────────────────┘
                            │ (ARM64 / QEMU Sanal Donanımı)
┌────────────────────────────────────────────────────────┐
│                      Sanal Cihaz                       │
└────────────────────────────────────────────────────────┘
```

### Temel Bileşenler (Tümü Pure Rust)
1. **`core/init` (PID 1)**: Rust diliyle yazılmış önyükleme yöneticisi (`crt-static`). VFS mount, `e1000.ko` yükleme, ağ yapılandırması, `init.rc` ayrıştırma ve servis süpervizyonu yapar.
2. **`servicemanager` (IPC Kayıt Defteri)**: İletişim kurmak isteyen tüm servislerin uç noktalarını (`/tmp/*.sock`) kaydettiği ve sorguladığı merkezi rehber sistemidir.
3. **`surfaceflinger`**: Çok katmanlı yazılım tabanlı grafik kompozitör daemon'ıdır.
4. **`powermanager`**: Pil durumunu ve güç profillerini (balanced, performance, powersave) takip eden sistem servisidir.
5. **`inputflinger`**: Girdi enjeksiyon servisidir.
6. **`apigateway`**: REST HTTP -> IPC dönüştürücü sunucudur.
7. **`statusbar`**: Pil durumunu, aktif güç profilini ve bildirim mesajlarını çizen görseller üst çubuk servisidir.
8. **`launcher` / `settings` / `dialer`**: Yerel mobil shell uygulamalarıdır.

---

## 🛠️ 2. Geliştirme Ortamı ve Bağımlılıklar

Tüm bağımlılıklar merkezi manifest dosyasında tanımlıdır: [`deps/deps.yml`](deps/deps.yml)

### Otomatik Kurulum

```bash
# Ortamı kontrol et
./scripts/update-deps.sh --check

# Eksik paketleri kur
./scripts/update-deps.sh --install

# Tam pipeline: kurulum + cargo update + kernel + derleme + initramfs
./scripts/update-deps.sh --all
```

---

## 🏗️ 3. Derleme Sistemi (Build System)

### ARM64 Target Derlemesi (Önerilen)
QEMU emulatoründe çalışacak statik binary'leri çapraz derlemek için:
```bash
./scripts/build.sh
```

---

## 📦 4. Rootfs ve Initramfs İmajının Hazırlanması

```bash
./scripts/make-rootfs.sh
```

---

## 💻 5. QEMU Emülasyonu ve Önyükleme (Boot)

```bash
./scripts/download-kernel.sh
./scripts/make-rootfs.sh
./scripts/qemu-run.sh --headless
```

---

*Copyright © 2026 mcsimon. Tüm Hakları Saklıdır.*
