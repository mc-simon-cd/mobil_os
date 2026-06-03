#!/usr/bin/env python3
# Copyright 2026 mcsimon
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import os
import json
import urllib.request

def main():
    # 1. Determine locale directory
    locale_dir = os.environ.get("LOCALE_DIR", "/system/usr/share/locale")
    if not os.path.exists(locale_dir):
        # Fallback to local repo directories for host/local development
        possible_paths = [
            "../../usr/share/locale",
            "../../../usr/share/locale",
            "../../../../rootfs/system/usr/share/locale",
            "./locale"
        ]
        for p in possible_paths:
            abs_p = os.path.abspath(os.path.join(os.path.dirname(__file__), p))
            if os.path.exists(abs_p):
                locale_dir = abs_p
                break

    # 2. Get language from environment
    lang = os.environ.get("LANG", "en").split('.')[0].split('_')[0]
    locale_path = os.path.join(locale_dir, f"{lang}.txt")
    if not os.path.exists(locale_path):
        locale_path = os.path.join(locale_dir, "en.txt")

    # 3. Read translation entries
    translations = {}
    if os.path.exists(locale_path):
        try:
            with open(locale_path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line or line.startswith("#"):
                        continue
                    if "=" in line:
                        parts = line.split("=", 1)
                        translations[parts[0].strip()] = parts[1].strip()
        except Exception as e:
            print(f"[WARN] Failed to parse translation file {locale_path}: {e}")

    def get_text(key):
        return translations.get(key, key)

    # 4. Resolve API Gateway endpoint
    api_port = os.environ.get("API_PORT", os.environ.get("PORT", "8080"))
    url = f"http://localhost:{api_port}/api/power"

    # 5. Query system status and print localized report
    try:
        req = urllib.request.Request(url)
        with urllib.request.urlopen(req, timeout=2) as response:
            res_body = response.read().decode("utf-8")
            data = json.loads(res_body)
            
            battery = data.get("battery_level", 0)
            mode = data.get("power_mode", "balanced")
            
            battery_lbl = get_text("power.battery")
            mode_lbl = get_text("power.mode")
            mode_trans_key = f"power.mode.{mode}"
            mode_trans = get_text(mode_trans_key)
            
            print("==========================================")
            print(f"📊 {get_text('launcher.title')} - System Report")
            print("------------------------------------------")
            print(f"🔋 {battery_lbl}: {battery}%")
            print(f"⚡ {mode_lbl}: {mode_trans} ({mode})")
            print("==========================================")
    except Exception as e:
        print(f"❌ Error querying API Gateway at {url}: {e}")
        # Print offline mock report if API is not running to avoid crash in basic testing
        print("==========================================")
        print(f"📊 {get_text('launcher.title')} - System Report (OFFLINE)")
        print("------------------------------------------")
        print(f"🔋 {get_text('power.battery')}: N/A")
        print(f"⚡ {get_text('power.mode')}: N/A")
        print("==========================================")

if __name__ == "__main__":
    main()
