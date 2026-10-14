# Çoklu Dil ve Çeviri Rehberi (Translation Guide — i18n & l10n)

> **Konum:** `docs/TRANSLATION_GUIDE.md`  
> **Kapsam:** `.opk` Paketleri, Store Tanımları ve CLI Arayüzü

---

## 1. Genel Bakış

Orion OS uluslararası kullanıma açık bir ekosistemdir. Bir `.opk` uygulamasının adı, açıklaması, kullanıcı onay metinleri ve uygulama içi dili çoklu dil standardına göre yapılandırılabilir.

---

## 2. Paket İçi Dil Dosyaları (`locales/`)

Bir `.opk` paketinin kök dizininde opsiyonel olarak `locales/` klasörü bulunabilir:

```
com.orion.myapp/
├── manifest.toml
├── locales/
│   ├── tr.json
│   ├── en.json
│   ├── de.json
│   └── es.json
```

### Örnek Dil Dosyası (`locales/tr.json`):
```json
{
  "app_name": "Orion Not Defteri",
  "short_description": "Hızlı, minimalist ve güvenli not alma uygulaması.",
  "permissions": {
    "network": "Senkronizasyon için internet bağlantısı gereklidir.",
    "documents": "Notlarınızı Belgeler klasörüne kaydetmek için erişim istenir."
  }
}
```

### Örnek Dil Dosyası (`locales/en.json`):
```json
{
  "app_name": "Orion Notepad",
  "short_description": "Fast, minimalist and secure note taking application.",
  "permissions": {
    "network": "Internet access is required for note synchronization.",
    "documents": "Access requested to save notes into your Documents directory."
  }
}
```

---

## 3. `manifest.toml` Çoklu Dil Başlıkları

Uygulamanın mağazada birden fazla dilde doğru listelenmesi için Store API `locales/` dizinindeki anahtarları otomatik eşler. Sistem varsayılan dili bulamazsa `manifest.toml` içindeki ana metinlere geri döner (fallback).

---

## 4. `opk` CLI Çevirileri

CLI terminal mesajları Orion OS sistem dilini (`LANG` ve `LC_MESSAGES` ortam değişkenleri) okur:

* `LANG=tr_TR.UTF-8` -> Türkçe çıktılar
* `LANG=en_US.UTF-8` -> İngilizce çıktılar

CLI çevirilerine katkı sağlamak isteyen geliştiriciler `libs/libi18n` veya `mcpkg-cli` altındaki yerelleştirme dosyalarını güncelleyebilir.
