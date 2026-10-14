# Orion OS — IPC (Süreçler Arası İletişim) Kılavuzu

> Versiyon: 1.0.0 · Tüm IPC, POSIX Unix domain socket üzerinden `libipc-rs` kütüphanesi ile senkron istek/yanıt modeliyle çalışır.

---

## İçindekiler

1. [Genel Bakış](#genel-bakış)
2. [Protokol Formatı](#protokol-formatı)
3. [Parcel API (`libipc-rs`)](#parcel-api-libipc-rs)
4. [Tam Bir İşlem Akışı](#tam-bir-işlem-akışı)
5. [Servis Komut Tablosu](#servis-komut-tablosu)
6. [Hata Kodları](#hata-kodları)

---

## Genel Bakış

Orion OS'un IPC sistemi Android Binder'dan esinlenmiş, Unix socket tabanlı istek/yanıt protokolüdür.

- **Taşıma:** POSIX Unix domain socket (`AF_UNIX`, `SOCK_STREAM`)
- **Model:** Senkron istek/yanıt (Connect -> Send Header & Parcel Payload -> Receive Reply -> Close)
- **Serileştirme:** `libipc-rs` Parcel — dinamik büyüyen byte tamponu
- **Header:** 8 byte sabit header (`code: i32`, `data_size: u32`) + değişken payload

```
İstemci (Uygulama)                   Sunucu (Servis)
   │── connect() ─────────────────────────►│
   │── write(header + parcel payload) ────►│
   │◄─ read(header + parcel reply) ────────│
   │── close() ───────────────────────────►│
```

---

## Protokol Formatı

Her mesaj iki bölümden oluşur:

```
┌──────────────────────────────────────────┐
│            IPC HEADER (8 byte)           │
│  int32_t  code        (4 byte, LE)       │
│  uint32_t data_size   (4 byte, LE)       │
├──────────────────────────────────────────┤
│            PAYLOAD (data_size byte)      │
│  Parcel ile serileştirilmiş veriler      │
└──────────────────────────────────────────┘
```

---

## Parcel API (`libipc-rs`)

```rust
use libipc_rs::{ipc_connect, ipc_send_transaction, Parcel};

// 1. Parcel oluştur ve veri yaz
let mut data = Parcel::new();
data.write_string("mobile.powermanager");
data.write_i32(100);

// 2. İletişim kur
let mut stream = ipc_connect("/tmp/servicemanager.sock").expect("Servise bağlanılamadı");
let mut reply = ipc_send_transaction(&mut stream, 2, &data).expect("İşlem başarısız");

// 3. Yanıtı oku
reply.reset_read();
let service_path = reply.read_string().expect("String okunamadı");
```

---

## Servis Komut Tablosu

### `servicemanager` (`/tmp/servicemanager.sock`)

| Kod | Komut | İstek Payload | Yanıt Payload |
| :--- | :--- | :--- | :--- |
| `1` | `CMD_REGISTER_SERVICE` | `string name`, `string path` | `i32 status` |
| `2` | `CMD_GET_SERVICE` | `string name` | `string path` |
| `3` | `CMD_LIST_SERVICES` | — | `string[] names` |

### `surfaceflinger` (`/tmp/surfaceflinger.sock`)

| Kod | Komut | İstek Payload | Yanıt Payload |
| :--- | :--- | :--- | :--- |
| `1` | `CMD_ALLOC_SURFACE` | `i32 width`, `i32 height`, `i32 format` | `i32 surface_id` |
| `2` | `CMD_COMPOSITE` | — | `i32 status` |
| `3` | `CMD_DESTROY_SURFACE` | `i32 surface_id` | `i32 status` |

### `inputflinger` (`/tmp/inputflinger.sock`)

| Kod | Komut | İstek Payload | Yanıt Payload |
| :--- | :--- | :--- | :--- |
| `1` | `CMD_REGISTER_LISTENER` | `string socket_path` | `i32 status` |
| `2` | `CMD_SEND_INPUT_EVENT` | `i32 type`, `i32 code`, `i32 value` | `i32 status` |

### `statusbar` (`/tmp/statusbar.sock`)

| Kod | Komut | İstek Payload | Yanıt Payload |
| :--- | :--- | :--- | :--- |
| `1` | `CMD_SHOW_NOTIFICATION` | `string message` | `i32 status` |

---

## Hata Kodları

| Kod | Anlam |
| :--- | :--- |
| `0` | Başarılı (`IPC_SUCCESS`) |
| `-1` | Genel hata (`IPC_ERROR`) |
| `-2` | Servis bulunamadı |
| `-3` | Geçersiz komut kodu |

---

*© 2026 mcsimon — Apache License 2.0*
