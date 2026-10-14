# Veri Tasarrufu ve Önbellek Yönetimi Rehberi (Data Saving Guide)

> **Hedef:** Orion OS Mobil Cihazlar ve Sınırlı Hücresel Ağlar  
> **Konum:** `docs/DATA_SAVING_GUIDE.md`

---

## 1. Giriş

Orion OS mobil bir işletim sistemi olduğu için kullanıcıların hücresel veri (4G/5G) ve batarya tüketimini korumak en kritik önceliklerden biridir. `opk`, paket indirme ve güncelleme süreçlerinde bant genişliğini optimize eden akıllı bir veri tasarrufu mimarisine sahiptir.

---

## 2. Temel Veri Tasarrufu Stratejileri

```
┌─────────────────────────────────────────────────────────┐
│              Veri Tasarrufu Katmanları                  │
├─────────────────────────────────────────────────────────┤
│ 1. Yerel Paket Önbelleği (Local Caching)                 │
│ 2. SHA-256 Doğrulamalı Tekrar İndirme Koruması          │
│ 3. Zstandard (Zstd) Arşivleme Desteği                    │
│ 4. Ağ Durumu Tespiti (Metered / Unmetered Connection)   │
└─────────────────────────────────────────────────────────┘
```

---

## 3. Yerel Paket Önbelleği (`CacheManager`)

Bir paket indirildiğinde doğrudan silinmez; `~/.cache/orion/packages/{id}/{version}.opk` adresinde saklanır.

* **Aynı Sürümün Tekrar İndirilmesini Önleme:** Bir paket ID'si ve sürümü önbellekte varsa ve SHA-256 hash'i eşleşiyorsa indirme isteği tamamen atlanır (0 bayt veri tüketimi).
* **Önbellek Temizleme:**
  Kullanıcı veya sistem disk alanını boşaltmak istediğinde eski paket önbelleklerini temizleyebilir:
  ```bash
  rm -rf ~/.cache/orion/packages/
  ```

---

## 4. Sıkıştırma ve Paket Optimizasyonu

Geliştiricilerin paketlerini oluştururken bant genişliği tasarrufu sağlamak için uygulaması gereken kurallar:

1. **Gereksiz Dosyaları Paketlememek:** `.git`, test ikilileri, geçici dosyalar ve derleme artıkları `.opk` içerisine dahil edilmemelidir.
2. **Görsel Optimizasyonu:** `assets/icon.png` 512x512 PNG olarak optimize edilmeli (`oxipng` veya `pngquant`), ekran görüntüleri ise WebP/AVIF formatında tutulmalıdır.
3. **Binary Strip:** ELF ikilileri yayınlanmadan önce `strip --strip-unneeded` ile debug sembollerinden arındırılmalıdır (ortalama %60-%80 boyut küçülmesi sağlar).

---

## 5. Hücresel Ağlarda İndirme Politikası

Gelecek Orion OS güncellemelerinde entegre edilecek olan ağ farkındalığı politikası:

* **Tarifeli Ağ (Metered Connection):** Hücresel veri açıkken 50 MB üzerindeki güncellemeler otomatik olarak duraklatılır ve kullanıcı onayına sunulur.
* **Wi-Fi (Unmetered):** Arka plan güncellemeleri yalnızca Wi-Fi bağlantısı ve şarj durumu %20'nin üzerindeyken otomatik yürütülür.
