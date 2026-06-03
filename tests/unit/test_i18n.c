/*
 * Copyright 2026 mcsimon
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "i18n.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

int main(void) {
    printf("\n🧪 Starting i18n Localization Engine Unit Tests...\n");

    const char *locale_dir = "rootfs/system/usr/share/locale";

    // 1. Verify pre-init fallback
    printf("  [TEST] Verifying pre-initialization fallback...\n");
    const char *fallback_val = i18n_get("launcher.title");
    assert(strcmp(fallback_val, "launcher.title") == 0);
    printf("  ✅ Pre-init fallback checked.\n");

    // 2. Load Turkish Locale (tr)
    printf("  [TEST] Loading Turkish locale (tr)...\n");
    int init_tr = i18n_init(locale_dir, "tr");
    assert(init_tr == 0);
    
    // Check Turkish translations
    assert(strcmp(i18n_get("launcher.title"), "Uygulama Başlatıcı") == 0);
    assert(strcmp(i18n_get("settings.title"), "Ayarlar") == 0);
    assert(strcmp(i18n_get("power.battery"), "Pil") == 0);
    assert(strcmp(i18n_get("power.mode.powersave"), "Güç Tasarrufu") == 0);
    assert(strcmp(i18n_get("non.existent.key"), "non.existent.key") == 0); // key fallback
    printf("  ✅ Turkish translations verified.\n");

    // 3. Load English Locale (en) - Re-init test
    printf("  [TEST] Loading English locale (en) to test re-initialization...\n");
    int init_en = i18n_init(locale_dir, "en");
    assert(init_en == 0);

    // Check English translations
    assert(strcmp(i18n_get("launcher.title"), "App Launcher") == 0);
    assert(strcmp(i18n_get("settings.title"), "Settings") == 0);
    assert(strcmp(i18n_get("power.battery"), "Battery") == 0);
    assert(strcmp(i18n_get("power.mode.powersave"), "Power Saver") == 0);
    printf("  ✅ English translations verified.\n");

    // 4. Load Spanish Locale (es)
    printf("  [TEST] Loading Spanish locale (es)...\n");
    int init_es = i18n_init(locale_dir, "es");
    assert(init_es == 0);
    assert(strcmp(i18n_get("settings.title"), "Ajustes") == 0);
    assert(strcmp(i18n_get("dialer.title"), "Teléfono") == 0);
    printf("  ✅ Spanish translations verified.\n");

    // 5. Load Russian Locale (ru)
    printf("  [TEST] Loading Russian locale (ru)...\n");
    int init_ru = i18n_init(locale_dir, "ru");
    assert(init_ru == 0);
    assert(strcmp(i18n_get("settings.title"), "Настройки") == 0);
    assert(strcmp(i18n_get("messaging.title"), "Сообщения") == 0);
    printf("  ✅ Russian (Cyrillic UTF-8) translations verified.\n");

    // 6. Test Error Handling (non-existent language)
    printf("  [TEST] Loading non-existent locale (xx) to check error handling...\n");
    int init_err = i18n_init(locale_dir, "xx");
    assert(init_err == -1);
    // Should fall back since 'xx' failed to load
    assert(strcmp(i18n_get("launcher.title"), "launcher.title") == 0);
    printf("  ✅ Error handling and fallback verified.\n");

    // 7. Cleanup Resources
    printf("  [TERM] Freeing i18n resources...\n");
    i18n_free();
    assert(strcmp(i18n_get("launcher.title"), "launcher.title") == 0);
    printf("  ✅ i18n freed successfully.\n");

    printf("🎉 i18n Localization Engine Unit Tests Passed Successfully!\n\n");
    return 0;
}
