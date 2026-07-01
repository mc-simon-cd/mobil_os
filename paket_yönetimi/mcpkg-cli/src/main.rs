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
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opk")]
#[command(about = "Orion Package Kit CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    },
    /// Verify a package's integrity and signature
    Verify {
        /// Path to the .opk file
        path: PathBuf,
        /// Optional path to the public key to verify against (defaults to ~/.config/mcsimon/signing/public.key)
        #[arg(long, short)]
        key: Option<PathBuf>,
    },
    /// Generate a new Ed25519 keypair for signing
    Keygen {
        /// Optional directory where key files should be written (defaults to ~/.config/mcsimon/signing/)
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
        /// Optional path to the private key (defaults to ~/.config/mcsimon/signing/private.key)
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
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { path_or_id, version, registry } => {
            let path = if path_or_id.ends_with(".opk") || std::path::Path::new(&path_or_id).exists() {
                // Install from local file path
                std::path::PathBuf::from(path_or_id)
            } else {
                // Install from registry
                let client = RegistryClient::new(registry);
                
                // Get or create the default cache root (~/.cache/mcsimon/packages)
                let cache_root = mcpkg_pipeline::registry::CacheManager::get_default_cache_root()?;
                let cache_manager = mcpkg_pipeline::registry::CacheManager::new(cache_root);
                
                println!("Resolving package '{}' from registry...", path_or_id);
                cache_manager.resolve_and_download(&client, &path_or_id, version.as_deref())?
            };

            let mut pipeline = PipelineManager::new(path);
            println!("Starting installation pipeline...");
            pipeline.run_install()?;
        }
        Commands::Verify { path, key } => {
            println!("Verifying package: {:?}", path);
            DeveloperTools::verify(&path, key.as_deref())?;
            println!("Package verification successful!");
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
            let apps_root = PathBuf::from("/tmp/orion/apps");
            mcpkg_pipeline::Launcher::launch(&app_id, &apps_root)?;
        }
    }

    Ok(())
}
