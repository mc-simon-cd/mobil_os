# Orion OS — Ekran ve Kabuk Mimari Kararı

> Versiyon: 1.0.0 · Mimarisi: %100 Pure-Rust Surface & Shell API

---

## 1. Mimari Kararı Özeti

Orion OS v1.0 sürümünde, dış C kütüphanesi bağımlılıklarını (`libwlroots`, `libwayland-client`, `wayland-protocols`) taşımak yerine, **Saf Rust Ekran & Kabuk Mimarisi** (`ORION_SHELL_API_SPECIFICATION.md`) benimsenmiştir.

### Neden Saf Rust Mimari Seçildi?

1. **Sıfır Dış C Bağımlılığı (Zero C Dependencies)**:
   - C Wayland başlıkları ve `wayland-scanner` bağımlılıkları tamamen kaldırılmıştır.
   - Sistem `crt-static` ile tek parça statik ARM64 binary'si olarak derlenir.

2. **Düşük Hafıza Kullanımı & Hızlı Boot**:
   - Masaüstü sınıfı karmaşık Wayland protokolleri yerine, Orion OS mobil ihtiyaçları için optimize edilmiş `surfaceflinger` (Rust Compositor) ve `libgraphics` (Rust Rasterizer) katmanı kullanılır.

3. **IPC Tabanlı Yüzey & Kabuk Protokolü**:
   - Uygulamalar yüzey katmanı (surface) istemek için `libipc-rs` üzerinden `mobile.surfaceflinger` servisi ile iletişim kurar.
   - Katman birleştirme senkron olarak `surfaceflinger` tarafından yönetilir.

---

## 2. Mimari Referans Dokümanları

- [ORION_SHELL_API_SPECIFICATION.md](file:///home/can/Masaüstü/mobil_işletim_sistemi/ORION_SHELL_API_SPECIFICATION.md) — Pure-Rust Ekran & Kabuk API Protokol Şartnamesi.
- [RUST_MIGRATION_STATUS.md](file:///home/can/Masaüstü/mobil_işletim_sistemi/RUST_MIGRATION_STATUS.md) — Rust Geçiş Durum Raporu.

---

*© 2026 mcsimon — Apache License 2.0*
