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

#ifndef I18N_H
#define I18N_H

// Initializes the i18n localization system.
// Loads translation file from "<locale_dir>/<locale_name>.txt".
// Returns 0 on success, or -1 on error.
int i18n_init(const char *locale_dir, const char *locale_name);

// Returns the localized string for the given key.
// If the key is not found, returns the key itself as a fallback.
const char *i18n_get(const char *key);

// Frees all loaded translation memory resources.
void i18n_free(void);

#endif // I18N_H
