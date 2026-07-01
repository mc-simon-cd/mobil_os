# Orion OS — Servisler Referans Kılavuzu

> Versiyon: 0.5.x · Tüm servisler `init.rc` tarafından başlatılır ve POSIX Unix socket üzerinden IPC kullanır.

---

## İçindekiler

1. [servicemanager](#servicemanager)
2. [surfaceflinger](#surfaceflinger)
3. [inputflinger](#inputflinger)
4. [apigateway](#apigateway)
5. [powermanager](#powermanager)
6. [apprunner](#apprunner)
7. [Ortak Desenler](#ortak-desenler)

---

## servicemanager

**Dil:** C  
**İkili:** `/system/bin/servicemanager`  
**Socket:** `/tmp/servicemanager.sock`  
**Kayıt adı:** *(kendisi kayıt altında değildir)*

### Görev

Sistem genelinde isim çözümleyici. Diğer servisler başladıklarında adlarını ve socket yollarını servicemanager'a kaydeder. İstemciler, bir servise bağlanmadan önce servicemanager'dan socket yolunu öğrenir.

### Komutlar

| Kod | Ad | İstek Payload | Yanıt Payload |
|---|---|---|---|
| `1` | `CMD_REGISTER_SERVICE` | `string name`, `string socket_path` | `int32 status` (0=OK) |
| `2` | `CMD_GET_SERVICE` | `string name` | `string socket_path` |
| `3` | `CMD_LIST_SERVICES` | — | `string[] names` |

### Kullanım Örneği

```c
// Servis kaydı (servisler tarafından yapılır)
parcel_t data, reply;
parcel_init(&data);
parcel_write_string(&data, "mobile.graphics");
parcel_write_string(&data, "/tmp/surfaceflinger.sock");
ipc_send_transaction(sm_fd, CMD_REGISTER_SERVICE, &data, &reply);

// Servis sorgusu (istemciler tarafından yapılır)
parcel_write_string(&data, "mobile.graphics");
ipc_send_transaction(sm_fd, CMD_GET_SERVICE, &data, &reply);
parcel_read_string(&reply, sf_path, sizeof(sf_path));
```

### Kayıtlı Servisler

| Ad | Sahip Servis |
|---|---|
| `mobile.graphics` | surfaceflinger |
| `mobile.input` | inputflinger |
| `mobile.api` | apigateway |
| `mobile.power` | powermanager |

---

## surfaceflinger

**Dil:** C++17  
**İkili:** `/system/bin/surfaceflinger`  
**Socket:** `/tmp/surfaceflinger.sock`  
**Kayıt adı:** `mobile.graphics`

### Görev

Grafik compositor. İstemci uygulamalardan gelen frame buffer'larını birleştirerek ekrana (PPM dosyası veya Wayland SHM) gönderir.

### Çalışma Modları

| Mod | Etkinleştirme | Arka uç |
|---|---|---|
| **PPM** | varsayılan | Software compositor → `out/display_composited.ppm` |
| **Wayland** | `--wayland` argümanı | wlroots headless backend |

### Komutlar (PPM Modu)

| Kod | Ad | İstek Payload | Yanıt Payload |
|---|---|---|---|
| `1` | `CMD_REGISTER_SURFACE` | `int32 width`, `int32 height` | `int32 surface_id` |
| `2` | `CMD_COMMIT_FRAME` | `int32 surface_id`, `raw pixels (RGBA)` | `int32 status` |
| `3` | `CMD_COMPOSITE` | — | `int32 status` |
| `4` | `CMD_SEND_INPUT_EVENT` | `int32 type`, `int32 code`, `int32 value` | `int32 status` *(Wayland modu)* |

### Wayland Modu Mimarisi

```
surfaceflinger --wayland
    │
    ├── wl_compositor_init()
    │     wl_display_create()
    │     wlr_backend_autocreate() [headless]
    │     wlr_renderer, wlr_allocator
    │     wlr_compositor, wlr_xdg_shell
    │     wlr_seat (touch injection için)
    │     wl_display_run()  ← event loop
    │
    └── IPC Thread (background)
          accept() /tmp/surfaceflinger.sock
          CMD=4 → wl_compositor_inject_touch(type, code, value)
```

### Touch Injection Akışı (Wayland)

```
wl_compositor_inject_touch(EV_ABS, ABS_X, x)
wl_compositor_inject_touch(EV_ABS, ABS_Y, y)
wl_compositor_inject_touch(EV_KEY, BTN_TOUCH, 1)   ← touch down
    → Y-koordinatına göre hedef surface bul
    → wlr_seat_touch_notify_down(seat, surface, time, id, sx, sy)
wl_compositor_inject_touch(EV_KEY, BTN_TOUCH, 0)   ← touch up
    → wlr_seat_touch_notify_up(seat, time, id)
```

### Önemli Dosyalar

| Dosya | İçerik |
|---|---|
| `src/main.cpp` | Ana giriş noktası, PPM ve Wayland dal seçimi |
| `src/composer.cpp` | PPM modunda surface birleştirme mantığı |
| `src/composer.h` | `CMD_*` sabitler, `handle_client()` |
| `wl/wl_compositor.cpp` | wlroots compositor implementasyonu |
| `wl/wl_compositor.h` | `wl_compositor_init/run/cleanup/inject_touch` |

---

## inputflinger

**Dil:** Rust  
**İkili:** `/system/bin/inputflinger`  
**Socket:** `/tmp/inputflinger.sock`  
**Kayıt adı:** `mobile.input`

### Görev

Dokunma ve tuş olaylarını alır, kayıtlı dinleyicilere iletir. Wayland modunda olayları ayrıca surfaceflinger'ın `wlr_seat`'ine enjekte eder.

### Komutlar

| Kod | Ad | İstek Payload | Yanıt Payload |
|---|---|---|---|
| `1` | `CMD_REGISTER_LISTENER` | `string socket_path` | `int32 status` |
| `2` | `CMD_SEND_INPUT_EVENT` | `int32 type`, `int32 code`, `int32 value` | `int32 status` |
| `3` | `CMD_GET_LAST_EVENT` | — | `int32 status`, `int32 type`, `int32 code`, `int32 value` |

### Olay Tipleri

Linux evdev standardına uyar:

| type | code | value | Açıklama |
|---|---|---|---|
| `3` (EV_ABS) | `0` (ABS_X) | 0–1080 | X koordinatı |
| `3` (EV_ABS) | `1` (ABS_Y) | 0–2400 | Y koordinatı |
| `1` (EV_KEY) | `330` (BTN_TOUCH) | `1` | Dokunma başlangıcı |
| `1` (EV_KEY) | `330` (BTN_TOUCH) | `0` | Dokunma sonu |

### Wayland Modu Akışı

```rust
// WAYLAND_ENABLED=1 ise CMD_SEND_INPUT_EVENT'te:
if wayland_enabled {
    // 1. surfaceflinger'a touch inject et
    ipc_send_transaction(&mut sf_stream, SF_CMD_INJECT_TOUCH=4, &parcel);
}
// 2. Kayıtlı listener'lara da ilet (legacy compat)
for listener in &state.listeners { ... }
```

### Örnek İstemci

```c
// Uygulama, inputflinger'a dinleyici olarak kayıt olur
parcel_write_string(&data, "/tmp/my_app_input.sock");
ipc_send_transaction(if_fd, CMD_REGISTER_LISTENER, &data, &reply);

// Test: touch olayı gönder
parcel_write_int32(&data, 3);   // EV_ABS
parcel_write_int32(&data, 0);   // ABS_X
parcel_write_int32(&data, 540); // x=540
ipc_send_transaction(if_fd, CMD_SEND_INPUT_EVENT, &data, &reply);
```

---

## apigateway

**Dil:** Rust  
**İkili:** `/system/bin/apigateway`  
**Socket:** `/tmp/apigateway.sock`  
**Kayıt adı:** `mobile.api`  
**HTTP Port:** `8080`

### Görev

HTTP REST arayüzü ile sistem servisleri arasında köprü kurar. Dış istemcilerin (test araçları, geliştirici araçları, web UI) IPC servislerine HTTP üzerinden erişmesini sağlar.

### HTTP Uç Noktaları

| Method | Path | Açıklama |
|---|---|---|
| `GET` | `/api/v1/status` | Sistem durumu ve servis listesi |
| `POST` | `/api/v1/input/touch` | Touch olayı enjekte et |
| `GET` | `/api/v1/display/snapshot` | PPM anlık görüntüsünü al |
| `POST` | `/api/v1/apps/launch` | Uygulama başlat |
| `GET` | `/api/v1/apps/list` | Yüklü uygulama listesi |
| `GET` | `/api/v1/power/status` | Batarya ve güç durumu |

### Touch Örneği

```bash
# Ekrana 540,1200 koordinatında dokunma simülasyonu
curl -X POST http://localhost:8080/api/v1/input/touch \
     -H "Content-Type: application/json" \
     -d '{"x": 540, "y": 1200, "action": "tap"}'
```

### IPC Komutları (Internal)

| Kod | Ad | Açıklama |
|---|---|---|
| `1` | `CMD_GET_STATUS` | Sistem durumu sorgula |
| `2` | `CMD_INJECT_TOUCH` | inputflinger'a olay ilet |
| `3` | `CMD_LIST_APPS` | Uygulama listesi al |
| `4` | `CMD_LAUNCH_APP` | apprunner üzerinden app başlat |

---

## powermanager

**Dil:** Rust  
**İkili:** `/system/bin/powermanager`  
**Kayıt adı:** `mobile.power`

### Görev

Ekran aydınlatma, batarya durumu izleme ve sistem uyku yönetimini üstlenir.

### Desteklenen Özellikler

| Özellik | Durum |
|---|---|
| Ekran açma/kapama | ✅ |
| Batarya seviyesi simülasyonu | ✅ |
| Uyku modu gecikmesi | ✅ |
| Şarj durumu bildirimi | 🔄 Geliştirilmekte |
| Termal yönetim | 🔄 Geliştirilmekte |

### IPC Komutları

| Kod | Ad | Yanıt |
|---|---|---|
| `1` | `CMD_GET_BATTERY` | `int32 level (0-100)`, `int32 charging` |
| `2` | `CMD_SCREEN_OFF` | `int32 status` |
| `3` | `CMD_SCREEN_ON` | `int32 status` |
| `4` | `CMD_GET_POWER_STATE` | `string state` |

---

## apprunner

**Dil:** Rust  
**İkili:** `/system/bin/apprunner`  
**Kayıt adı:** `mobile.runner`

### Görev

Uygulama yaşam döngüsü yöneticisi. OPK paket ikililerini izole namespace ortamında (`CLONE_NEWPID`, `CLONE_NEWNS`) başlatır.

### Sandbox Özellikleri

| Mekanizma | Durum |
|---|---|
| PID namespace izolasyonu | ✅ |
| Mount namespace izolasyonu | ✅ |
| seccomp filtresi | 🔄 Geliştirilmekte |
| AppArmor profili | 🔄 Geliştirilmekte |

### IPC Komutları

| Kod | Ad | İstek Payload | Yanıt |
|---|---|---|---|
| `1` | `CMD_LAUNCH` | `string package_name` | `int32 pid` |
| `2` | `CMD_KILL` | `int32 pid` | `int32 status` |
| `3` | `CMD_STATUS` | `int32 pid` | `string state` |
| `4` | `CMD_LIST` | — | `string[] running_apps` |

---

## Ortak Desenler

### Servis Başlangıç Şablonu

Her servis şu adımları izler:

```
1. register_with_servicemanager()   ← Adı ve socket yolunu kaydet
2. unlink(SOCKET_PATH)              ← Eski socket temizle
3. socket() + bind() + listen()    ← Dinlemeye başla
4. chmod(SOCKET_PATH, 0666)         ← İzin aç
5. accept() loop                    ← İstemci bağlantılarını kabul et
   └── handle_client()              ← Her bağlantı için işlem yap
```

### Hata Yönetimi

- Servisler başlatılamadığında `init` SIGCHLD alır
- `respawn=1` olan servisler otomatik yeniden başlatılır
- `init.rc`'de `class core` servisleri kritik sayılır

### Güvenlik Modeli

- Tüm servisler şu an `root` kullanıcısıyla çalışır
- `/tmp/*.sock` soketleri `0666` modunda
- Gelecek: UID-bazlı izin modeli, seccomp

---

*© 2026 mcsimon — Apache License 2.0*
