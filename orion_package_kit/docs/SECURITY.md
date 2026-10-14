# Güvenlik Politikası ve Tehdit Modeli (Security Policy) — `opk`

> **Konum:** `docs/SECURITY.md`  
> **İletişim:** `security@orion.app`  
> **Durum:** Aktif Politika

---

## 1. Güvenlik Felsefesi: Sıfır Güven (Zero Trust)

Orion Package Kit (`opk`), kullanıcı cihazını üçüncü parti yazılımların potansiyel zararlarından korumak için tasarlanmıştır. Tüm paketler doğası gereği güvenilmeyen (untrusted) kabul edilir ve çok katmanlı kriptografik ve çekirdek seviyesinde kontrollerden geçer.

---

## 2. Tehdit Modeli ve Savunma Mekanizmaları

| Tehdit Türü | Savunma Mekanizması | Etki & Sonuç |
|---|---|---|
| **Paketin Yolda Değiştirilmesi (MitM)** | Ed25519 Dijital İmzası + SHA-256 Checksums | Paket derhal reddedilir, kurulum kesin olarak durdurulur. |
| **Kötü Amaçlı Sistem Çağrısı (Syscall)** | Seccomp BPF Filtresi | `ptrace`, `kexec_load`, izinsiz `socket` çağrıları engellenir. |
| **İzinsiz Dosya Erişimi (Veri Sızıntısı)** | AppArmor LSM + Mount Namespace | Uygulama yalnızca `~/.local/share/{id}` altını görebilir. |
| **Ağ Üzerinden Veri Kaçırma** | Network Namespace (`CLONE_NEWNET`) | `internet = deny` ise uygulama interneti hiç göremez. |
| **Kaynak Tüketimi / DoS Saldırısı** | cgroups v2 (`memory.max`, `cpu.max`) | Cihazın kilitlenmesi veya aşırı ısınması önlenir. |
| **Root Yetkisine Sızma** | Rootless User Namespaces (`CLONE_NEWUSER`) | Uygulama container içinde sahte root olsa dahi hostta normal kullanıcıdır. |

---

## 3. Desteklenen Sürümler

Aşağıdaki sürümler aktif olarak güvenlik yamaları almaktadır:

| Sürüm | Güvenlik Desteği |
|---|---|
| `0.2.x` | ✅ Aktif Destek |
| `0.1.x` | ⚠️ Yalnızca Kritik Yamalar |
| `< 0.1.0` | ❌ Desteklenmiyor |

---

## 4. Güvenlik Açığı Bildirimi (Vulnerability Disclosure)

Eğer `opk`, sandbox motoru veya kriptografik imza doğrulamasında bir güvenlik açığı bulursanız:

1. **Kamuya Açık Paylaşmayın:** Lütfen GitHub üzerinde açık issue veya sosyal medyada paylaşım yapmayın.
2. **E-posta ile İletişim:** Detayları, PoC (Proof of Concept) kodunu ve etkilenen bileşeni `security@orion.app` adresine gönderin.
3. **PGP Anahtarı:** Hassas veriler için Orion Güvenlik Kurulu'nun PGP açık anahtarını kullanabilirsiniz.
4. **Yanıt Süresi:** Güvenlik ekibimiz bildirimlere en geç **48 saat** içinde resmi yanıt verir ve yama süreci başlatılır.

---

## 5. Sorumlu İfşa (Responsible Disclosure) Politikası

Açığın bildirilmesinden sonra yamayı hazırlayıp yayınlamamız için lütfen **90 gün** süre tanıyınız. Güvenlik açığını bildiren araştırmacılara teşekkür metninde ve sürüm notlarında resmi olarak yer verilir.
