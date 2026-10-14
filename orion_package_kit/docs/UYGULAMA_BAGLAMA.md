# Uygulama Bağlama ve Sistem Entegrasyon Rehberi (App Linking & IPC)

> **Konum:** `docs/UYGULAMA_BAGLAMA.md`  
> **Kapsam:** Derin Bağlantılar (Deep Linking), MIME Türleri, Masaüstü Kısayolları ve Shell Entegrasyonu

---

## 1. Genel Bakış

Orion OS mobil kabuğu (Orion Shell / Launcher), uygulamaların birbirleriyle güvenli bir şekilde veri paylaşabilmesi ve harici bağlantılardan tetiklenebilmesi için standart XDG ve Orion IPC köprülerini kullanır.

Bu rehber, bir `.opk` uygulamasının:
1. Özel URL şemalarını (`myapp://open?id=123`),
2. Dosya türü (MIME Type) ilişkilerini (`.pdf`, `.png` vb.),
3. Masaüstü başlatıcı ve ikon bağlantılarını nasıl tanımlayacağını açıklar.

---

## 2. `.desktop` Entegrasyonu ve Parametre Geçişi

Bir uygulama kurulduğunda `~/.local/share/applications/{package_id}.desktop` dosyası otomatik üretilir:

```ini
[Desktop Entry]
Name=Orion Notepad
Exec=opk launch %u com.orion.notepad
Icon=orion-com.orion.notepad
Type=Application
Categories=Utility;TextEditor;
MimeType=text/plain;application/json;
X-Orion-Scheme=notepad;
```

### Parametre Açıklamaları:
* **`%u` Bayrağı:** Bir web bağlantısı veya dosya yolu tıklandığında argüman olarak `opk launch` aracılığıyla sandbox altındaki ikiliye iletilir.
* **`Icon=orion-{package_id}`:** `~/.local/share/icons/orion/{package_id}.png` dosyasını temsil eder ve sistem teması tarafından yüklenir.

---

## 3. Derin Bağlantı (Deep Linking) Tanımlama

Uygulamanızın özel bir URL şemasına (Custom Scheme) yanıt vermesini istiyorsanız `manifest.toml` içerisinde tanımlayabilirsiniz:

```toml
[build]
entry_x86_64 = "binaries/x86_64/myapp"
exec_type    = "elf"

[store]
category = "productivity"

# Derin bağlantı şemaları
[package.linking]
schemes = ["myapp", "notes"]
mime_types = ["text/plain", "text/markdown"]
```

---

## 4. Sandbox İçi Bağlantı Güvenliği

Bir bağlantı tıklandığında veya başka bir uygulama veri göndermek istediğinde:

1. **İzolasyon Koruması:** Çağrılan uygulama yine kendi sandbox'ı içerisinde başlatılır.
2. **IPC Denetimi:** `sandbox.policy` dosyasında `[system] ipc = "deny"` ise diğer uygulamaların bellek veya doğrudan soket erişimi engellenir; veri yalnızca CLI argümanı olarak güvenli bir şekilde aktarılır.
3. **Kullanıcı Onayı:** İlk defa bir harici URL şeması çağrıldığında sistem "Bu bağlantı Notepad uygulamasını açmak istiyor, izin veriyor musunuz?" uyarısını gösterir.
