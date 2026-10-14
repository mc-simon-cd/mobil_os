use mcpkg_core::{SandboxPolicy, policy::Permission};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    Verified,
    Signed,
    Unsigned,
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustLevel::Verified => write!(f, "Verified (Doğrulanmış)"),
            TrustLevel::Signed => write!(f, "Signed (İmzalı)"),
            TrustLevel::Unsigned => write!(f, "Unsigned (İmzasız)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low (Düşük)"),
            RiskLevel::Medium => write!(f, "Medium (Orta)"),
            RiskLevel::High => write!(f, "High (Yüksek)"),
            RiskLevel::Critical => write!(f, "Critical (Kritik)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlag {
    pub permission: String,
    pub value: String,
    pub weight: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub total: u32,
    pub level: RiskLevel,
    pub flags: Vec<RiskFlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Option<i64>,
    pub timestamp: String,
    pub action: String,
    pub package_id: String,
    pub version: String,
    pub trust_level: TrustLevel,
    pub risk_level: RiskLevel,
    pub risk_score: u32,
    pub user_choice: String,
    pub warnings: Vec<String>,
    pub checksum_ok: bool,
    pub sig_ok: bool,
    pub key_fp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BekciDecision {
    pub package_id: String,
    pub version: String,
    pub trust_level: TrustLevel,
    pub risk_score: RiskScore,
    pub warnings: Vec<String>,
    pub require_user_confirmation: bool,
    pub hard_block: bool,
    pub checksum_ok: bool,
    pub signature_ok: bool,
    pub key_fp: Option<String>,
}

pub struct BekciEngine;

impl BekciEngine {
    /// Evaluates sandbox policy and produces an aggregate risk score (0-100).
    pub fn score_policy(policy: &SandboxPolicy) -> RiskScore {
        let mut flags = Vec::new();
        let mut total: u32 = 0;

        if policy.network.internet == Permission::Allow {
            total += 10;
            flags.push(RiskFlag {
                permission: "network.internet".into(),
                value: "allow".into(),
                weight: 10,
                message: "İnternet erişimi talep ediyor".into(),
            });
        }

        if policy.filesystem.system_read == Permission::Allow {
            total += 25;
            flags.push(RiskFlag {
                permission: "filesystem.system_read".into(),
                value: "allow".into(),
                weight: 25,
                message: "Sistem dosyalarına (/etc, /usr vb.) erişim istiyor".into(),
            });
        }

        if policy.filesystem.arbitrary_path == Permission::Allow {
            total += 30;
            flags.push(RiskFlag {
                permission: "filesystem.arbitrary_path".into(),
                value: "allow".into(),
                weight: 30,
                message: "Rastgele dosya sistemi erişimi — dikkatli olun".into(),
            });
        }

        if policy.system.exec_subprocess == Permission::Allow {
            total += 20;
            flags.push(RiskFlag {
                permission: "system.exec_subprocess".into(),
                value: "allow".into(),
                weight: 20,
                message: "Alt süreçler başlatabilir".into(),
            });
        }

        if policy.system.autostart == Permission::Allow {
            total += 15;
            flags.push(RiskFlag {
                permission: "system.autostart".into(),
                value: "allow".into(),
                weight: 15,
                message: "Sistem başlangıcında otomatik çalışır".into(),
            });
        }

        if policy.hardware.camera == Permission::Allow {
            total += 10;
            flags.push(RiskFlag {
                permission: "hardware.camera".into(),
                value: "allow".into(),
                weight: 10,
                message: "Kamera erişimi talep ediyor".into(),
            });
        }

        if policy.hardware.microphone == Permission::Allow {
            total += 10;
            flags.push(RiskFlag {
                permission: "hardware.microphone".into(),
                value: "allow".into(),
                weight: 10,
                message: "Mikrofon erişimi talep ediyor".into(),
            });
        }

        if policy.hardware.location == Permission::Allow {
            total += 10;
            flags.push(RiskFlag {
                permission: "hardware.location".into(),
                value: "allow".into(),
                weight: 10,
                message: "Coğrafi konum bilgisi talep ediyor".into(),
            });
        }

        if policy.system.ipc == Permission::Allow {
            total += 15;
            flags.push(RiskFlag {
                permission: "system.ipc".into(),
                value: "allow".into(),
                weight: 15,
                message: "Diğer süreçlerle IPC iletişimi kurabilir".into(),
            });
        }

        if policy.network.internet == Permission::Deny && total >= 5 {
            total = total.saturating_sub(5);
        }

        let level = match total {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        RiskScore { total, level, flags }
    }

    /// Evaluates an .opk package: checks checksums, validates signature against known keys,
    /// scores the sandbox policy, and generates a BekciDecision.
    pub fn evaluate_package(
        package_path: &Path,
        trusted_key: Option<&ed25519_dalek::VerifyingKey>,
    ) -> Result<BekciDecision, String> {
        let file = fs::File::open(package_path)
            .map_err(|e| format!("Paket açılamadı: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Geçersiz .opk arşivi: {}", e))?;

        // 1. Read manifest.toml
        let manifest_content = {
            let mut f = archive.by_name("manifest.toml")
                .map_err(|_| "manifest.toml pakette bulunamadı")?;
            let mut s = String::new();
            f.read_to_string(&mut s).map_err(|e| e.to_string())?;
            s
        };
        let manifest = mcpkg_core::Manifest::from_toml(&manifest_content)
            .map_err(|e| format!("Geçersiz manifest.toml: {}", e))?;

        // 2. Read sandbox.policy
        let policy_content = {
            let mut f = archive.by_name("sandbox.policy")
                .map_err(|_| "sandbox.policy pakette bulunamadı")?;
            let mut s = String::new();
            f.read_to_string(&mut s).map_err(|e| e.to_string())?;
            s
        };
        let policy = mcpkg_core::SandboxPolicy::from_toml(&policy_content)
            .map_err(|e| format!("Geçersiz sandbox.policy: {}", e))?;

        // 3. Read checksums.sha256 and verify files
        let (checksum_ok, mut warnings) = {
            let checksum_content = {
                if let Ok(mut f) = archive.by_name("checksums.sha256") {
                    let mut s = String::new();
                    if f.read_to_string(&mut s).is_ok() {
                        Some(s)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(content) = checksum_content {
                let checksums = mcpkg_core::checksum::ChecksumListParser::parse(&content);
                let mut ok = true;
                let mut warns = Vec::new();
                for entry in checksums {
                    if archive.by_name(&entry.path).is_err() {
                        ok = false;
                        warns.push(format!("Sağlama toplamında listelenen dosya eksik: {}", entry.path));
                        break;
                    }
                }
                (ok, warns)
            } else {
                (false, vec!["checksums.sha256 eksik veya okunamadı".to_string()])
            }
        };

        // 4. Verify signature.ed25519
        let mut sig_bytes = [0u8; 64];
        let has_sig = if let Ok(mut sig_f) = archive.by_name("signature.ed25519") {
            sig_f.read_exact(&mut sig_bytes).is_ok()
        } else {
            false
        };

        let (trust_level, signature_ok, key_fp) = if has_sig {
            if let Ok(sig) = mcpkg_crypto::signing::CryptoManager::signature_from_bytes(&sig_bytes) {
                if let Ok(msg) = crate::developer::calculate_aggregate_message(&mut archive, &manifest) {
                    let key = if let Some(k) = trusted_key {
                        Some(k.clone())
                    } else if let Ok(def_dir) = crate::developer::DeveloperTools::get_default_key_dir() {
                        let pub_path = def_dir.join("public.key");
                        if pub_path.exists() {
                            fs::read_to_string(&pub_path)
                                .ok()
                                .and_then(|c| crate::developer::parse_public_key(&c).ok())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(vk) = key {
                        let fp = crate::developer::calculate_fingerprint(vk.as_bytes());
                        if mcpkg_crypto::signing::CryptoManager::verify(&vk, &msg, &sig).is_ok() {
                            (TrustLevel::Verified, true, Some(fp))
                        } else {
                            warnings.push("İmza doğrulaması başarısız oldu (anahtar uyuşmazlığı)".to_string());
                            (TrustLevel::Signed, false, Some(fp))
                        }
                    } else {
                        warnings.push("Paket imzalı fakat genel anahtar bulunamadı".to_string());
                        (TrustLevel::Signed, false, None)
                    }
                } else {
                    warnings.push("Paket imza özeti hesaplanamadı".to_string());
                    (TrustLevel::Unsigned, false, None)
                }
            } else {
                warnings.push("Bozuk Ed25519 imza biçimi".to_string());
                (TrustLevel::Unsigned, false, None)
            }
        } else {
            warnings.push("Paket imzasız".to_string());
            (TrustLevel::Unsigned, false, None)
        };

        // 5. Score policy
        let risk_score = Self::score_policy(&policy);
        for flag in &risk_score.flags {
            warnings.push(format!("[{}] {}", flag.permission, flag.message));
        }

        let hard_block = !checksum_ok;
        let require_user_confirmation = risk_score.total > 25 || trust_level == TrustLevel::Unsigned;

        Ok(BekciDecision {
            package_id: manifest.package.id,
            version: manifest.package.version,
            trust_level,
            risk_score,
            warnings,
            require_user_confirmation,
            hard_block,
            checksum_ok,
            signature_ok,
            key_fp,
        })
    }
}

pub struct AuditLogger;

impl AuditLogger {
    /// Returns default path for the Bekci SQLite audit database.
    pub fn get_default_db_path() -> PathBuf {
        if let Ok(val) = std::env::var("ORION_AUDIT_DB") {
            return PathBuf::from(val);
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".local/share/orion/bekci/audit.db");
        }
        PathBuf::from("/tmp/orion/audit.db")
    }

    /// Initializes SQLite database and creates audit_log table if not exists.
    pub fn init_db(db_path: &Path) -> Result<Connection, String> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create audit db directory {:?}: {}", parent, e))?;
        }

        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open SQLite database at {:?}: {}", db_path, e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp   TEXT NOT NULL,
                action      TEXT NOT NULL,
                package_id  TEXT NOT NULL,
                version     TEXT NOT NULL,
                trust_level TEXT NOT NULL,
                risk_level  TEXT NOT NULL,
                risk_score  INTEGER NOT NULL,
                user_choice TEXT NOT NULL,
                warnings    TEXT,
                checksum_ok INTEGER NOT NULL,
                sig_ok      INTEGER NOT NULL,
                key_fp      TEXT
            );",
            [],
        )
        .map_err(|e| format!("Failed to initialize audit_log table: {}", e))?;

        Ok(conn)
    }

    /// Records a new audit log entry into the SQLite database.
    pub fn log_event(db_path: &Path, entry: &AuditEntry) -> Result<(), String> {
        let conn = Self::init_db(db_path)?;
        let warnings_json = serde_json::to_string(&entry.warnings).unwrap_or_else(|_| "[]".to_string());
        let trust_str = match entry.trust_level {
            TrustLevel::Verified => "verified",
            TrustLevel::Signed => "signed",
            TrustLevel::Unsigned => "unsigned",
        };
        let risk_str = match entry.risk_level {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        };

        conn.execute(
            "INSERT INTO audit_log (
                timestamp, action, package_id, version,
                trust_level, risk_level, risk_score, user_choice,
                warnings, checksum_ok, sig_ok, key_fp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                entry.timestamp,
                entry.action,
                entry.package_id,
                entry.version,
                trust_str,
                risk_str,
                entry.risk_score,
                entry.user_choice,
                warnings_json,
                if entry.checksum_ok { 1 } else { 0 },
                if entry.sig_ok { 1 } else { 0 },
                entry.key_fp.as_deref()
            ],
        )
        .map_err(|e| format!("Failed to insert into audit_log: {}", e))?;

        Ok(())
    }

    /// Lists audit logs with optional filter by package_id.
    pub fn list_logs(db_path: &Path, package_id_filter: Option<&str>) -> Result<Vec<AuditEntry>, String> {
        if !db_path.exists() {
            return Ok(Vec::new());
        }

        let conn = Self::init_db(db_path)?;
        let mut stmt = if let Some(_) = package_id_filter {
            conn.prepare(
                "SELECT id, timestamp, action, package_id, version, trust_level, risk_level, risk_score, user_choice, warnings, checksum_ok, sig_ok, key_fp
                 FROM audit_log WHERE package_id = ?1 ORDER BY id DESC"
            ).map_err(|e| e.to_string())?
        } else {
            conn.prepare(
                "SELECT id, timestamp, action, package_id, version, trust_level, risk_level, risk_score, user_choice, warnings, checksum_ok, sig_ok, key_fp
                 FROM audit_log ORDER BY id DESC"
            ).map_err(|e| e.to_string())?
        };

        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<AuditEntry> {
            let id: i64 = row.get(0)?;
            let timestamp: String = row.get(1)?;
            let action: String = row.get(2)?;
            let package_id: String = row.get(3)?;
            let version: String = row.get(4)?;
            let trust_str: String = row.get(5)?;
            let risk_str: String = row.get(6)?;
            let risk_score: u32 = row.get(7)?;
            let user_choice: String = row.get(8)?;
            let warnings_raw: String = row.get(9)?;
            let checksum_ok: i32 = row.get(10)?;
            let sig_ok: i32 = row.get(11)?;
            let key_fp: Option<String> = row.get(12)?;

            let trust_level = match trust_str.as_str() {
                "verified" => TrustLevel::Verified,
                "signed" => TrustLevel::Signed,
                _ => TrustLevel::Unsigned,
            };

            let risk_level = match risk_str.as_str() {
                "critical" => RiskLevel::Critical,
                "high" => RiskLevel::High,
                "medium" => RiskLevel::Medium,
                _ => RiskLevel::Low,
            };

            let warnings: Vec<String> = serde_json::from_str(&warnings_raw).unwrap_or_default();

            Ok(AuditEntry {
                id: Some(id),
                timestamp,
                action,
                package_id,
                version,
                trust_level,
                risk_level,
                risk_score,
                user_choice,
                warnings,
                checksum_ok: checksum_ok != 0,
                sig_ok: sig_ok != 0,
                key_fp,
            })
        };

        let rows = if let Some(pkg_id) = package_id_filter {
            stmt.query_map([pkg_id], map_row).map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], map_row).map_err(|e| e.to_string())?
        };

        let mut results = Vec::new();
        for row in rows {
            if let Ok(entry) = row {
                results.push(entry);
            }
        }

        Ok(results)
    }

    /// Returns the latest audit entry for a given package_id, if any.
    pub fn get_latest_for_package(db_path: &Path, package_id: &str) -> Result<Option<AuditEntry>, String> {
        let entries = Self::list_logs(db_path, Some(package_id))?;
        Ok(entries.into_iter().next())
    }

    /// Exports all audit entries matching optional package filter in CSV format.
    pub fn export_csv(db_path: &Path, package_id_filter: Option<&str>) -> Result<String, String> {
        let entries = Self::list_logs(db_path, package_id_filter)?;
        let mut csv = String::from("id,timestamp,action,package_id,version,trust_level,risk_level,risk_score,user_choice,warnings,checksum_ok,sig_ok,key_fp\n");
        for e in entries {
            let warn_escaped = e.warnings.join("; ").replace('"', "\"\"");
            let trust_str = match e.trust_level {
                TrustLevel::Verified => "verified",
                TrustLevel::Signed => "signed",
                TrustLevel::Unsigned => "unsigned",
            };
            let risk_str = match e.risk_level {
                RiskLevel::Low => "low",
                RiskLevel::Medium => "medium",
                RiskLevel::High => "high",
                RiskLevel::Critical => "critical",
            };
            let line = format!(
                "{},{},{},{},{},{},{},{},{},\"{}\",{},{},{}\n",
                e.id.unwrap_or(0),
                e.timestamp,
                e.action,
                e.package_id,
                e.version,
                trust_str,
                risk_str,
                e.risk_score,
                e.user_choice,
                warn_escaped,
                if e.checksum_ok { 1 } else { 0 },
                if e.sig_ok { 1 } else { 0 },
                e.key_fp.as_deref().unwrap_or("")
            );
            csv.push_str(&line);
        }
        Ok(csv)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub id: String,
    pub action: String, // "evaluate", "audit_list", "ping"
    #[serde(default)]
    pub package_path: Option<String>,
    #[serde(default)]
    pub package_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub id: String,
    pub status: String, // "ok", "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<BekciDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_logs: Option<Vec<AuditEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub fn get_default_socket_path() -> PathBuf {
    if let Ok(val) = std::env::var("ORION_BEKCI_SOCKET") {
        return PathBuf::from(val);
    }
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir).join("orion/bekci.sock");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/run/orion/bekci.sock");
    }
    PathBuf::from("/tmp/orion/bekci.sock")
}

pub struct BekciDaemon;

impl BekciDaemon {
    /// Runs the Bekçi IPC socket listener loop.
    pub fn run(socket_path: &Path, audit_db_path: &Path) -> Result<(), String> {
        if socket_path.exists() {
            let _ = fs::remove_file(socket_path);
        }
        if let Some(parent) = socket_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create socket directory: {}", e))?;
        }

        let listener = std::os::unix::net::UnixListener::bind(socket_path)
            .map_err(|e| format!("Failed to bind Bekçi Unix socket at {:?}: {}", socket_path, e))?;

        println!("🛡️ Bekçi IPC Daemon dinleniyor: {:?}", socket_path);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut reader = std::io::BufReader::new(&mut stream);
                    let mut line = String::new();
                    use std::io::BufRead;
                    if let Ok(n) = reader.read_line(&mut line) {
                        if n > 0 {
                            let resp = Self::handle_request(line.trim(), audit_db_path);
                            let resp_json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
                            let _ = writeln!(stream, "{}", resp_json);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Socket bağlantı hatası: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handles a single JSON IPC request string.
    pub fn handle_request(req_str: &str, audit_db_path: &Path) -> IpcResponse {
        let req: IpcRequest = match serde_json::from_str(req_str) {
            Ok(r) => r,
            Err(e) => {
                return IpcResponse {
                    id: "error".to_string(),
                    status: "error".to_string(),
                    decision: None,
                    audit_logs: None,
                    message: Some(format!("Geçersiz JSON isteği: {}", e)),
                };
            }
        };

        match req.action.as_str() {
            "ping" => IpcResponse {
                id: req.id,
                status: "ok".to_string(),
                decision: None,
                audit_logs: None,
                message: Some("pong".to_string()),
            },
            "evaluate" => {
                if let Some(pkg_path) = req.package_path {
                    match BekciEngine::evaluate_package(Path::new(&pkg_path), None) {
                        Ok(dec) => IpcResponse {
                            id: req.id,
                            status: "ok".to_string(),
                            decision: Some(dec),
                            audit_logs: None,
                            message: None,
                        },
                        Err(e) => IpcResponse {
                            id: req.id,
                            status: "error".to_string(),
                            decision: None,
                            audit_logs: None,
                            message: Some(e),
                        },
                    }
                } else {
                    IpcResponse {
                        id: req.id,
                        status: "error".to_string(),
                        decision: None,
                        audit_logs: None,
                        message: Some("evaluate isteğinde 'package_path' eksik".to_string()),
                    }
                }
            }
            "audit_list" => {
                match AuditLogger::list_logs(audit_db_path, req.package_id.as_deref()) {
                    Ok(logs) => IpcResponse {
                        id: req.id,
                        status: "ok".to_string(),
                        decision: None,
                        audit_logs: Some(logs),
                        message: None,
                    },
                    Err(e) => IpcResponse {
                        id: req.id,
                        status: "error".to_string(),
                        decision: None,
                        audit_logs: None,
                        message: Some(e),
                    },
                }
            }
            unknown => IpcResponse {
                id: req.id,
                status: "error".to_string(),
                decision: None,
                audit_logs: None,
                message: Some(format!("Bilinmeyen eylem: {}", unknown)),
            },
        }
    }
}

pub struct BekciClient;

impl BekciClient {
    /// Sends a JSON request over the Unix domain socket and returns the response.
    pub fn send_request(socket_path: &Path, req: &IpcRequest) -> Result<IpcResponse, String> {
        let mut stream = std::os::unix::net::UnixStream::connect(socket_path)
            .map_err(|e| format!("Bekçi soketine bağlanılamadı {:?}: {}", socket_path, e))?;

        let json = serde_json::to_string(req)
            .map_err(|e| format!("İstek serileştirilemedi: {}", e))?;
        writeln!(stream, "{}", json)
            .map_err(|e| format!("İstek gönderilemedi: {}", e))?;

        let mut reader = std::io::BufReader::new(stream);
        let mut line = String::new();
        use std::io::BufRead;
        reader.read_line(&mut line)
            .map_err(|e| format!("Yanıt okunamadı: {}", e))?;

        serde_json::from_str(&line)
            .map_err(|e| format!("Yanıt JSON ayrıştırılamadı: {}", e))
    }
}

pub fn get_current_timestamp_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days = (secs / 86400) as i64;
    let day_secs = secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_risk_scoring_engine() {
        let toml_content = r#"[meta]
policy_version = "1.0"
strict_mode = false

[filesystem]
home_read = "allow"
home_write = "allow"
temp_access = "allow"
documents_read = "deny"
documents_write = "deny"
downloads_read = "deny"
system_read = "allow"
arbitrary_path = "allow"

[network]
internet = "allow"
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
"#;
        let policy = SandboxPolicy::from_toml(toml_content).unwrap();
        let score = BekciEngine::score_policy(&policy);

        // arbitrary_path(30) + system_read(25) + internet(10) = 65 -> High
        assert_eq!(score.total, 65);
        assert_eq!(score.level, RiskLevel::High);
        assert_eq!(score.flags.len(), 3);
    }

    #[test]
    fn test_sqlite_audit_logger() {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("audit.db");

        let entry = AuditEntry {
            id: None,
            timestamp: "2026-10-13T22:00:00Z".to_string(),
            action: "install".to_string(),
            package_id: "com.test.auditapp".to_string(),
            version: "1.0.0".to_string(),
            trust_level: TrustLevel::Verified,
            risk_level: RiskLevel::Low,
            risk_score: 10,
            user_choice: "approved".to_string(),
            warnings: vec!["İnternet erişimi talep ediyor".to_string()],
            checksum_ok: true,
            sig_ok: true,
            key_fp: Some("SHA256:abcdef123456".to_string()),
        };

        AuditLogger::log_event(&db_path, &entry).unwrap();

        let logs = AuditLogger::list_logs(&db_path, None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].package_id, "com.test.auditapp");
        assert_eq!(logs[0].trust_level, TrustLevel::Verified);
        assert_eq!(logs[0].risk_score, 10);
        assert_eq!(logs[0].warnings.len(), 1);

        let filtered = AuditLogger::list_logs(&db_path, Some("com.test.auditapp")).unwrap();
        assert_eq!(filtered.len(), 1);

        let none_found = AuditLogger::list_logs(&db_path, Some("com.other.app")).unwrap();
        assert_eq!(none_found.len(), 0);

        // Test get_latest_for_package
        let latest = AuditLogger::get_latest_for_package(&db_path, "com.test.auditapp").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, "1.0.0");

        // Test export_csv
        let csv = AuditLogger::export_csv(&db_path, None).unwrap();
        assert!(csv.contains("com.test.auditapp"));
        assert!(csv.contains("1.0.0"));
        assert!(csv.contains("verified"));
    }

    #[test]
    fn test_ipc_daemon_handle_request() {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("audit.db");

        // Ping
        let req_ping = r#"{"id":"1","action":"ping"}"#;
        let resp_ping = BekciDaemon::handle_request(req_ping, &db_path);
        assert_eq!(resp_ping.status, "ok");
        assert_eq!(resp_ping.message.as_deref(), Some("pong"));

        // Audit list empty
        let req_list = r#"{"id":"2","action":"audit_list"}"#;
        let resp_list = BekciDaemon::handle_request(req_list, &db_path);
        assert_eq!(resp_list.status, "ok");
        assert_eq!(resp_list.audit_logs.unwrap().len(), 0);
    }
}
