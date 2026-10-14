use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,
    pub build: Build,
    pub store: Store,
    pub dependencies: Option<Dependencies>,
    pub update: Option<Update>,
    pub resources: Option<Resources>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resources {
    pub memory_mb: Option<u64>,
    pub cpu_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub author_email: String,
    pub homepage: Option<String>,
    pub source_url: Option<String>,
    pub license: Option<String>,
    pub min_os: String,
    pub max_os: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub entry_x86_64: String,
    pub entry_aarch64: Option<String>,
    pub exec_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub category: String,
    pub rating: String,
    pub price_usd: f64,
    pub tier: String,
    pub tags: Option<Vec<String>>,
    pub subscription: Option<Subscription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub enabled: bool,
    pub price_monthly: f64,
    pub price_yearly: f64,
    pub trial_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependencies {
    pub system: Option<Vec<String>>,
    pub packages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update {
    pub channel: String,
    pub auto_update: bool,
    pub changelog_url: Option<String>,
}

impl Manifest {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_manifest_parse_and_roundtrip() {
        let toml_str = r#"
[package]
id = "com.orion.notepad"
name = "Orion Notepad"
version = "1.2.3"
description = "Fast notes"
author = "Dev Team"
author_email = "dev@orion.app"
min_os = "1.0.0"

[build]
entry_x86_64 = "binaries/x86_64/notepad"
entry_aarch64 = "binaries/aarch64/notepad"
exec_type = "elf"

[store]
category = "productivity"
rating = "ALL"
price_usd = 0.0
tier = "free"

[resources]
memory_mb = 512
cpu_percent = 50.0
"#;

        let manifest = Manifest::from_toml(toml_str).unwrap();
        assert_eq!(manifest.package.id, "com.orion.notepad");
        assert_eq!(manifest.package.version, "1.2.3");
        assert_eq!(manifest.build.entry_x86_64, "binaries/x86_64/notepad");
        assert_eq!(manifest.build.entry_aarch64.as_deref(), Some("binaries/aarch64/notepad"));
        assert_eq!(manifest.resources.as_ref().unwrap().memory_mb, Some(512));
        assert_eq!(manifest.resources.as_ref().unwrap().cpu_percent, Some(50.0));

        let serialized = manifest.to_toml().unwrap();
        let re_parsed = Manifest::from_toml(&serialized).unwrap();
        assert_eq!(re_parsed.package.id, manifest.package.id);
    }

    #[test]
    fn test_missing_required_package_id_fails() {
        let invalid_toml = r#"
[package]
name = "No ID App"
version = "1.0.0"
description = "Invalid"
author = "Dev"
author_email = "d@d.com"
min_os = "1.0.0"

[build]
entry_x86_64 = "bin/app"
exec_type = "elf"

[store]
category = "utility"
rating = "ALL"
price_usd = 0.0
tier = "free"
"#;

        assert!(Manifest::from_toml(invalid_toml).is_err());
    }
}

