# Orion Registry API — Teknik Spesifikasyon

> **Versiyon:** 1.0.0-draft  
> **Base URL:** `https://registry.orion.app/v1`  
> **Durum:** Tasarım Aşaması  
> **Auth:** `mcs_live_xxx` API anahtarları (SSO entegreli)

---

## İçindekiler

1. [Genel Mimari](#genel-mimari)
2. [Kimlik Doğrulama](#kimlik-doğrulama)
3. [Rate Limiting](#rate-limiting)
4. [Hata Formatı](#hata-formatı)
5. [Paket Endpointleri](#paket-endpointleri)
6. [Geliştirici Endpointleri](#geliştirici-endpointleri)
7. [Arama Endpointleri](#arama-endpointleri)
8. [Webhook Sistemi](#webhook-sistemi)
9. [Veri Modelleri](#veri-modelleri)
10. [Önbellek Stratejisi](#önbellek-stratejisi)

---

## Genel Mimari

```
İstemci (OS / CLI / Store UI)
          │
          ▼
    [CDN — Cloudflare]          ← statik dosyalar, .opk indirme
          │
          ▼
    [API Gateway]               ← rate limit, auth, routing
          │
    ┌─────┴──────┐
    ▼            ▼
[Registry API]  [Search API]    ← ayrı servisler
    │                │
    ▼                ▼
[Supabase DB]   [Meilisearch]   ← metadata + full-text arama
    │
    ▼
[Redis]                         ← rate limit, oturum, kısa ömürlü cache
    │
    ▼
[Object Storage]                ← .opk dosyaları (S3 uyumlu)
```

### Tasarım Kararları

| Karar | Neden |
|---|---|
| Versiyonlu API (`/v1/`) | Breaking change'lerde `/v2/` açılır, `/v1/` korunur |
| REST (GraphQL değil) | Önbellek dostu, CDN ile uyumlu |
| Supabase | Zaten ekosistemde mevcut |
| Meilisearch | Türkçe dil desteği, self-host edilebilir |
| CDN üzerinden indirme | Registry API'yi yük altına sokmaz |

---

## Kimlik Doğrulama

### API Anahtarı Türleri

| Tür | Prefix | Kapsam |
|---|---|---|
| Canlı ortam | `mcs_live_` | Tüm yazma işlemleri |
| Test ortam | `mcs_test_` | Sandbox, gerçek yayın yapılamaz |
| Salt okunur | `mcs_read_` | Yalnızca GET, arama |

### Kullanım

```http
Authorization: Bearer mcs_live_xxxxxxxxxxxxxxxxxxxx
Content-Type: application/json
```

### İzin Seviyeleri

```
public          → Auth gerektirmez (paket okuma, arama)
developer       → mcs_live_ gerekli (paket yükleme, güncelleme)
admin           → Özel admin anahtarı (paket silme, geliştirici banlama)
```

---

## Rate Limiting

Redis tabanlı, sliding window algoritması.

| Kapsam | Limit | Pencere |
|---|---|---|
| Public GET | 300 istek | 1 dakika |
| Authenticated GET | 1000 istek | 1 dakika |
| Upload (POST) | 10 istek | 1 saat |
| Arama | 100 istek | 1 dakika |

### Rate Limit Header'ları

```http
X-RateLimit-Limit: 300
X-RateLimit-Remaining: 247
X-RateLimit-Reset: 1736938200
Retry-After: 43          ← yalnızca 429 durumunda
```

---

## Hata Formatı

Tüm hatalar aynı yapıyı döner:

```json
{
  "error": {
    "code": "PACKAGE_NOT_FOUND",
    "message": "İnsan okunur hata mesajı",
    "details": {},
    "request_id": "req_01HX..."
  }
}
```

### Hata Kodları

| HTTP | Code | Açıklama |
|---|---|---|
| 400 | `INVALID_MANIFEST` | manifest.toml parse hatası |
| 400 | `INVALID_VERSION` | SemVer uyumsuz |
| 400 | `SIGNATURE_INVALID` | Ed25519 doğrulama başarısız |
| 400 | `CHECKSUM_MISMATCH` | Dosya bütünlüğü hatası |
| 401 | `UNAUTHORIZED` | Auth header eksik |
| 403 | `FORBIDDEN` | Yetersiz izin |
| 404 | `PACKAGE_NOT_FOUND` | Paket bulunamadı |
| 409 | `VERSION_CONFLICT` | Bu versiyon zaten mevcut |
| 413 | `PAYLOAD_TOO_LARGE` | 4GB limitini aşıyor |
| 422 | `POLICY_VIOLATION` | sandbox.policy geçersiz |
| 429 | `RATE_LIMITED` | Rate limit aşıldı |
| 500 | `INTERNAL_ERROR` | Sunucu hatası |

---

## Paket Endpointleri

---

### `GET /packages/{id}`

Bir paketin güncel kararlı versiyonunu döner.

**Parametreler:**

| Alan | Tür | Açıklama |
|---|---|---|
| `id` | string | Paket ID (örn. `com.orion.notepad`) |

**İstek:**

```http
GET /v1/packages/com.orion.notepad
```

**Yanıt `200 OK`:**

```json
{
  "id": "com.orion.notepad",
  "name": "Orion Notepad",
  "version": "1.2.3",
  "description": "Sade ve hızlı not alma uygulaması.",
  "author": "Mustafa",
  "license": "MIT",
  "rating": "ALL",
  "tier": "free",
  "price_usd": 0.00,
  "category": "productivity",
  "tags": ["notes", "text", "editor"],
  "min_os": "1.0.0",
  "trust_level": "verified",
  "downloads_total": 14820,
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-04-15T12:00:00Z",
  "assets": {
    "icon": "https://cdn.orion.app/packages/com.orion.notepad/icon.png",
    "screenshots": [
      "https://cdn.orion.app/packages/com.orion.notepad/screenshots/01.webp"
    ]
  },
  "latest": {
    "version": "1.2.3",
    "channel": "stable",
    "download_url": "https://cdn.orion.app/packages/com.orion.notepad/1.2.3/com.orion.notepad-1.2.3.opk",
    "size_bytes": 2048000,
    "sha256": "fcab2ce88c0e4a758b9a...",
    "published_at": "2026-04-15T12:00:00Z"
  }
}
```

---

### `GET /packages/{id}/versions`

Paketin tüm versiyon geçmişini döner.

**Query Parametreleri:**

| Parametre | Varsayılan | Açıklama |
|---|---|---|
| `channel` | `stable` | `stable` / `beta` / `nightly` / `all` |
| `limit` | `20` | Max 100 |
| `offset` | `0` | Sayfalama |

**İstek:**

```http
GET /v1/packages/com.orion.notepad/versions?channel=all&limit=5
```

**Yanıt `200 OK`:**

```json
{
  "id": "com.orion.notepad",
  "total": 12,
  "versions": [
    {
      "version": "1.2.3",
      "channel": "stable",
      "size_bytes": 2048000,
      "sha256": "fcab2ce88c0e4a758b9a...",
      "download_url": "https://cdn.orion.app/...",
      "changelog": "Hata düzeltmeleri ve performans iyileştirmeleri.",
      "published_at": "2026-04-15T12:00:00Z",
      "yanked": false
    },
    {
      "version": "1.2.2",
      "channel": "stable",
      "yanked": true,
      "yank_reason": "Kritik bellek sızıntısı",
      "published_at": "2026-04-01T08:00:00Z"
    }
  ]
}
```

---

### `GET /packages/{id}/versions/{version}`

Belirli bir versiyonun detayını döner.

```http
GET /v1/packages/com.orion.notepad/versions/1.2.3
```

**Yanıt** — yukarıdaki versiyon objesiyle aynı yapı, ek olarak:

```json
{
  "manifest": { /* manifest.toml içeriği JSON olarak */ },
  "permissions": {
    "filesystem": { "home_read": "allow", "documents_write": "ask" },
    "network": { "internet": "deny" },
    "hardware": { "camera": "deny" }
  }
}
```

---

### `POST /packages/upload`

Yeni paket veya yeni versiyon yükler. Auth zorunlu.

**İstek:**

```http
POST /v1/packages/upload
Authorization: Bearer mcs_live_xxx
Content-Type: multipart/form-data
```

```
Form Fields:
  package   → .opk dosyası (binary)
  changelog → string (opsiyonel, max 4096 karakter)
  channel   → "stable" | "beta" | "nightly"  (varsayılan: stable)
```

**Yükleme Akışı:**

```
1. Auth doğrula
2. Dosya boyutu kontrolü (max 4GB)
3. ZIP yapısı doğrula
4. manifest.toml parse et ve validate et
5. sandbox.policy validate et
6. checksums.sha256 doğrula
7. Ed25519 imza doğrula
8. Versiyon çakışması kontrolü
9. Güvenlik taraması (async)
10. Object Storage'a yükle
11. DB'ye kaydet
12. CDN'i invalidate et
13. Webhook'ları tetikle
```

**Yanıt `202 Accepted`:**

```json
{
  "upload_id": "upl_01HX...",
  "status": "processing",
  "package_id": "com.orion.notepad",
  "version": "1.3.0",
  "estimated_review_minutes": 5,
  "status_url": "/v1/uploads/upl_01HX.../status"
}
```

> **Not:** Yükleme asenkrondur. Güvenlik taraması tamamlanana kadar paket yayınlanmaz.

---

### `GET /uploads/{upload_id}/status`

Yükleme durumunu sorgular.

```http
GET /v1/uploads/upl_01HX.../status
Authorization: Bearer mcs_live_xxx
```

**Yanıt `200 OK`:**

```json
{
  "upload_id": "upl_01HX...",
  "status": "published",
  "stages": {
    "validation": "passed",
    "security_scan": "passed",
    "publishing": "completed"
  },
  "published_at": "2026-04-15T12:05:00Z",
  "download_url": "https://cdn.orion.app/..."
}
```

**Status Değerleri:** `processing` → `scanning` → `published` / `rejected`

---

### `POST /packages/{id}/versions/{version}/yank`

Bir versiyonu geri çeker. Dosya silinmez, indirilmez hale gelir. Auth zorunlu.

```http
POST /v1/packages/com.orion.notepad/versions/1.2.2/yank
Authorization: Bearer mcs_live_xxx
Content-Type: application/json

{
  "reason": "Kritik bellek sızıntısı"
}
```

**Yanıt `200 OK`:**

```json
{
  "yanked": true,
  "version": "1.2.2",
  "reason": "Kritik bellek sızıntısı",
  "yanked_at": "2026-04-20T09:00:00Z"
}
```

---

### `GET /packages/{id}/manifest`

Güncel versiyonun `manifest.toml` içeriğini ham metin olarak döner.

```http
GET /v1/packages/com.orion.notepad/manifest
```

**Yanıt `200 OK`:** `Content-Type: text/plain`

---

## Geliştirici Endpointleri

---

### `POST /developer/register`

Yeni geliştirici hesabı oluşturur. SSO entegreli.

```http
POST /v1/developer/register
Authorization: Bearer mcs_live_xxx

{
  "display_name": "Mustafa",
  "email": "dev@orion.app",
  "website": "https://orion.app"
}
```

**Yanıt `201 Created`:**

```json
{
  "developer_id": "dev_01HX...",
  "display_name": "Mustafa",
  "public_key_ed25519": null,
  "verified": false,
  "created_at": "2026-01-01T00:00:00Z"
}
```

---

### `POST /developer/keys`

Ed25519 public key kaydeder (imzalama için).

```http
POST /v1/developer/keys
Authorization: Bearer mcs_live_xxx

{
  "public_key": "base64_encoded_ed25519_public_key",
  "label": "Geliştirme Makinesi"
}
```

**Yanıt `201 Created`:**

```json
{
  "key_id": "key_01HX...",
  "label": "Geliştirme Makinesi",
  "fingerprint": "SHA256:abc123...",
  "created_at": "2026-01-01T00:00:00Z"
}
```

---

### `GET /developer/keys`

Kayıtlı public key'leri listeler.

```http
GET /v1/developer/keys
Authorization: Bearer mcs_live_xxx
```

---

### `DELETE /developer/keys/{key_id}`

Public key'i kaldırır. Bu key ile imzalanmış paketler `Signed` güven seviyesine düşer.

---

### `GET /developer/packages`

Geliştiriciye ait paketleri listeler.

```http
GET /v1/developer/packages
Authorization: Bearer mcs_live_xxx
```

**Yanıt `200 OK`:**

```json
{
  "packages": [
    {
      "id": "com.orion.notepad",
      "name": "Orion Notepad",
      "latest_version": "1.2.3",
      "downloads_total": 14820,
      "revenue_usd": 0.00,
      "status": "published"
    }
  ],
  "total": 1
}
```

---

### `GET /developer/analytics/{package_id}`

Paket analitikleri. Auth zorunlu, yalnızca kendi paketi.

```http
GET /v1/developer/analytics/com.orion.notepad?period=30d
```

**Yanıt `200 OK`:**

```json
{
  "package_id": "com.orion.notepad",
  "period": "30d",
  "downloads": {
    "total": 1240,
    "by_day": [
      { "date": "2026-04-01", "count": 42 },
      { "date": "2026-04-02", "count": 38 }
    ]
  },
  "active_installs": 9800,
  "ratings": {
    "average": 4.7,
    "count": 312
  },
  "revenue_usd": 0.00
}
```

---

## Arama Endpointleri

---

### `GET /search`

Full-text paket arama. Meilisearch tabanlı.

**Query Parametreleri:**

| Parametre | Tür | Varsayılan | Açıklama |
|---|---|---|---|
| `q` | string | — | Arama sorgusu (zorunlu) |
| `category` | string | `all` | Kategori filtresi |
| `rating` | string | `all` | `ALL` / `13+` / `17+` / `18+` |
| `tier` | string | `all` | `free` / `standard` / `premium` |
| `sort` | string | `relevance` | `relevance` / `downloads` / `updated` / `name` |
| `limit` | int | `20` | Max 50 |
| `offset` | int | `0` | Sayfalama |

**İstek:**

```http
GET /v1/search?q=not+alma&category=productivity&rating=ALL&sort=downloads
```

**Yanıt `200 OK`:**

```json
{
  "query": "not alma",
  "total": 8,
  "limit": 20,
  "offset": 0,
  "results": [
    {
      "id": "com.orion.notepad",
      "name": "Orion Notepad",
      "description": "Sade ve hızlı not alma uygulaması.",
      "version": "1.2.3",
      "rating": "ALL",
      "tier": "free",
      "price_usd": 0.00,
      "downloads_total": 14820,
      "icon_url": "https://cdn.orion.app/...",
      "trust_level": "verified",
      "_score": 0.97
    }
  ]
}
```

---

### `GET /search/suggest`

Otomatik tamamlama. Anlık arama için optimize edilmiş.

```http
GET /v1/search/suggest?q=not
```

**Yanıt `200 OK`:**

```json
{
  "suggestions": ["notepad", "notes sync", "not defteri"]
}
```

---

### `GET /packages/featured`

Öne çıkan paketler. Editoryal seçim, reklam değil.

```http
GET /v1/packages/featured?limit=10
```

---

### `GET /packages/trending`

Son 7 günde en çok indirilen paketler.

```http
GET /v1/packages/trending?period=7d&limit=10
```

---

## Webhook Sistemi

Geliştiriciler, paket olaylarını dinleyebilir.

### `POST /developer/webhooks`

```http
POST /v1/developer/webhooks
Authorization: Bearer mcs_live_xxx

{
  "url": "https://myserver.com/orion-hook",
  "events": ["package.published", "package.yanked", "review.completed"],
  "secret": "webhook_imza_sirri"
}
```

### Olaylar

| Olay | Tetikleyici |
|---|---|
| `package.published` | Paket yayınlandı |
| `package.yanked` | Versiyon geri çekildi |
| `upload.rejected` | Yükleme reddedildi |
| `review.completed` | Güvenlik taraması tamamlandı |
| `key.revoked` | Bir signing key iptal edildi |

### Webhook İmzası

Her webhook isteği `X-Orion-Signature` header'ı taşır:

```
X-Orion-Signature: sha256=hex(HMAC-SHA256(secret, payload))
```

---

## Veri Modelleri

### Package

```typescript
interface Package {
  id: string;              // com.orion.notepad
  name: string;
  description: string;
  author: string;
  author_email: string;    // registry'de saklanır, herkese açık değil
  license?: string;
  homepage?: string;
  category: Category;
  rating: ContentRating;
  tier: StoreTier;
  price_usd: number;
  tags: string[];
  min_os: string;
  trust_level: TrustLevel;
  downloads_total: number;
  created_at: string;      // ISO 8601
  updated_at: string;
}

type Category = "productivity" | "utilities" | "development" | ...;
type ContentRating = "ALL" | "13+" | "17+" | "18+";
type StoreTier = "free" | "standard" | "premium";
type TrustLevel = "verified" | "signed" | "unsigned";
```

### PackageVersion

```typescript
interface PackageVersion {
  version: string;         // SemVer
  channel: "stable" | "beta" | "nightly";
  size_bytes: number;
  sha256: string;
  download_url: string;
  changelog?: string;
  published_at: string;
  yanked: boolean;
  yank_reason?: string;
}
```

---

## Önbellek Stratejisi

| Endpoint | TTL | Notlar |
|---|---|---|
| `GET /packages/{id}` | 5 dakika | CDN cache, yayın sonrası invalidate |
| `GET /packages/{id}/versions` | 5 dakika | Yeni versiyon sonrası invalidate |
| `GET /search` | 2 dakika | Sorgu bazlı cache key |
| `GET /search/suggest` | 10 dakika | Çok değişmez |
| `GET /packages/trending` | 30 dakika | Pahalı sorgu |
| `GET /packages/featured` | 1 saat | Manuel güncelleme |

### Cache Key Şeması

```
registry:pkg:{id}                    → paket detayı
registry:pkg:{id}:versions           → versiyon listesi
registry:search:{hash(q+filters)}    → arama sonucu
registry:trending:{period}           → trending liste
```

---

*Bu doküman Orion ekosistemi dahilinde geliştirilmektedir.*  
*Sonraki: `opk-bekci-arch.md`*
