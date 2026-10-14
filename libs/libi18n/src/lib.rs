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

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::sync::Mutex;

static G_TRANSLATIONS: Mutex<Option<HashMap<String, CString>>> = Mutex::new(None);

/// Initializes the i18n localization system.
/// Loads translation file from "<locale_dir>/<locale_name>.txt".
/// Returns 0 on success, or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn i18n_init(
    locale_dir: *const c_char,
    locale_name: *const c_char,
) -> c_int {
    // 1. Clean up existing resources if initialized
    i18n_free();

    if locale_dir.is_null() || locale_name.is_null() {
        return -1;
    }

    let dir = match CStr::from_ptr(locale_dir).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let name = match CStr::from_ptr(locale_name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    // 2. Construct file path
    let file_path = format!("{}/{}.txt", dir, name);
    let path = Path::new(&file_path);

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return -1,
    };

    // 3. Parse file line by line
    let mut map = HashMap::new();
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();
        // Ignore comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(separator_idx) = trimmed.find('=') {
            let key = trimmed[..separator_idx].trim();
            let value = trimmed[separator_idx + 1..].trim();
            if !key.is_empty() {
                if let Ok(c_val) = CString::new(value) {
                    map.insert(key.to_string(), c_val);
                }
            }
        }
    }

    let mut guard = G_TRANSLATIONS.lock().unwrap();
    *guard = Some(map);

    0
}

/// Returns the localized string for the given key.
/// If the key is not found, returns the key itself as a fallback.
#[no_mangle]
pub unsafe extern "C" fn i18n_get(key: *const c_char) -> *const c_char {
    if key.is_null() {
        return key;
    }
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return key,
    };

    let guard = G_TRANSLATIONS.lock().unwrap();
    if let Some(ref map) = *guard {
        if let Some(c_val) = map.get(key_str) {
            return c_val.as_ptr();
        }
    }
    key // Graceful fallback to the key itself
}

/// Frees all loaded translation memory resources.
#[no_mangle]
pub unsafe extern "C" fn i18n_free() {
    let mut guard = G_TRANSLATIONS.lock().unwrap();
    *guard = None;
}
