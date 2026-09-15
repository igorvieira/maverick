use super::{manifest::validate_command, validate_id, CommandSet, PackError, PackManifest};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// Optional structured companion to the consuming project's Markdown adapter.
/// Pack order is explicit; commands are resolved separately for each selected pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAdapter {
    pub schema_version: u32,
    pub packs: Vec<String>,
    #[serde(default)]
    pub commands: CommandSet,
    #[serde(default)]
    pub pack_commands: BTreeMap<String, CommandSet>,
    #[serde(default)]
    pub project_commands: BTreeMap<String, String>,
    #[serde(default)]
    pub conventions: Vec<String>,
    #[serde(default)]
    pub layout: BTreeMap<String, String>,
}

impl ProjectAdapter {
    pub fn from_json(input: &str) -> Result<Self, PackError> {
        let adapter: Self = serde_json::from_str(input)?;
        adapter.validate()?;
        Ok(adapter)
    }

    pub fn validate(&self) -> Result<(), PackError> {
        if self.schema_version != 1 {
            return Err(PackError::SchemaVersion(self.schema_version));
        }
        let mut seen = HashSet::new();
        for id in &self.packs {
            validate_id("selected pack", id)?;
            if !seen.insert(id) {
                return Err(PackError::Duplicate {
                    field: "selected pack".into(),
                    id: id.clone(),
                });
            }
        }
        self.commands.validate()?;
        for (id, commands) in &self.pack_commands {
            if !seen.contains(id) {
                return Err(PackError::UnknownPack(id.clone()));
            }
            commands.validate()?;
        }
        for (name, command) in &self.project_commands {
            validate_id("project command", name)?;
            validate_command(name, command)?;
        }
        Ok(())
    }

    pub fn resolve_commands(&self, pack: &PackManifest) -> Result<CommandSet, PackError> {
        self.validate()?;
        pack.validate()?;
        if !self.packs.contains(&pack.id) {
            return Err(PackError::UnknownPack(pack.id.clone()));
        }
        let commands = pack.commands.with_overrides(&self.commands);
        Ok(match self.pack_commands.get(&pack.id) {
            Some(overrides) => commands.with_overrides(overrides),
            None => commands,
        })
    }
}
