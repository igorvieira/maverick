use maverick_core::*;
use serde_json::json;

fn grok() -> ProviderConfig {
    ProviderConfig {
        id: "grok".into(),
        kind: ProviderKind::Grok,
        display_name: "Grok CLI".into(),
        enabled: true,
        base_url: None,
        auth_env: None,
        models: vec!["grok-4.6".into(), "grok-4.5".into()],
    }
}

fn custom() -> ProviderConfig {
    ProviderConfig {
        id: "local-llm".into(),
        kind: ProviderKind::OpenAiCompatible,
        display_name: "Local OpenAI-compatible".into(),
        enabled: true,
        base_url: Some("http://127.0.0.1:11434/v1".into()),
        auth_env: Some("LOCAL_API_KEY".into()),
        models: vec!["llama3.2".into(), "qwen2.5".into()],
    }
}

fn setup(providers: Vec<ProviderConfig>) -> ModelSetup {
    let setup = ModelSetup {
        schema_version: 1,
        providers,
        default: None,
        roles: Default::default(),
    };
    setup.validate().unwrap();
    setup
}

fn unchecked(providers: Vec<ProviderConfig>) -> ModelSetup {
    ModelSetup {
        schema_version: 1,
        providers,
        default: None,
        roles: Default::default(),
    }
}

#[test]
fn both_provider_kinds_share_default_and_planner_selection() {
    for provider in [grok(), custom()] {
        let mut setup = setup(vec![provider.clone()]);
        let first = ModelRef::new(&provider.id, &provider.models[0]).unwrap();
        let second = ModelRef::new(&provider.id, &provider.models[1]).unwrap();
        setup.set_default(first.clone()).unwrap();
        setup.toggle_planner(second.clone()).unwrap();
        setup.toggle_planner(first.clone()).unwrap();
        assert_eq!(setup.default.as_ref(), Some(&first));
        assert_eq!(setup.planner(), [second.clone(), first.clone()]);
        setup.toggle_planner(second).unwrap();
        assert_eq!(setup.planner(), std::slice::from_ref(&first));
    }
}

#[test]
fn schema_and_ids_and_unknown_fields_are_rejected() {
    let mut value = serde_json::to_value(setup(vec![grok()])).unwrap();
    value["schema_version"] = json!(99);
    assert!(matches!(
        ModelSetup::from_json(&value.to_string()),
        Err(ModelError::SchemaVersion(99))
    ));
    for id in ["", "Grok", "two words", "-bad", "bad__id"] {
        let mut value = serde_json::to_value(setup(vec![grok()])).unwrap();
        value["providers"][0]["id"] = json!(id);
        assert!(
            ModelSetup::from_json(&value.to_string())
                .unwrap_err()
                .to_string()
                .contains("provider id"),
            "{id}"
        );
    }
    for id in ["", "Grok 4", "grok-4.6!", "/model"] {
        assert!(
            ModelRef::new("grok", id).is_err(),
            "{id} should be an invalid model id"
        );
    }
    assert!(ModelRef::new("grok", "grok-4.6").is_ok());
    assert!(ModelRef::new("ollama", "llama3.2:latest").is_ok());
    let mut value = serde_json::to_value(setup(vec![grok()])).unwrap();
    value["commandz"] = json!({});
    assert!(ModelSetup::from_json(&value.to_string()).is_err());
}

#[test]
fn duplicate_providers_and_planner_entries_are_rejected() {
    let mut setup = setup(vec![grok()]);
    assert!(matches!(
        setup.add_provider(grok()),
        Err(ModelError::Duplicate { field, .. }) if field == "provider id"
    ));
    let mut value = serde_json::to_value(setup.clone()).unwrap();
    value["providers"] = json!([grok(), grok()]);
    assert!(matches!(
        ModelSetup::from_json(&value.to_string()),
        Err(ModelError::Duplicate { field, .. }) if field == "provider id"
    ));
    let first = ModelRef::new("grok", "grok-4.6").unwrap();
    setup.toggle_planner(first.clone()).unwrap();
    let mut value = serde_json::to_value(&setup).unwrap();
    value["roles"]["planner"] = json!([first, first]);
    assert!(matches!(
        ModelSetup::from_json(&value.to_string()),
        Err(ModelError::Duplicate { field, .. }) if field == "role planner"
    ));
}

#[test]
fn unknown_or_disabled_models_cannot_be_selected() {
    let mut setup = setup(vec![grok()]);
    assert!(matches!(
        setup.set_default(ModelRef::new("grok", "missing-model").unwrap()),
        Err(ModelError::UnknownModel { .. })
    ));
    setup.providers[0].enabled = false;
    assert!(setup
        .set_default(ModelRef::new("grok", "grok-4.6").unwrap())
        .is_err());
}

#[test]
fn http_kinds_require_base_url_and_cli_kinds_reject_it() {
    let mut provider = custom();
    provider.base_url = None;
    assert!(unchecked(vec![provider.clone()]).validate().is_err());
    provider = grok();
    provider.base_url = Some("https://api.x.ai/v1".into());
    assert!(unchecked(vec![provider]).validate().is_err());
    let mut xai = grok();
    xai.id = "xai-api".into();
    xai.kind = ProviderKind::XaiApi;
    xai.base_url = Some("https://api.x.ai/v1".into());
    xai.auth_env = Some("XAI_API_KEY".into());
    setup(vec![xai]).validate().unwrap();
}

#[test]
fn auth_env_stores_the_variable_name_never_a_secret() {
    let mut provider = custom();
    provider.auth_env = Some("sk-secret".into());
    assert!(unchecked(vec![provider.clone()]).validate().is_err());
    provider.auth_env = Some("LOCAL_API_KEY".into());
    let toml = setup(vec![provider]).to_toml().unwrap();
    assert!(toml.contains("LOCAL_API_KEY"));
    assert!(!toml.contains("sk-"));
}

#[test]
fn toml_round_trip_preserves_planner_order() {
    let mut setup = setup(vec![grok(), custom()]);
    setup
        .set_default(ModelRef::new("grok", "grok-4.6").unwrap())
        .unwrap();
    setup
        .toggle_planner(ModelRef::new("local-llm", "qwen2.5").unwrap())
        .unwrap();
    setup
        .toggle_planner(ModelRef::new("grok", "grok-4.5").unwrap())
        .unwrap();
    let restored = ModelSetup::from_toml(&setup.to_toml().unwrap()).unwrap();
    assert_eq!(restored, setup);
    assert_eq!(
        restored
            .planner()
            .iter()
            .map(|m| format!("{}/{}", m.provider, m.model))
            .collect::<Vec<_>>(),
        ["local-llm/qwen2.5", "grok/grok-4.5"]
    );
}

#[test]
fn project_overlay_replaces_matching_providers_and_roles() {
    let mut base = setup(vec![grok()]);
    base.set_default(ModelRef::new("grok", "grok-4.6").unwrap())
        .unwrap();
    base.toggle_planner(ModelRef::new("grok", "grok-4.6").unwrap())
        .unwrap();
    let mut overlay = setup(vec![custom()]);
    overlay
        .set_default(ModelRef::new("local-llm", "llama3.2").unwrap())
        .unwrap();
    overlay
        .toggle_planner(ModelRef::new("local-llm", "llama3.2").unwrap())
        .unwrap();
    let merged = base.merge(&overlay).unwrap();
    assert_eq!(merged.providers.len(), 2);
    assert_eq!(merged.default.as_ref().unwrap().provider, "local-llm");
    assert_eq!(
        merged.planner(),
        [ModelRef::new("local-llm", "llama3.2").unwrap()]
    );
}

#[test]
fn discovery_fills_empty_model_lists_without_replacing_declared_ones() {
    let mut declared = grok();
    declared.models = vec!["grok-4.6".into()];
    let setup = setup(vec![declared]);
    let mut discovered = grok();
    discovered.models = vec!["grok-4.6".into(), "grok-4.5".into()];
    let filled = setup.with_discovery(vec![discovered, custom()]).unwrap();
    assert_eq!(filled.providers[0].models, ["grok-4.6"]);
    assert_eq!(filled.providers[1].id, "local-llm");
}
