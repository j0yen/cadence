//! Manifest of registered tools and their tier defaults.

use serde::{Deserialize, Serialize};

use crate::period::Tier;

/// Top-level manifest persisted at `$CADENCE_HOME/manifest.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// Schema version string for future migrations.
    #[serde(default = "default_schema")]
    pub schema: String,
    /// Cadence version that wrote this manifest.
    #[serde(default = "default_version")]
    pub version: String,
    /// Registered tools.
    #[serde(default)]
    pub tools: Vec<ToolEntry>,
}

fn default_schema() -> String {
    "cadence.manifest.v1".into()
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// One registered tool entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolEntry {
    /// Tool name (matches `Record.produced_by`).
    pub name: String,
    /// Default tier the tool records under.
    pub tier: Tier,
}

impl Manifest {
    /// Insert or update the tier for a tool by name.
    pub fn register(&mut self, name: &str, tier: Tier) {
        if let Some(t) = self.tools.iter_mut().find(|t| t.name == name) {
            t.tier = tier;
        } else {
            self.tools.push(ToolEntry { name: name.to_string(), tier });
        }
    }
}
