//! Language-independent model setup. Providers are adapters; model IDs are open.
use crate::id::identifier;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

pub const MODEL_SETUP_SCHEMA_VERSION: u32 = 1;
pub const PLANNER_ROLE: &str = "planner";

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("invalid model JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid model TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("unsupported schema_version {0}; expected 1")]
    SchemaVersion(u32),
    #[error("invalid {field}: {value:?}")]
    Invalid { field: String, value: String },
    #[error("duplicate {field}: {id}")]
    Duplicate { field: String, id: String },
    #[error("unknown model {provider}/{model}")]
    UnknownModel { provider: String, model: String },
}

fn invalid(field: &str, value: &str) -> ModelError {
    ModelError::Invalid {
        field: field.into(),
        value: value.into(),
    }
}

fn validate_provider_id(value: &str) -> Result<(), ModelError> {
    if !identifier(value, &['-', '_']) {
        return Err(invalid("provider id", value));
    }
    Ok(())
}

fn validate_model_id(value: &str) -> Result<(), ModelError> {
    if !identifier(value, &['-', '_', '.', ':']) {
        return Err(invalid("model id", value));
    }
    Ok(())
}

fn validate_role_id(value: &str) -> Result<(), ModelError> {
    if !identifier(value, &['-', '_']) {
        return Err(invalid("role id", value));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelRef {
    pub provider: String,
    pub model: String,
}

impl ModelRef {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Result<Self, ModelError> {
        let value = Self {
            provider: provider.into(),
            model: model.into(),
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), ModelError> {
        validate_provider_id(&self.provider)?;
        validate_model_id(&self.model)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Grok,
    XaiApi,
    Claude,
    Codex,
    Ollama,
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
}

impl ProviderKind {
    pub fn requires_base_url(self) -> bool {
        matches!(self, Self::XaiApi | Self::Ollama | Self::OpenAiCompatible)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderConfig {
    pub id: String,
    pub kind: ProviderKind,
    pub display_name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub auth_env: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
}

fn default_enabled() -> bool {
    true
}

impl ProviderConfig {
    fn validate(&self) -> Result<(), ModelError> {
        validate_provider_id(&self.id)?;
        if self.display_name.trim().is_empty() {
            return Err(invalid("display_name", &self.display_name));
        }
        if self.kind.requires_base_url() {
            match &self.base_url {
                None => return Err(invalid("base_url", "required for this provider kind")),
                Some(url) if url.trim().is_empty() || url.contains('\0') => {
                    return Err(invalid("base_url", url));
                }
                Some(_) => {}
            }
        } else if self.base_url.as_ref().is_some_and(|u| !u.trim().is_empty()) {
            return Err(invalid(
                "base_url",
                "only xai_api, ollama, and openai_compatible may set base_url",
            ));
        }
        if let Some(env) = &self.auth_env {
            if !env.chars().all(|c| c.is_ascii_uppercase() || c == '_') || env.is_empty() {
                return Err(invalid("auth_env (ENV_NAME)", env));
            }
        }
        let mut seen = HashSet::new();
        for model in &self.models {
            validate_model_id(model)?;
            if !seen.insert(model) {
                return Err(ModelError::Duplicate {
                    field: "provider model".into(),
                    id: format!("{}/{}", self.id, model),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSetup {
    pub schema_version: u32,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub default: Option<ModelRef>,
    #[serde(default)]
    pub roles: BTreeMap<String, Vec<ModelRef>>,
}

impl Default for ModelSetup {
    fn default() -> Self {
        Self {
            schema_version: MODEL_SETUP_SCHEMA_VERSION,
            providers: vec![],
            default: None,
            roles: BTreeMap::from([(PLANNER_ROLE.into(), vec![])]),
        }
    }
}

impl ModelSetup {
    pub fn from_json(input: &str) -> Result<Self, ModelError> {
        let setup: Self = serde_json::from_str(input)?;
        setup.validate()?;
        Ok(setup)
    }

    pub fn from_toml(input: &str) -> Result<Self, ModelError> {
        let setup: Self = toml::from_str(input)?;
        setup.validate()?;
        Ok(setup)
    }

    pub fn to_toml(&self) -> Result<String, ModelError> {
        self.validate()?;
        toml::to_string_pretty(self).map_err(|e| invalid("toml", &e.to_string()))
    }

    pub fn validate(&self) -> Result<(), ModelError> {
        if self.schema_version != MODEL_SETUP_SCHEMA_VERSION {
            return Err(ModelError::SchemaVersion(self.schema_version));
        }
        let mut ids = HashSet::new();
        for provider in &self.providers {
            provider.validate()?;
            if !ids.insert(&provider.id) {
                return Err(ModelError::Duplicate {
                    field: "provider id".into(),
                    id: provider.id.clone(),
                });
            }
        }
        if let Some(default) = &self.default {
            self.require_enabled(default)?;
        }
        for (role, refs) in &self.roles {
            validate_role_id(role)?;
            let mut seen = HashSet::new();
            for model_ref in refs {
                model_ref.validate()?;
                self.require_enabled(model_ref)?;
                if !seen.insert(model_ref) {
                    return Err(ModelError::Duplicate {
                        field: format!("role {role}"),
                        id: format!("{}/{}", model_ref.provider, model_ref.model),
                    });
                }
            }
        }
        Ok(())
    }

    fn require_enabled(&self, model_ref: &ModelRef) -> Result<(), ModelError> {
        model_ref.validate()?;
        let Some(provider) = self.providers.iter().find(|p| p.id == model_ref.provider) else {
            return Err(ModelError::UnknownModel {
                provider: model_ref.provider.clone(),
                model: model_ref.model.clone(),
            });
        };
        if !provider.enabled || !provider.models.iter().any(|m| m == &model_ref.model) {
            return Err(ModelError::UnknownModel {
                provider: model_ref.provider.clone(),
                model: model_ref.model.clone(),
            });
        }
        Ok(())
    }

    pub fn planner(&self) -> &[ModelRef] {
        self.roles
            .get(PLANNER_ROLE)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn set_default(&mut self, model_ref: ModelRef) -> Result<(), ModelError> {
        self.require_enabled(&model_ref)?;
        self.default = Some(model_ref);
        self.validate()
    }

    pub fn toggle_planner(&mut self, model_ref: ModelRef) -> Result<(), ModelError> {
        self.require_enabled(&model_ref)?;
        let panel = self.roles.entry(PLANNER_ROLE.into()).or_default();
        if let Some(index) = panel.iter().position(|item| item == &model_ref) {
            panel.remove(index);
        } else {
            panel.push(model_ref);
        }
        self.validate()
    }

    pub fn add_provider(&mut self, provider: ProviderConfig) -> Result<(), ModelError> {
        provider.validate()?;
        if self.providers.iter().any(|p| p.id == provider.id) {
            return Err(ModelError::Duplicate {
                field: "provider id".into(),
                id: provider.id,
            });
        }
        self.providers.push(provider);
        self.validate()
    }

    /// Overlay wins on default and declared roles. Matching provider IDs are replaced;
    /// remaining overlay providers append in overlay order.
    pub fn merge(&self, overlay: &Self) -> Result<Self, ModelError> {
        self.validate()?;
        overlay.validate()?;
        let mut providers = self.providers.clone();
        for incoming in &overlay.providers {
            if let Some(existing) = providers.iter_mut().find(|p| p.id == incoming.id) {
                *existing = incoming.clone();
            } else {
                providers.push(incoming.clone());
            }
        }
        let mut roles = self.roles.clone();
        for (role, refs) in &overlay.roles {
            roles.insert(role.clone(), refs.clone());
        }
        let merged = Self {
            schema_version: MODEL_SETUP_SCHEMA_VERSION,
            providers,
            default: overlay.default.clone().or_else(|| self.default.clone()),
            roles,
        };
        merged.validate()?;
        Ok(merged)
    }

    /// Fill empty model lists from discovery without replacing declared models or IDs.
    pub fn with_discovery(&self, discovered: Vec<ProviderConfig>) -> Result<Self, ModelError> {
        self.validate()?;
        let mut setup = self.clone();
        for incoming in discovered {
            incoming.validate()?;
            if let Some(existing) = setup.providers.iter_mut().find(|p| p.id == incoming.id) {
                if existing.models.is_empty() {
                    existing.models = incoming.models;
                }
                if existing.display_name.trim().is_empty() {
                    existing.display_name = incoming.display_name;
                }
            } else {
                setup.providers.push(incoming);
            }
        }
        setup.validate()?;
        Ok(setup)
    }
}
