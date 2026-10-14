# Kullanım Şartları ve Ekosistem Kuralları — Orion Package Kit (`opk`)

> **Son Güncelleme:** 2026-10-13  
> **Kapsam:** `.opk` Ekosistemi Geliştiricileri, Dağıtıcıları ve Kullanıcıları  
> **Konum:** `docs/SARTLAR.md`

---

## 1. Şartların Kabulü

Orion Package Kit (`opk`) altyapısını kullanarak paket oluşturan, imzalayan, yayınlayan veya kuran her birey ve kurum bu kullanım şartlarını kabul etmiş sayılır. Şartlara uymayan geliştiricilerin paketleri Orion Registry üzerinden dağıtılamaz.

---

## 2. Geliştirici Sorumlulukları ve İlkeler

1. **Zararlı Yazılım Yasağı:** Paketler; truva atı, casus yazılım (spyware), fidye yazılımı (ransomware), yetkisiz kripto para madencisi veya zararlı kod içeremez.
2. **Sandbox Sınırlarına Saygı:** Linux çekirdek açıklarını istismar etmek, Seccomp/AppArmor kısıtlamalarını hileyle atlatmaya çalışmak veya root yetkisi elde etmeye kalkışmak kesinlikle yasaktır.
3. **Dürüst İzin Beyanı:** `manifest.toml` ve `sandbox.policy` dosyalarında uygulamanın talep ettiği tüm izinler dürüstçe açıklanmalıdır. Örneğin, basit bir not defteri uygulamasının kamera veya mikrofon izni talep etmesi ve bunu gizlemesi kabul edilemez.
4. **Anahtar Güvenliği:** Geliştiriciler kendilerine ait `private.key` dosyasının güvenliğinden bizzat sorumludur. Anahtarın çalınması durumunda registry üzerinden acil anahtar iptali (key revocation) bildirilmelidir.

---

## 3. Kullanıcı Gizliliği ve Veri Güvenliği

* Uygulamalar, kullanıcının açık rızası olmaksızın kişisel veri toplayamaz, analiz amacıyla üçüncü taraflarla paylaşamaz.
* Uygulamanın disk erişimi yalnızca kendi dizini (`~/.local/share/{app_id}`) ile sınırlı kalmalıdır.

---

## 4. Paket Kaldırma ve Müdahale Hakları

Orion Güvenlik Kurulu (Orion Security Board):
* Kritik güvenlik açığı barındıran veya kötü niyetli olduğu kanıtlanan paketleri derhal "Geri Çekilmiş" (Yanked) ilan edebilir.
* Bekçi (Gatekeeper) servisi aracılığıyla kullanıcı cihazlarına güvenlik uyarısı iletebilir.
* Kuralları ihlal eden geliştirici API anahtarlarını askıya alabilir veya tamamen kapatabilir.

---

## 5. Sorumluluk Reddi (Disclaimer)

ORION PACKAGE KIT YAZILIMI "OLDUĞU GİBİ" (AS-IS) SAĞLANMAKTADIR. YAZILIMIN KULLANIMINDAN DOĞABİLECEK VERİ KAYBI, DONANIM HASARI VEYA KESİNTİLERDEN PROJE GELİŞTİRİCİLERİ VE KATKIDA BULUNANLAR HUKUKİ VEYA CEZAİ OLARAK SORUMLU TUTULAMAZ.
