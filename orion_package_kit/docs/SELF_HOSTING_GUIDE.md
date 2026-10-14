# Kendi Registry Sunucunu Barındırma Rehberi (Self-Hosting Guide)

> **Konum:** `docs/SELF_HOSTING_GUIDE.md`  
> **Hedef:** Kurumsal Ağlar, Topluluklar ve Bağımsız Dağıtıcılar

---

## 1. Giriş

Orion ekosistemi merkeziyetçi değildir. Kurumlar, okullar veya özel geliştirici ekipleri genel Orion Store'a bağımlı olmadan kendi **Özel Paket Depolarını (Private Registry)** çalıştırabilirler.

`opk` istemcisi, standart `opk-registry-api.md` protokolünü uygulayan herhangi bir HTTP/HTTPS uç noktası ile çalışacak şekilde tasarlanmıştır.

---

## 2. Mimari Bileşenler

Örnek bir kendi kendine barındırılan sunucu yığını:

* **API Gateway / Tersine Proxy:** Nginx veya Caddy (SSL/TLS sonlandırma)
* **Registry Servisi:** Rust / Go / Node.js tabanlı REST API servisi
* **Paket Depolama:** S3 uyumlu nesne deposu (MinIO, Cloudflare R2 veya AWS S3)
* **Veritabanı:** PostgreSQL / Supabase
* **Arama Motoru (İsteğe bağlı):** Meilisearch

---

## 3. Docker Compose ile Hızlı Başlangıç

Aşağıdaki `docker-compose.yml` şablonu ile kendi registry ve depolama servislerinizi dakikalar içinde ayağa kaldırabilirsiniz:

```yaml
version: '3.8'

services:
  registry-api:
    image: orionos/registry-api:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://orion:secret@postgres:5432/registry
      - S3_ENDPOINT=http://minio:9000
      - S3_BUCKET=orion-packages
      - S3_ACCESS_KEY=minioadmin
      - S3_SECRET_KEY=minioadmin
    depends_on:
      - postgres
      - minio

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    volumes:
      - minio_data:/data

  postgres:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=orion
      - POSTGRES_PASSWORD=secret
      - POSTGRES_DB=registry
    volumes:
      - pg_data:/var/lib/postgresql/data

volumes:
  minio_data:
  pg_data:
```

Başlatmak için:
```bash
docker compose up -d
```

---

## 4. `opk` CLI'ı Özel Sunucuya Yönlendirme

Kendi sunucunuzdan paket yüklemek ve indirmek için iki yöntem mevcuttur:

### Yöntem A: Komut Satırı Bayrağı ile
```bash
# Özel sunucudan paket kurma:
opk install com.example.internalapp --registry https://packages.mycompany.internal/v1

# Özel sunucuya paket yayınlama:
export ORION_API_KEY="orion_live_custom_key_123"
opk publish ./myapp-1.0.0.opk --registry https://packages.mycompany.internal/v1
```

### Yöntem B: Genel Yapılandırma Dosyası ile (`~/.config/orion/config.toml`)
```toml
[registry]
default_url = "https://packages.mycompany.internal/v1"
cache_dir   = "~/.cache/orion/packages"
timeout_sec = 30
```
