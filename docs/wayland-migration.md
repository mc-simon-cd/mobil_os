# Orion OS — Wayland Geçiş Planı

Bu belge, Orion OS v1 alpha’daki **PPM yazılım kompozitör** modelinden **Wayland + wlroots** tabanlı gerçek ekran çıkışına geçiş stratejisini tanımlar.

> **Mevcut durum (v1.0.0-alpha):** `surfaceflinger` Wayland kullanmaz. Uygulamalar `libgraphics` ile çizer, `out/surface_<id>.ppm` yazar; kompozitör `out/display_composited.ppm` üretir. Control Center bu dosyayı HTTP ile önizler (Milestone 15).

---

## 1. Neden Wayland?

| PPM (bugün) | Wayland (hedef) |
|:---|:---|
| QEMU / geliştirme için basit | Gerçek panel, GPU, vsync |
| Disk I/O her karede | Bellek içi buffer paylaşımı |
| Özel input IPC | `wl_touch`, `wl_pointer`, seat |
| HTTP önizleme kolay | Standart Linux mobil yığını |

PPM alpha için doğru seçimdi; üretim cihazı ve performans için Wayland gerekli.

---

## 2. Mimari Karşılaştırma

### 2.1 Bugün (PPM pipeline)

```
launcher / statusbar / settings
        │
        ▼
   libgraphics (canvas_t)
        │
        ▼
 out/surface_<id>.ppm
        │
        ▼
 surfaceflinger (composer.cpp)
        │
        ▼
 out/display_composited.ppm
        │
        ├──► apigateway GET /api/display/frame
        └──► (dosya olarak debug)
```

### 2.2 Hedef (Wayland pipeline)

```
Wayland istemcileri (launcher, settings, …)
        │
        ▼
 wl_surface + wl_buffer (SHM veya EGL)
        │
        ▼
 surfaceflinger (wlroots compositor)
        │
        ├──► DRM/KMS → fiziksel panel
        ├──► wayland-backend headless → test
        └──► screencopy → apigateway önizleme
        │
 inputflinger ← libinput / wl_seat
```

---

## 3. Geçiş Aşamaları

### Faz 0 — Tamamlanan (alpha)
- [x] PPM `surfaceflinger`, `libgraphics`, shell uygulamaları
- [x] HTTP display preview (`/api/display/frame`)
- [x] `inputflinger` + launcher touch loop

### Faz 1 — Wayland altyapısı (Milestone 17)
- [x] `deps/deps.yml`: `wayland`, `wayland-protocols`, `wlroots`, `libinput`, `mesa` (dev)
- [x] `services/surfaceflinger/` altında `wl/` modülü; mevcut PPM compositor paralel çalışsın
- [x] Headless backend: QEMU’da `WLR_BACKENDS=headless`
- [x] `WAYLAND_DISPLAY=wayland-0` socket’i `/run/user/0/wayland-0` veya `/tmp/wayland-0`

### Faz 2 — İlk Wayland istemcisi
- [x] `libs/libwlcanvas` veya `libgraphics` içinde `wl_shm` buffer export
- [x] Launcher’ı Wayland native client olarak çift mod (`--ppm-legacy` / varsayılan Wayland)
- [ ] `inputflinger`: `wl_touch` olaylarını mevcut IPC formatına çevir (geriye uyum)

### Faz 3 — Tam shell geçişi
- [x] Statusbar, Settings, Dialer → Wayland surface
- [ ] PPM export yalnızca `--debug-dump-ppm` flag’i ile
- [ ] `apigateway`: `wlr-screencopy` veya shared memory ile `/api/display/frame` (PPM fallback)

### Faz 4 — Üretim sertleştirme
- [ ] ARM64 gerçek donanım (PinePhone / custom board) DRM backend
- [ ] Rotation, güç yönetimi (panel off), vsync
- [ ] PPM kod yolunun kaldırılması

---

## 4. Bileşen Eşlemesi

| Bileşen | PPM (bugün) | Wayland (hedef) |
|:---|:---|:---|
| `surfaceflinger` | `composer.cpp` PPM merge | wlroots `wlr_compositor`, layer shell |
| `libgraphics` | RAM canvas → PPM | SHM buffer veya Cairo → wl_buffer |
| `apps/launcher` | `canvas_save_ppm` + IPC alloc | `xdg_shell` / `layer_shell` surface |
| `inputflinger` | IPC inject + listener | `wlr_seat`, `libinput` |
| `apigateway` | `read(display_composited.ppm)` | screencopy / pipewire (opsiyonel) |
| Control Center | PPM parse + canvas | WebSocket veya PNG tile stream |

---

## 5. Bağımlılıklar (taslak)

`deps/deps.yml` eklenecek paketler (Arch örnek):

```yaml
host.pacman:
  - wayland
  - wayland-protocols
  - wlroots
  - libinput
  - mesa
  - seatd
```

QEMU guest rootfs için statik veya dinamik link kararı Faz 1’de verilecek.

---

## 6. QEMU Stratejisi

| Ortam | Backend | Not |
|:---|:---|:---|
| Geliştirme (host) | `headless` | X11/DRM gerekmez |
| QEMU ARM64 | `headless` + port forward | Mevcut initramfs ile uyumlu |
| Gerçek cihaz | `drm` + `libinput` | Device tree / panel driver |

```bash
# Örnek (Faz 1 sonrası)
WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 ./system/bin/surfaceflinger
```

---

## 7. Geriye Uyumluluk

Geçiş süresince **çift mod** zorunlu:

1. `init.rc` property: `persist.graphics.backend=ppm|wayland`
2. Uygulamalar önce Wayland dener; başarısızsa PPM fallback
3. Entegrasyon testleri: `test_compositor_host.sh` (PPM) + `test_wayland_headless.sh` (yeni)

---

## 8. Riskler ve Azaltma

| Risk | Azaltma |
|:---|:---|
| wlroots ARM64 cross-compile | Faz 1’de yalnızca host headless doğrula |
| Initramfs boyutu | Dinamik `.so` ayrı partition veya ext4 (M16) |
| Control Center kırılması | PPM HTTP endpoint fallback olarak kalsın |
| Input çift yolu | `inputflinger` tek kayıt noktası; backend değişir |

---

## 9. Milestone Özeti

| ID | Başlık | Bağımlılık |
|:---|:---|:---|
| **15** | Control Center HTTP display preview (PPM) | — |
| **16** | `out/rootfs.ext4` disk boot | — |
| **17** | Wayland Faz 1: headless wlroots compositor | 15, 16 opsiyonel |
| **18** | Wayland Faz 2–3: shell istemci geçişi | 17 |
| **19** | PPM yolunun kaldırılması | 18 |

---

## 10. İlgili Dosyalar

| Dosya | Açıklama |
|:---|:---|
| `services/surfaceflinger/src/composer.cpp` | Mevcut PPM kompozitör |
| `libs/libgraphics/` | 2D çizim motoru |
| `services/apigateway/src/main.rs` | `/api/display/frame` |
| `rootfs/system/apps/control_center/` | Canlı önizleme UI |
| `progress.md` | Sprint takibi |

---

*Son güncelleme: 2026-06-09 — Orion OS v1.0.0-alpha*
