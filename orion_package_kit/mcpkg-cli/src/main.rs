// -----------------------------------------------------------------------------
// Copyright 2026 mcsimon project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------

use clap::{Parser, Subcommand};
use mcpkg_pipeline::PipelineManager;
use mcpkg_pipeline::developer::DeveloperTools;
use mcpkg_pipeline::registry::RegistryClient;
use mcpkg_pipeline::LifecycleManager;
use mcpkg_pipeline::bekci::{AuditLogger, BekciEngine, BekciDaemon, get_default_socket_path};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opk")]
#[command(about = "Orion Package Kit CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum AuditCommands {
    /// List recorded audit entries (default)
    List {
        /// Optional package ID filter
        #[arg(long, short)]
        package: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Export audit logs as CSV or JSON
    Export {
        /// Optional package ID filter
        #[arg(long, short)]
        package: Option<String>,
        /// Format (csv or json, default: csv)
        #[arg(long, default_value = "csv")]
        format: String,
    },
    /// Show detailed audit log for a package
    Show {
        /// Package ID (e.g. com.example.app)
        package_id: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Install a .opk package (either by local file path or package ID)
    Install {
        /// Path to the .opk file or a package ID from registry
        path_or_id: String,
        /// Optional version if installing by ID
        #[arg(long, short)]
        version: Option<String>,
        /// Optional custom registry base URL
        #[arg(long, short)]
        registry: Option<String>,
        /// Automatically approve permission prompts without interaction
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Verify a package's integrity and signature
    Verify {
        /// Path to the .opk file
        path: PathBuf,
        /// Optional path to the public key to verify against (defaults to ~/.config/orion/signing/public.key)
        #[arg(long, short)]
        key: Option<PathBuf>,
    },
    /// Generate a new Ed25519 keypair for signing
    Keygen {
        /// Optional directory where key files should be written (defaults to ~/.config/orion/signing/)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Compress a directory into a .opk archive and generate checksums.sha256
    Pack {
        /// Path to the source directory to package
        source_dir: PathBuf,
        /// Optional path for the output .opk file (defaults to {id}-{version}.opk)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Sign a package using a private key
    Sign {
        /// Path to the .opk package to sign
        package: PathBuf,
        /// Optional path to the private key (defaults to ~/.config/orion/signing/private.key)
        #[arg(long, short)]
        key: Option<PathBuf>,
    },
    /// Publish a signed package to the registry
    Publish {
        /// Path to the .opk package to publish
        package: PathBuf,
        /// Optional channel (stable, beta, nightly)
        #[arg(long, short)]
        channel: Option<String>,
        /// Optional custom registry base URL
        #[arg(long, short)]
        registry: Option<String>,
    },
    /// Launch an installed app in a sandbox
    Launch {
        /// ID of the app to launch (e.g., com.example.test)
        app_id: String,
    },
    /// Uninstall an installed application
    Uninstall {
        /// ID of the app to uninstall (e.g., com.example.test)
        app_id: String,
        /// Also remove user configuration and data in ~/.local/share and ~/.config
        #[arg(long)]
        purge: bool,
    },
    /// List installed applications
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Display metadata, permissions and details of a package or installed application
    Info {
        /// Package ID or path to .opk package file
        path_or_id: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Update installed packages to their latest version from registry
    Update {
        /// Optional specific package ID to update
        app_id: Option<String>,
        /// Optional custom registry base URL
        #[arg(long, short)]
        registry: Option<String>,
    },
    /// Manage and inspect Orion Bekçi security audit logs
    Audit {
        #[command(subcommand)]
        cmd: Option<AuditCommands>,
        /// Optional package ID filter
        #[arg(long, short)]
        package: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Start Bekçi background IPC daemon (bekci-d)
    Daemon {
        /// Optional path for Unix domain socket
        #[arg(long, short)]
        socket: Option<PathBuf>,
    },
    /// Search for applications in the Orion package registry
    Search {
        /// Search keyword or query
        query: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Optional custom registry base URL
        #[arg(long, short)]
        registry: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { path_or_id, version, registry, yes } => {
            let path = if path_or_id.ends_with(".opk") || std::path::Path::new(&path_or_id).exists() {
                // Install from local file path
                std::path::PathBuf::from(path_or_id)
            } else {
                // Install from registry
                let client = RegistryClient::new(registry);
                
                let cache_root = mcpkg_pipeline::registry::CacheManager::get_default_cache_root()?;
                let cache_manager = mcpkg_pipeline::registry::CacheManager::new(cache_root);
                
                println!("Resolving package '{}' from registry...", path_or_id);
                cache_manager.resolve_and_download(&client, &path_or_id, version.as_deref())?
            };

            let mut pipeline = PipelineManager::new(path);
            pipeline.assume_yes = yes;
            println!("Starting installation pipeline...");
            pipeline.run_install()?;
        }
        Commands::Verify { path, key } => {
            println!("Verifying package: {:?}", path);
            DeveloperTools::verify(&path, key.as_deref())?;
            println!("Package verification successful!");

            let vk = if let Some(ref k) = key {
                let content = std::fs::read_to_string(k)
                    .map_err(|e| format!("Failed to read public key: {}", e))?;
                Some(mcpkg_pipeline::developer::parse_public_key(&content)?)
            } else {
                None
            };
            if let Ok(decision) = BekciEngine::evaluate_package(&path, vk.as_ref()) {
                println!("\n🛡️ Bekçi Güvenlik Değerlendirmesi:");
                println!("  Güven Seviyesi: {}", decision.trust_level);
                println!("  Risk Skoru:     {}/100 ({})", decision.risk_score.total, decision.risk_score.level);
                println!("  Sağlama:        {}", if decision.checksum_ok { "✅ Doğrulandı" } else { "❌ Hatalı" });
                println!("  İmza Durumu:    {}", if decision.signature_ok { "✅ Geçerli" } else { "⚠️ Doğrulanamadı" });
                if !decision.warnings.is_empty() {
                    println!("  Uyarılar:");
                    for w in &decision.warnings {
                        println!("    • {}", w);
                    }
                }
            }
        }
        Commands::Keygen { output } => {
            let out_dir = match output {
                Some(dir) => dir,
                None => DeveloperTools::get_default_key_dir()?,
            };
            println!("Generating new Ed25519 keypair inside: {:?}", out_dir);
            let (priv_path, pub_path) = DeveloperTools::keygen(&out_dir)?;
            println!("✅ Keys generated successfully:");
            println!("   Private Key: {:?}", priv_path);
            println!("   Public Key:  {:?}", pub_path);
        }
        Commands::Pack { source_dir, output } => {
            let output_path = match output {
                Some(out) => out,
                None => {
                    let manifest_path = source_dir.join("manifest.toml");
                    if manifest_path.exists() {
                        let manifest_content = std::fs::read_to_string(&manifest_path)
                            .map_err(|e| format!("Failed to read manifest.toml: {}", e))?;
                        let manifest = mcpkg_core::Manifest::from_toml(&manifest_content)
                            .map_err(|e| format!("Failed to parse manifest.toml: {}", e))?;
                        PathBuf::from(format!("{}-{}.opk", manifest.package.id, manifest.package.version))
                    } else {
                        PathBuf::from("package.opk")
                    }
                }
            };
            println!("Packing directory {:?} into {:?}", source_dir, output_path);
            DeveloperTools::pack(&source_dir, &output_path)?;
            println!("✅ Package created successfully at {:?}", output_path);
        }
        Commands::Sign { package, key } => {
            let key_path = match key {
                Some(k) => k,
                None => DeveloperTools::get_default_key_dir()?.join("private.key"),
            };
            println!("Signing package {:?} using private key: {:?}", package, key_path);
            DeveloperTools::sign(&package, &key_path)?;
        }
        Commands::Publish { package, channel, registry } => {
            let ch = channel.unwrap_or_else(|| "stable".to_string());
            let client = RegistryClient::new(registry);

            let api_key = match std::env::var("ORION_API_KEY").or_else(|_| std::env::var("MCSIMON_API_KEY")) {
                Ok(key) => key,
                Err(_) => {
                    println!("⚠️ WARNING: ORION_API_KEY environment variable not set.");
                    println!("   Using mock api key fallback for publishing.");
                    "mcs_live_mock_key".to_string()
                }
            };

            println!("Publishing package {:?} to {} on channel '{}'...", package, client.base_url, ch);
            let response = client.publish_package(&package, &ch, &api_key)?;
            println!("✅ Package published successfully!");
            println!("Registry Response:\n{}", serde_json::to_string_pretty(&response).unwrap_or_default());
        }
        Commands::Launch { app_id } => {
            let apps_root = LifecycleManager::get_default_apps_root();
            mcpkg_pipeline::Launcher::launch(&app_id, &apps_root)?;
        }
        Commands::Uninstall { app_id, purge } => {
            let apps_root = LifecycleManager::get_default_apps_root();
            LifecycleManager::uninstall(&app_id, &apps_root, None, None, purge)?;
        }
        Commands::List { json } => {
            let apps_root = LifecycleManager::get_default_apps_root();
            let apps = LifecycleManager::list_installed(&apps_root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&apps).unwrap_or_default());
            } else if apps.is_empty() {
                println!("No applications installed in {:?}", apps_root);
            } else {
                println!("{:<28} {:<24} {:<12} {:<14} {}", "PACKAGE ID", "NAME", "VERSION", "CATEGORY", "SIZE");
                println!("{}", "-".repeat(90));
                for app in apps {
                    let size_str = format_size(app.size_bytes);
                    println!("{:<28} {:<24} {:<12} {:<14} {}", app.id, app.name, app.version, app.category, size_str);
                }
            }
        }
        Commands::Info { path_or_id, json } => {
            let apps_root = LifecycleManager::get_default_apps_root();
            let info = LifecycleManager::get_info(&path_or_id, &apps_root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&info).unwrap_or_default());
            } else {
                println!("📦 Package: {} ({})", info.name, info.id);
                println!("   Version:      {}", info.version);
                println!("   Description:  {}", info.description);
                println!("   Author:       {} <{}>", info.author, info.author_email);
                println!("   Category:     {}", info.category);
                println!("   Tier / Price: {} (${:.2})", info.tier, info.price_usd);
                println!("   Exec Type:    {} ({})", info.exec_type, info.entry_point);
                println!("   Installed:    {}", if info.is_installed { "Yes" } else { "No" });
                if let Some(ref path) = info.install_path {
                    println!("   Install Path: {:?}", path);
                }
                println!("\n🛡️ Declared Permissions:");
                if info.permissions.is_empty() {
                    println!("   (None / Minimal Process Sandbox)");
                } else {
                    for perm in &info.permissions {
                        println!("   • {}", perm);
                    }
                }
            }
        }
        Commands::Update { app_id, registry } => {
            let apps_root = LifecycleManager::get_default_apps_root();
            let client = RegistryClient::new(registry);
            let cache_root = mcpkg_pipeline::registry::CacheManager::get_default_cache_root()?;
            let cache_manager = mcpkg_pipeline::registry::CacheManager::new(cache_root);

            let apps_to_update = match app_id {
                Some(id) => vec![id],
                None => {
                    let installed = LifecycleManager::list_installed(&apps_root)?;
                    installed.into_iter().map(|a| a.id).collect()
                }
            };

            if apps_to_update.is_empty() {
                println!("No installed applications found to update.");
                return Ok(());
            }

            for id in apps_to_update {
                println!("Checking for updates: {}...", id);
                match client.get_package(&id) {
                    Ok(pkg_info) => {
                        let latest_ver = pkg_info.get("latest")
                            .and_then(|l| l.get("version"))
                            .and_then(|v| v.as_str());

                        let current_info = LifecycleManager::get_info(&id, &apps_root).ok();
                        let current_ver = current_info.as_ref().map(|i| i.version.as_str());

                        if let (Some(latest), Some(current)) = (latest_ver, current_ver) {
                            if latest == current {
                                println!("  {} is already at the latest version (v{}).", id, current);
                                continue;
                            }
                            println!("  Updating {}: v{} -> v{}", id, current, latest);
                        }

                        println!("  Downloading update package for {}...", id);
                        match cache_manager.resolve_and_download(&client, &id, latest_ver) {
                            Ok(downloaded_opk) => {
                                let mut pipeline = PipelineManager::new(downloaded_opk);
                                pipeline.apps_root = apps_root.clone();
                                if let Err(e) = pipeline.run_install() {
                                    println!("  ❌ Update failed for {}: {}", id, e);
                                } else {
                                    println!("  ✅ Successfully updated {}!", id);
                                }
                            }
                            Err(e) => println!("  ❌ Failed to download update: {}", e),
                        }
                    }
                    Err(e) => println!("  ⚠️ Could not query registry for {}: {}", id, e),
                }
            }
        }
        Commands::Audit { cmd, package, json } => {
            let db_path = AuditLogger::get_default_db_path();
            match cmd {
                Some(AuditCommands::Export { package: pkg, format }) => {
                    let target_pkg = pkg.or(package);
                    if format.to_lowercase() == "json" {
                        let logs = AuditLogger::list_logs(&db_path, target_pkg.as_deref())?;
                        println!("{}", serde_json::to_string_pretty(&logs).unwrap_or_else(|_| "[]".to_string()));
                    } else {
                        let csv = AuditLogger::export_csv(&db_path, target_pkg.as_deref())?;
                        print!("{}", csv);
                    }
                }
                Some(AuditCommands::Show { package_id, json: is_json }) => {
                    match AuditLogger::get_latest_for_package(&db_path, &package_id)? {
                        Some(entry) => {
                            if is_json || json {
                                println!("{}", serde_json::to_string_pretty(&entry).unwrap_or_else(|_| "{}".to_string()));
                            } else {
                                println!("🛡️ Bekçi Denetim Detayı: {}", entry.package_id);
                                println!("───────────────────────────────────────────────────");
                                println!("  Zaman Damgası: {}", entry.timestamp);
                                println!("  Eylem:         {}", entry.action);
                                println!("  Sürüm:         v{}", entry.version);
                                println!("  Güven Seviyesi:{}", entry.trust_level);
                                println!("  Risk Skoru:    {}/100 ({})", entry.risk_score, entry.risk_level);
                                println!("  Kullanıcı:     {}", entry.user_choice);
                                println!("  Sağlama:       {}", if entry.checksum_ok { "✅ Geçerli" } else { "❌ Hatalı" });
                                println!("  İmza:          {}", if entry.sig_ok { "✅ Doğrulandı" } else { "⚠️ Doğrulanamadı" });
                                if let Some(fp) = &entry.key_fp {
                                    println!("  Anahtar Parmak İzi: {}", fp);
                                }
                                if !entry.warnings.is_empty() {
                                    println!("  Uyarılar:");
                                    for w in &entry.warnings {
                                        println!("    • {}", w);
                                    }
                                }
                            }
                        }
                        None => {
                            println!("'{}' paketi için denetim kaydı bulunamadı.", package_id);
                        }
                    }
                }
                Some(AuditCommands::List { package: pkg, json: is_json }) => {
                    let target_pkg = pkg.or(package);
                    let logs = AuditLogger::list_logs(&db_path, target_pkg.as_deref())?;
                    if is_json || json {
                        println!("{}", serde_json::to_string_pretty(&logs).unwrap_or_else(|_| "[]".to_string()));
                    } else if logs.is_empty() {
                        println!("Kayıtlı denetim günlüğü bulunamadı.");
                    } else {
                        println!("{:<6} {:<20} {:<10} {:<24} {:<10} {:<8} {:<10}", "ID", "TARİH", "EYLEM", "PAKET", "SÜRÜM", "RİSK", "KARAR");
                        println!("{:-<92}", "");
                        for l in logs {
                            println!(
                                "{:<6} {:<20} {:<10} {:<24} {:<10} {:<8} {:<10}",
                                l.id.unwrap_or(0),
                                l.timestamp.chars().take(19).collect::<String>(),
                                l.action,
                                l.package_id,
                                format!("v{}", l.version),
                                format!("{}/100", l.risk_score),
                                l.user_choice
                            );
                        }
                    }
                }
                None => {
                    let logs = AuditLogger::list_logs(&db_path, package.as_deref())?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&logs).unwrap_or_else(|_| "[]".to_string()));
                    } else if logs.is_empty() {
                        println!("Kayıtlı denetim günlüğü bulunamadı.");
                    } else {
                        println!("{:<6} {:<20} {:<10} {:<24} {:<10} {:<8} {:<10}", "ID", "TARİH", "EYLEM", "PAKET", "SÜRÜM", "RİSK", "KARAR");
                        println!("{:-<92}", "");
                        for l in logs {
                            println!(
                                "{:<6} {:<20} {:<10} {:<24} {:<10} {:<8} {:<10}",
                                l.id.unwrap_or(0),
                                l.timestamp.chars().take(19).collect::<String>(),
                                l.action,
                                l.package_id,
                                format!("v{}", l.version),
                                format!("{}/100", l.risk_score),
                                l.user_choice
                            );
                        }
                    }
                }
            }
        }
        Commands::Daemon { socket } => {
            let socket_path = match socket {
                Some(s) => s,
                None => get_default_socket_path(),
            };
            let db_path = AuditLogger::get_default_db_path();
            println!("Starting Bekçi daemon on {:?}", socket_path);
            BekciDaemon::run(&socket_path, &db_path)?;
        }
        Commands::Search { query, json, registry } => {
            let client = RegistryClient::new(registry);
            println!("Searching registry for '{}'...", query);
            let response = client.search_packages(&query)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()));
            } else if let Some(results) = response.get("results").and_then(|r| r.as_array()) {
                if results.is_empty() {
                    println!("No matching packages found.");
                } else {
                    println!("{:<24} {:<10} {:<8} {:<12} {:<30}", "PAKET ID", "SÜRÜM", "FİYAT", "GÜVEN", "AÇIKLAMA");
                    println!("{:-<86}", "");
                    for item in results {
                        let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let ver = item.get("version").and_then(|v| v.as_str()).unwrap_or("");
                        let tier = item.get("tier").and_then(|v| v.as_str()).unwrap_or("free");
                        let trust = item.get("trust_level").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let desc = item.get("description").and_then(|v| v.as_str()).unwrap_or("");
                        println!(
                            "{:<24} {:<10} {:<8} {:<12} {:<30}",
                            id, format!("v{}", ver), tier, trust, desc
                        );
                    }
                }
            } else {
                println!("{}", serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()));
            }
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
