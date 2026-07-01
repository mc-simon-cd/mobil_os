use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,
    pub build: Build,
    pub store: Store,
    pub dependencies: Option<Dependencies>,
    pub update: Option<Update>,
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
