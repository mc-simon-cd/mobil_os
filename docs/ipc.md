# Orion OS — IPC (Süreçler Arası İletişim) Kılavuzu

> Versiyon: 0.5.x · Tüm IPC, POSIX Unix domain socket üzerinden senkron istek/yanıt modeliyle çalışır.

---

## İçindekiler

1. [Genel Bakış](#genel-bakış)
2. [Protokol Formatı](#protokol-formatı)
3. [Parcel API](#parcel-api)
4. [Binder API](#binder-api)
5. [Rust IPC Kütüphanesi (libipc-rs)](#rust-ipc-kütüphanesi-libipc-rs)
6. [Tam Bir İşlem Akışı](#tam-bir-işlem-akışı)
7. [Servis Komut Tablosu](#servis-komut-tablosu)
8. [Hata Kodları](#hata-kodları)
9. [Wayland IPC Uzantısı](#wayland-ipc-uzantısı)
10. [Test ve Hata Ayıklama](#test-ve-hata-ayıklama)

---

## Genel Bakış

Orion OS'un IPC sistemi Android Binder'dan esinlenmiş, ancak çok daha basit bir Unix socket tabanlı istek/yanıt protokolüdür. Bütün servisler şu özelliklerle çalışır:

- **Taşıma:** POSIX Unix domain socket (`AF_UNIX`, `SOCK_STREAM`)
- **Model:** Senkron istek/yanıt — istemci bağlanır, isteği gönderir, yanıtı alır, bağlantıyı kapatır
- **Serileştirme:** `parcel_t` — boyutu dinamik büyüyen byte dizisi
- **Header:** 8 byte sabit header (`ipc_header_t`) + değişken payload

```
İstemci                        Sunucu
   │── connect() ─────────────────►│
   │── write(header + payload) ───►│
   │◄─ read(header + payload) ─────│
   │── close() ──────────────────►│
```

---

## Protokol Formatı

### İstek ve Yanıt Yapısı

Her mesaj iki bölümden oluşur:

```
┌──────────────────────────────────────────┐
│            İPC HEADER (8 byte)           │
│  int32_t  code        (4 byte, LE)       │
│  uint32_t data_size   (4 byte, LE)       │
├──────────────────────────────────────────┤
│            PAYLOAD (data_size byte)      │
│  Parcel ile serileştirilmiş veriler      │
└──────────────────────────────────────────┘
```

### C Tanımı

```c
// libs/libipc/include/ipc/binder.h

typedef struct {
    int32_t  code;        // İşlem kodu (komut kimliği)
    uint32_t data_size;   // Payload boyutu (byte)
} ipc_header_t;
```

### Veri Akışı

```
[code: 4B][data_size: 4B][payload: N byte]
     ↑           ↑              ↑
   komut      kaç byte    parcel içeriği
```

---

## Parcel API

`parcel_t`, IPC veri serileştirme için kullanılan dinamik byte tamponu.

### Yapı

```c
// libs/libipc/include/ipc/parcel.h

typedef struct {
    char   *data;       // Raw byte dizisi
    size_t  capacity;   // Allocate edilmiş toplam byte
    size_t  size;       // Yazılan toplam byte
    size_t  read_pos;   // Okuma imleci
} parcel_t;
```

### Yaşam Döngüsü

```c
parcel_t p;
parcel_init(&p);        // Başlat (capacity = 64, dinamik büyür)
// ... veri yaz/oku ...
parcel_free(&p);        // Serbest bırak
```

### Yazma Fonksiyonları

```c
parcel_write_int32(parcel_t *p, int32_t val);       // 4 byte (little-endian)
parcel_write_uint32(parcel_t *p, uint32_t val);     // 4 byte (little-endian)
parcel_write_string(parcel_t *p, const char *str);  // 4B uzunluk + içerik
parcel_write_raw(parcel_t *p, const void *d, size_t len); // Ham byte
```

### Okuma Fonksiyonları

```c
parcel_read_int32(parcel_t *p, int32_t *val);
parcel_read_uint32(parcel_t *p, uint32_t *val);
parcel_read_string(parcel_t *p, char *buf, size_t buf_len);
parcel_read_raw(parcel_t *p, void *buf, size_t len);
```

### String Serileştirme Formatı

```
[length: uint32 4B][string bytes: N byte][null terminator: 1B]
```

### Örnek

```c
parcel_t data;
parcel_init(&data);
parcel_write_string(&data, "mobile.graphics");  // 4B uzunluk + içerik
parcel_write_int32(&data, 1080);                // 4B tamsayı
parcel_write_int32(&data, 2400);                // 4B tamsayı
// data.size = 4 + 15 + 1 + 4 + 4 = 28 byte
parcel_free(&data);
```

---

## Binder API

`libipc`'nin istemci tarafı arayüzü — bağlantı ve işlem yönetimi.

### Fonksiyonlar

```c
// Servise Unix socket bağlantısı kur
int ipc_connect(const char *socket_path);
// Döner: >=0 = dosya tanımlayıcı, -1 = hata

// İşlem gönder ve yanıt bekle
int ipc_send_transaction(int fd, int32_t code,
                         parcel_t *data, parcel_t *reply);
// Döner: IPC_SUCCESS (0) veya IPC_ERROR (-1)
```

### Dahili Akış (ipc_send_transaction)

```
1. ipc_header_t header = { .code = code, .data_size = data->size }
2. write(fd, &header, 8)           ← header gönder
3. write(fd, data->data, data->size) ← payload gönder
4. read(fd, &reply_header, 8)      ← yanıt header oku
5. read(fd, reply->data, reply_header.data_size)  ← yanıt payload oku
```

### Tam Kullanım Örneği

```c
#include "ipc/binder.h"
#include "ipc/parcel.h"

void ornek_ipc_cagrisi(void) {
    // 1. Bağlan
    int fd = ipc_connect("/tmp/surfaceflinger.sock");
    if (fd < 0) {
        fprintf(stderr, "Bağlantı kurulamadı!\n");
        return;
    }

    // 2. İstek parcel'ı hazırla
    parcel_t data, reply;
    parcel_init(&data);
    parcel_init(&reply);
    parcel_write_int32(&data, 1080);   // genişlik
    parcel_write_int32(&data, 2400);   // yükseklik

    // 3. CMD_REGISTER_SURFACE = 1
    if (ipc_send_transaction(fd, 1, &data, &reply) == IPC_SUCCESS) {
        int32_t surface_id;
        parcel_read_int32(&reply, &surface_id);
        printf("Surface ID: %d\n", surface_id);
    }

    // 4. Temizle
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
}
```

---

## Rust IPC Kütüphanesi (libipc-rs)

Rust servisleri için `libs/libipc-rs` crate'i.

### Parcel

```rust
use libipc_rs::Parcel;

let mut p = Parcel::new();
p.write_i32(42);
p.write_string("merhaba");

let val = p.read_i32().unwrap();    // → 42
let s   = p.read_string().unwrap(); // → "merhaba"
```

### Bağlantı ve İşlem

```rust
use libipc_rs::{ipc_connect, ipc_send_transaction};

// Bağlan
let mut stream = ipc_connect("/tmp/inputflinger.sock")?;

// Parcel hazırla
let mut data = Parcel::new();
data.write_i32(3);    // type = EV_ABS
data.write_i32(0);    // code = ABS_X
data.write_i32(540);  // value

// Gönder ve yanıt al (CMD_SEND_INPUT_EVENT = 2)
let mut reply = ipc_send_transaction(&mut stream, 2, &data)?;
let status = reply.read_i32()?;  // 0 = başarılı
```

---

## Tam Bir İşlem Akışı

### Örnek: Uygulama → surfaceflinger Frame Commit

```
┌─ LAUNCHER ───────────────────────────────────────────────┐
│                                                           │
│  1. ipc_connect("/tmp/servicemanager.sock")              │
│     CMD_GET_SERVICE("mobile.graphics")                   │
│     → sf_path = "/tmp/surfaceflinger.sock"               │
│                                                           │
│  2. ipc_connect(sf_path)                                 │
│     CMD_REGISTER_SURFACE(1080, 2400)                     │
│     → surface_id = 1                                     │
│                                                           │
│  3. canvas_fill_rect(...) / canvas_draw_text(...)        │
│                                                           │
│  4. ipc_connect(sf_path)                                 │
│     CMD_COMMIT_FRAME(surface_id, pixel_data)             │
│     → status = 0                                         │
│                                                           │
└───────────────────────────────────────────────────────────┘
        │
        ▼
┌─ SURFACEFLINGER ─────────────────────────────────────────┐
│  handle_client() → CMD_COMMIT_FRAME                      │
│  → surface_map[id] = pixel_data                          │
│  → composite_all_surfaces()                              │
│  → out/display_composited.ppm                            │
└───────────────────────────────────────────────────────────┘
```

### Örnek: Test Aracı → Touch Injection

```
┌─ TEST ARACI ──────────────────────────────────────────────┐
│  curl POST /api/v1/input/touch {x:540, y:1200}           │
└───────────────────────────────────────────────────────────┘
        │ HTTP POST
        ▼
┌─ APIGATEWAY ──────────────────────────────────────────────┐
│  ipc_connect("/tmp/inputflinger.sock")                   │
│  CMD_SEND_INPUT_EVENT(EV_ABS, ABS_X, 540)                │
│  CMD_SEND_INPUT_EVENT(EV_ABS, ABS_Y, 1200)               │
│  CMD_SEND_INPUT_EVENT(EV_KEY, BTN_TOUCH, 1)              │
└───────────────────────────────────────────────────────────┘
        │ IPC
        ▼
┌─ INPUTFLINGER ────────────────────────────────────────────┐
│  WAYLAND_ENABLED=1?                                       │
│  ├─ Evet: ipc_connect("/tmp/surfaceflinger.sock")        │
│  │         CMD_SEND_INPUT_EVENT=4 → wlr_seat inject      │
│  └─ Her zaman: listener'lara dispatch                    │
└───────────────────────────────────────────────────────────┘
        │ (Wayland)
        ▼
┌─ SURFACEFLINGER (Wayland IPC Thread) ─────────────────────┐
│  wl_compositor_inject_touch(EV_KEY, BTN_TOUCH, 1)        │
│  → surface hit-test (Y koordinatına göre)                │
│  → wlr_seat_touch_notify_down(seat, surface, t, 0, x, y) │
└───────────────────────────────────────────────────────────┘
        │ wl_seat event
        ▼
┌─ LAUNCHER (Wayland Client) ───────────────────────────────┐
│  touch_up() callback → handle_touch_tap(x, y)            │
│  → app icon hit-test → launch_app(app_id)                │
└───────────────────────────────────────────────────────────┘
```

---

## Servis Komut Tablosu

### servicemanager

| Kod | Komut | İstek | Yanıt |
|---|---|---|---|
| `1` | `CMD_REGISTER_SERVICE` | `str name`, `str path` | `i32 status` |
| `2` | `CMD_GET_SERVICE` | `str name` | `str path` |
| `3` | `CMD_LIST_SERVICES` | — | `str[] names` |

### surfaceflinger

| Kod | Komut | İstek | Yanıt |
|---|---|---|---|
| `1` | `CMD_REGISTER_SURFACE` | `i32 width`, `i32 height` | `i32 surface_id` |
| `2` | `CMD_COMMIT_FRAME` | `i32 id`, `raw pixels` | `i32 status` |
| `3` | `CMD_COMPOSITE` | — | `i32 status` |
| `4` | `CMD_SEND_INPUT_EVENT` *(Wayland)* | `i32 type`, `i32 code`, `i32 val` | `i32 status` |

### inputflinger

| Kod | Komut | İstek | Yanıt |
|---|---|---|---|
| `1` | `CMD_REGISTER_LISTENER` | `str socket_path` | `i32 status` |
| `2` | `CMD_SEND_INPUT_EVENT` | `i32 type`, `i32 code`, `i32 val` | `i32 status` |
| `3` | `CMD_GET_LAST_EVENT` | — | `i32 status`, `i32 t`, `i32 c`, `i32 v` |

### apigateway

| Kod | Komut | İstek | Yanıt |
|---|---|---|---|
| `1` | `CMD_GET_STATUS` | — | `str json` |
| `2` | `CMD_INJECT_TOUCH` | `i32 x`, `i32 y` | `i32 status` |
| `3` | `CMD_LIST_APPS` | — | `str[] names` |
| `4` | `CMD_LAUNCH_APP` | `str package` | `i32 pid` |

### powermanager

| Kod | Komut | İstek | Yanıt |
|---|---|---|---|
| `1` | `CMD_GET_BATTERY` | — | `i32 level`, `i32 charging` |
| `2` | `CMD_SCREEN_OFF` | — | `i32 status` |
| `3` | `CMD_SCREEN_ON` | — | `i32 status` |
| `4` | `CMD_GET_POWER_STATE` | — | `str state` |

---

## Hata Kodları

| Kod | Anlam |
|---|---|
| `0` | Başarılı (`IPC_SUCCESS`) |
| `-1` | Genel hata (`IPC_ERROR`) |
| `-2` | Servis bulunamadı |
| `-3` | Geçersiz komut kodu |
| `-4` | Payload parse hatası |
| `-5` | Surface ID geçersiz |

---

## Wayland IPC Uzantısı

Standart Orion IPC'nin Wayland modu için ek protokol uzantısı.

### surfaceflinger'a Touch Injection

```
Kod: 4 (CMD_SEND_INPUT_EVENT)
Yön: inputflinger → surfaceflinger

Payload (Parcel):
  int32 ev_type   (EV_KEY=1, EV_ABS=3)
  int32 ev_code   (ABS_X=0, ABS_Y=1, BTN_TOUCH=330)
  int32 ev_value  (koordinat veya 0/1)
```

### Uç-Uç Sıralama (Wayland Dokunma)

```
1. CMD_SEND_INPUT_EVENT(type=3, code=0, val=X)  → ABS_X akümüle
2. CMD_SEND_INPUT_EVENT(type=3, code=1, val=Y)  → ABS_Y akümüle
3. CMD_SEND_INPUT_EVENT(type=1, code=330, val=1) → DOWN
       → surface hit-test
       → wlr_seat_touch_notify_down()
       → wl_touch::down callback → handle_touch_tap()
4. CMD_SEND_INPUT_EVENT(type=1, code=330, val=0) → UP
       → wlr_seat_touch_notify_up()
       → wl_touch::up callback
```

---

## Test ve Hata Ayıklama

### Unix Socket'e Manuel Bağlanma

```bash
# Servis listesini sorgula (raw bytes)
python3 -c "
import socket, struct
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect('/tmp/servicemanager.sock')
# CMD_LIST_SERVICES = 3, data_size = 0
s.send(struct.pack('<iI', 3, 0))
print(s.recv(4096))
s.close()
"
```

### Touch Enjeksiyonu Test

```bash
# apigateway üzerinden (önerilir)
curl -s -X POST http://localhost:8080/api/v1/input/touch \
     -H "Content-Type: application/json" \
     -d '{"x": 540, "y": 1200, "action": "tap"}'

# Entegrasyon testi
./tests/integration/test_wayland_apps.sh
```

### Servis Durumu Kontrol

```bash
# Soketlerin var olup olmadığını kontrol et
ls -la /tmp/*.sock

# Bağlantı testi
nc -U /tmp/servicemanager.sock </dev/null && echo "UP" || echo "DOWN"

# Log izle (QEMU'da)
dmesg | tail -20
```

### IPC Hata Ayıklama İpuçları

| Sorun | Olası Sebep | Çözüm |
|---|---|---|
| `Connection refused` | Servis çalışmıyor | `init.rc`'de servis tanımlı mı? |
| `Permission denied` | Socket izni yanlış | `chmod 0666 /tmp/*.sock` |
| Yanıt gelmiyor | Deadlock / servis çöktü | `respawn=1` ile yeniden başlayacak |
| Yanlış payload | Parcel sırası hatalı | Yazma/okuma sırasını kontrol et |
| Wayland inject çalışmıyor | WAYLAND_ENABLED=0 | `wayland=1` kernel cmdline ekle |

---

*© 2026 mcsimon — Apache License 2.0*
