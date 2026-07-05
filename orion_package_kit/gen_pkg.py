import zipfile
import os
import hashlib

def calculate_sha256(filepath):
    hasher = hashlib.sha256()
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hasher.update(chunk)
    return hasher.hexdigest()

def create_sample_pkg(name):
    pkg_name = f"{name}.opk"
    
    # Create files
    with open("manifest.toml", "w") as f:
        f.write('''[package]
id = "com.example.test"
name = "Test App"
version = "1.0.0"
description = "A test application"
author = "Tester"
author_email = "test@example.com"
min_os = "1.0.0"

[build]
entry_x86_64 = "binaries/x86_64/test"
exec_type = "elf"

[store]
category = "utilities"
rating = "ALL"
price_usd = 0.00
tier = "free"
''')

    with open("sandbox.policy", "w") as f:
        f.write('''[meta]
policy_version = "1.0"
strict_mode = false

[filesystem]
home_read = "allow"
home_write = "allow"
temp_access = "allow"
documents_read = "deny"
documents_write = "deny"
downloads_read = "deny"
system_read = "deny"
arbitrary_path = "deny"

[network]
internet = "deny"
localhost = "deny"
local_network = "deny"

[hardware]
camera = "deny"
microphone = "deny"
gpu_access = "deny"
usb_devices = "deny"
bluetooth = "deny"
location = "deny"

[system]
notifications = "deny"
clipboard_read = "deny"
clipboard_write = "deny"
autostart = "deny"
background_run = "deny"
ipc = "deny"
exec_subprocess = "deny"
''')

    os.makedirs("binaries/x86_64", exist_ok=True)
    with open("binaries/x86_64/test", "w") as f:
        f.write("#!/usr/bin/env bash\necho 'Hello from Orion Sandbox!'\n")

    # Generate checksums.sha256
    files_to_hash = ["manifest.toml", "sandbox.policy", "binaries/x86_64/test"]
    with open("checksums.sha256", "w") as f:
        for file in files_to_hash:
            h = calculate_sha256(file)
            f.write(f"{h}  {file}\n")
    
    # Generate dummy signature (64 bytes)
    with open("signature.ed25519", "wb") as f:
        f.write(b"0" * 64)
    
    # Create ZIP
    with zipfile.ZipFile(pkg_name, 'w') as zipf:
        for file in files_to_hash + ["checksums.sha256", "signature.ed25519"]:
            zipf.write(file)
            
    print(f"Created {pkg_name}")

if __name__ == "__main__":
    create_sample_pkg("test_app")
