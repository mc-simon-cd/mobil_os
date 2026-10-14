# Orion OS — Servisler Referans Kılavuzu

> Versiyon: 1.0.0 · Tüm servisler %100 Rust dilinde yazılmıştır, `init.rc` tarafından başlatılır ve POSIX Unix Domain Socket üzerinden `libipc-rs` kullanır.

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

**Dil:** Rust  
**İkili:** `/system/bin/servicemanager`  
**Socket:** `/tmp/servicemanager.sock`  

### Görev

Sistem genelinde isim çözümleyici (Service Directory). Diğer servisler başladıklarında adlarını ve socket yollarını `servicemanager`'a kaydeder. İstemciler, bir servise bağlanmadan önce `servicemanager`'dan socket yolunu öğrenir.

### Komutlar

| Kod | Ad | İstek Payload | Yanıt Payload |
| :--- | :--- | :--- | :--- |
| `1` | `CMD_REGISTER_SERVICE` | `string name`, `string socket_path` | `int32 status` (0=OK) |
| `2` | `CMD_GET_SERVICE` | `string name` | `string socket_path` |
| `3` | `CMD_LIST_SERVICES` | — | `string[] names` |

---

## surfaceflinger

**Dil:** Rust  
**İkili:** `/system/bin/surfaceflinger`  
**Socket:** `/tmp/surfaceflinger.sock`  
**Kayıt adı:** `mobile.surfaceflinger`

### Görev

Yazılım Tabanlı Ekran Kompozitörü (Software Display Compositor). İstemci uygulamalardan gelen grafik katmanlarını (surface) harmanlayarak master PPM ekran dosyasına (`out/display_composited.ppm`) aktarır.

### Komutlar

| Kod | Ad | İstek Payload | Yanıt Payload |
| :--- | :--- | :--- | :--- |
| `1` | `CMD_ALLOC_SURFACE` | `int32 width`, `int32 height`, `int32 format` | `int32 surface_id` |
| `2` | `CMD_COMPOSITE` | — | `int32 status` (0=OK) |
| `3` | `CMD_DESTROY_SURFACE` | `int32 surface_id` | `int32 status` |

---

## inputflinger

**Dil:** Rust  
**İkili:** `/system/bin/inputflinger`  
**Socket:** `/tmp/inputflinger.sock`  
**Kayıt adı:** `mobile.inputflinger`

### Görev

Dokunma, klavye ve girdi olaylarını (input events) alır ve aktif dinleyici uygulamalara iletir.

### Komutlar

| Kod | Ad | İstek Payload | Yanıt Payload |
| :--- | :--- | :--- | :--- |
| `1` | `CMD_REGISTER_LISTENER` | `string socket_path` | `int32 status` |
| `2` | `CMD_SEND_INPUT_EVENT` | `int32 type`, `int32 code`, `int32 value` | `int32 status` |
| `3` | `CMD_GET_LAST_EVENT` | — | `int32 status`, `int32 type`, `int32 code`, `int32 value` |

---

## apigateway

**Dil:** Rust  
**İkili:** `/system/bin/apigateway`  
**Socket:** `/tmp/apigateway.sock`  
**Kayıt adı:** `mobile.apigateway`  
**HTTP Port:** `8080`

### Görev

HTTP REST arayüzü ile sistem servisleri arasında köprü kurar. Test ve web arayüzlerinin HTTP üzerinden işlem yapmasını sağlar.

---

## powermanager

**Dil:** Rust  
**İkili:** `/system/bin/powermanager`  
**Socket:** `/tmp/powermanager.sock`  
**Kayıt adı:** `mobile.powermanager`

### Görev

Batarya seviyesi takibi, ekran durumu ve güç profili (`powersave`, `balanced`, `performance`) yönetimi.

---

## apprunner

**Dil:** Rust  
**İkili:** `/system/bin/apprunner`  
**Socket:** `/tmp/apprunner.sock`  
**Kayıt adı:** `mobile.apprunner`

### Görev

Uygulama yaşam döngüsü yöneticisi. Paket ikililerini başlatır ve yönetir.

---

*© 2026 mcsimon — Apache License 2.0*
