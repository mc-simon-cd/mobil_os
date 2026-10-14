# Terms of Service & Ecosystem Rules — Orion Package Kit

> **Last Updated:** 2026-10-13  
> **Applies to:** Developers, Distributors, and Users of the `.opk` Ecosystem  
> **Location:** `docs/TERMS.md`

---

## 1. Acceptance of Terms

By packaging, signing, publishing, or installing software using the Orion Package Kit (`opk`), you agree to comply with these Terms of Service. If you do not agree to these terms, you must refrain from distributing packages via the Orion Registry.

---

## 2. Developer Responsibilities & Code of Conduct

1. **Malware & Harmful Code Prohibition:** Packages must not contain trojans, spyware, ransomware, keyloggers, unauthorized cryptocurrency miners, or any obfuscated malicious payloads.
2. **Respect for Sandbox Boundaries:** Exploiting Linux kernel vulnerabilities, bypassing Seccomp/AppArmor policies without user disclosure, or attempting privilege escalation is strictly forbidden.
3. **Truthful Metadata:** `manifest.toml` and `sandbox.policy` must accurately declare the application's required permissions and purpose. Misleading permission requests (e.g., claiming to be a calculator while requesting microphone/camera access) will result in immediate package revocation.
4. **Cryptographic Integrity:** Developers are solely responsible for protecting their private Ed25519 signing keys (`private.key`). Packages signed with compromised keys must be revoked immediately via the registry.

---

## 3. User Privacy and Data Protection

* Applications must not collect personally identifiable information (PII) without transparent, opt-in consent.
* Access to storage directories (`~/.local/share/`) must remain restricted to application-owned namespaces unless explicitly requested through system file pickers.

---

## 4. Takedowns, Revocations, and Governance

The Orion Security Board reserves the right to:
1. Mark vulnerable or compromised packages as "Yanked" or "Revoked".
2. Issue emergency security advisories via the Bekçi (Gatekeeper) daemon.
3. Terminate developer API keys for repeated or severe violations.

---

## 5. Disclaimer of Warranty

ORION PACKAGE KIT AND ASSOCIATED REGISTRY SERVICES ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED. UNDER NO CIRCUMSTANCES SHALL THE AUTHORS OR CONTRIBUTORS BE HELD LIABLE FOR ANY DAMAGES ARISING FROM THE USE OF PACKAGED SOFTWARE.
