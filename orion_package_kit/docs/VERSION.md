# Versiyonlama ve Sürüm Yönetimi Rehberi (Versioning Guide)

> **Konum:** `docs/VERSION.md`  
> **Standart:** Semantic Versioning 2.0.0 (SemVer)

---

## 1. Genel Versiyonlama Standardı

Orion Package Kit (`opk`) ve dağıtılan tüm `.opk` paketleri kesinlikle **Semantic Versioning 2.0.0** standardına uymak zorundadır:

```
MAJOR . MINOR . PATCH [- ÖN_YAYIN] [+ DERLEME_METADATA]

Örnekler:
  1.0.0
  2.1.3
  1.0.0-beta.1
  2.0.0-rc.2+build.20261013
```

---

## 2. Sürüm Artırma Kuralları

| Bileşen | Ne Zaman Artırılır? | Örnek |
|---|---|---|
| **MAJOR (Ana Sürüm)** | Geriye dönük uyumsuz API veya paket formatı değişiklikleri olduğunda. | `1.4.2` -> `2.0.0` |
| **MINOR (Alt Sürüm)** | Geriye dönük uyumlu yeni özellikler, komutlar eklendiğinde. | `1.4.2` -> `1.5.0` |
| **PATCH (Yama Sürümü)** | Geriye dönük uyumlu hata ve güvenlik düzeltmeleri yapıldığında. | `1.4.2` -> `1.4.3` |

---

## 3. Ön Sürüm Etiketleri

* `-alpha.X`: Dahili test ve ilk geliştirici önizleme sürümleri.
* `-beta.X`: Özellik dondurma aşamasına gelmiş, halka açık test sürümleri.
* `-rc.X`: (Release Candidate) Yayına aday, stabilite testleri süren sürümler.

---

## 4. `min_os` ve `max_os` Eşleşme Kuralları

`manifest.toml` dosyası içerisindeki işletim sistemi sürüm kısıtlamaları `opk` tarafından şu şekilde denetlenir:

```toml
[package]
min_os = "1.0.0"       # Kurulum için gereken minimum Orion OS sürümü
max_os = "2.0.0"       # Opsiyonel: Uyumsuzluk biliniyorsa üst sınır
```

* Eğer `mevcut_os < min_os` ise: `opk install` kurulumu durdurur ve kullanıcıyı uyarır:
  ```
  HATA: Bu uygulama Orion OS 1.0.0 veya üzerini gerektirir (Mevcut: 0.9.2).
  ```
* `max_os` boş bırakılırsa üst sınır olmadığı kabul edilir.

---

## 5. Değişmezlik İlkesi (Immutability)

Orion Registry'ye yayınlanan bir paket sürümü **asla üzerine yazılamaz (immutable)**.  
Aynı `id` ve `version` kombinasyonuna sahip yeni bir dosya yüklenemez; bir düzeltme gerekiyorsa sürüm numarası artırılmalıdır (`1.0.0` -> `1.0.1`).
