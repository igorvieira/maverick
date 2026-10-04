use maverick_core::{ModelRef, ModelSetup, ProviderConfig, ProviderKind};
use ratatui::widgets::ListState;

const KINDS: [ProviderKind; 6] = [
    ProviderKind::Grok,
    ProviderKind::XaiApi,
    ProviderKind::Claude,
    ProviderKind::Codex,
    ProviderKind::Ollama,
    ProviderKind::OpenAiCompatible,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Models,
    Add,
    Grok,
    Help,
    ConfirmQuit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Up,
    Down,
    Tab,
    BackTab,
    Backspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Save,
    Rediscover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddField {
    Kind,
    Id,
    DisplayName,
    BaseUrl,
    AuthEnv,
    Models,
}

#[derive(Debug, Clone)]
pub struct AddForm {
    pub kind: usize,
    pub field: AddField,
    pub id: String,
    pub display_name: String,
    pub base_url: String,
    pub auth_env: String,
    pub models: String,
    pub error: Option<String>,
}

impl Default for AddForm {
    fn default() -> Self {
        Self {
            kind: 5,
            field: AddField::Kind,
            id: String::new(),
            display_name: String::new(),
            base_url: "http://127.0.0.1:11434/v1".into(),
            auth_env: String::new(),
            models: String::new(),
            error: None,
        }
    }
}

impl AddForm {
    pub fn provider_kind(&self) -> ProviderKind {
        KINDS[self.kind]
    }

    pub fn visible_fields(&self) -> Vec<AddField> {
        let kind = self.provider_kind();
        let mut fields = vec![AddField::Kind, AddField::Id, AddField::DisplayName];
        if kind.requires_base_url() {
            fields.push(AddField::BaseUrl);
        }
        if matches!(kind, ProviderKind::XaiApi | ProviderKind::OpenAiCompatible) {
            fields.push(AddField::AuthEnv);
        }
        fields.push(AddField::Models);
        fields
    }

    fn clamp_field(&mut self) {
        if !self.visible_fields().contains(&self.field) {
            self.field = AddField::Kind;
        }
    }
}

pub struct App {
    pub setup: ModelSetup,
    pub screen: Screen,
    pub selected: usize,
    pub list_state: ListState,
    pub dirty: bool,
    pub status: String,
    pub grok_cli: bool,
    pub xai_key: bool,
    pub add: AddForm,
}

impl App {
    pub fn new(setup: ModelSetup, grok_cli: bool, xai_key: bool) -> Self {
        let mut app = Self {
            setup,
            screen: Screen::Models,
            selected: 0,
            list_state: ListState::default(),
            dirty: false,
            status: String::new(),
            grok_cli,
            xai_key,
            add: AddForm::default(),
        };
        app.sync_list();
        app.status = app.ready_status();
        app
    }

    pub fn entries(&self) -> Vec<ModelRef> {
        self.setup
            .providers
            .iter()
            .filter(|p| p.enabled)
            .flat_map(|p| {
                p.models
                    .iter()
                    .filter_map(|model| ModelRef::new(&p.id, model).ok())
            })
            .collect()
    }

    fn ready_status(&self) -> String {
        let n = self.entries().len();
        if n == 0 {
            "no models yet".into()
        } else if self.dirty {
            format!("unsaved · {n} model(s)")
        } else {
            format!("{n} model(s)")
        }
    }

    fn show_models(&mut self) {
        self.screen = Screen::Models;
        self.status = self.ready_status();
    }

    fn sync_list(&mut self) {
        let len = self.entries().len();
        if len == 0 {
            self.selected = 0;
            self.list_state.select(None);
            return;
        }
        self.selected = self.selected.min(len - 1);
        self.list_state.select(Some(self.selected));
    }

    pub fn handle(&mut self, key: Key) -> Action {
        match self.screen {
            Screen::Models => self.handle_models(key),
            Screen::Add => self.handle_add(key),
            Screen::Grok | Screen::Help => {
                if matches!(key, Key::Esc | Key::Char('q') | Key::Enter | Key::Char('?')) {
                    self.show_models();
                }
                Action::None
            }
            Screen::ConfirmQuit => match key {
                Key::Char('y') | Key::Enter => Action::Quit,
                Key::Char('s') => Action::Save,
                _ => {
                    self.show_models();
                    Action::None
                }
            },
        }
    }

    fn handle_models(&mut self, key: Key) -> Action {
        match key {
            Key::Char('q') | Key::Esc => {
                if self.dirty {
                    self.screen = Screen::ConfirmQuit;
                    self.status = "unsaved changes".into();
                    Action::None
                } else {
                    Action::Quit
                }
            }
            Key::Char('s') => Action::Save,
            Key::Char('r') => Action::Rediscover,
            Key::Char('a') => {
                self.add = AddForm::default();
                self.screen = Screen::Add;
                self.status = "add provider".into();
                Action::None
            }
            Key::Char('g') => {
                self.screen = Screen::Grok;
                self.status = "grok details".into();
                Action::None
            }
            Key::Char('?') | Key::Char('h') => {
                self.screen = Screen::Help;
                Action::None
            }
            Key::Down | Key::Char('j') => {
                let len = self.entries().len();
                if len > 0 {
                    self.selected = (self.selected + 1) % len;
                    self.sync_list();
                }
                Action::None
            }
            Key::Up | Key::Char('k') => {
                let len = self.entries().len();
                if len > 0 {
                    self.selected = (self.selected + len - 1) % len;
                    self.sync_list();
                }
                Action::None
            }
            Key::Enter => {
                if let Some(model) = self.entries().get(self.selected).cloned() {
                    match self.setup.set_default(model.clone()) {
                        Ok(()) => {
                            self.dirty = true;
                            self.status = format!("default {}/{}", model.provider, model.model);
                        }
                        Err(e) => self.status = e.to_string(),
                    }
                }
                Action::None
            }
            Key::Char(' ') => {
                if let Some(model) = self.entries().get(self.selected).cloned() {
                    match self.setup.toggle_planner(model) {
                        Ok(()) => {
                            self.dirty = true;
                            self.status =
                                format!("planner panel: {} model(s)", self.setup.planner().len());
                        }
                        Err(e) => self.status = e.to_string(),
                    }
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_add(&mut self, key: Key) -> Action {
        match key {
            Key::Esc => {
                self.show_models();
                Action::None
            }
            Key::Tab => {
                self.cycle_field(false);
                Action::None
            }
            Key::BackTab => {
                self.cycle_field(true);
                Action::None
            }
            Key::Up | Key::Down if self.add.field == AddField::Kind => {
                let delta = if matches!(key, Key::Down) {
                    1
                } else {
                    KINDS.len() - 1
                };
                self.add.kind = (self.add.kind + delta) % KINDS.len();
                if KINDS[self.add.kind].requires_base_url() && self.add.base_url.is_empty() {
                    self.add.base_url = match KINDS[self.add.kind] {
                        ProviderKind::XaiApi => "https://api.x.ai/v1".into(),
                        _ => "http://127.0.0.1:11434/v1".into(),
                    };
                }
                self.add.clamp_field();
                Action::None
            }
            Key::Enter => self.submit_add(),
            Key::Backspace => {
                if let Some(value) = self.edit_field() {
                    value.pop();
                }
                Action::None
            }
            Key::Char(c) if self.add.field != AddField::Kind && !c.is_control() => {
                if let Some(value) = self.edit_field() {
                    value.push(c);
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn cycle_field(&mut self, reverse: bool) {
        let fields = self.add.visible_fields();
        let Some(i) = fields.iter().position(|f| *f == self.add.field) else {
            self.add.field = fields[0];
            return;
        };
        let next = if reverse {
            (i + fields.len() - 1) % fields.len()
        } else {
            (i + 1) % fields.len()
        };
        self.add.field = fields[next];
    }

    fn edit_field(&mut self) -> Option<&mut String> {
        match self.add.field {
            AddField::Id => Some(&mut self.add.id),
            AddField::DisplayName => Some(&mut self.add.display_name),
            AddField::BaseUrl => Some(&mut self.add.base_url),
            AddField::AuthEnv => Some(&mut self.add.auth_env),
            AddField::Models => Some(&mut self.add.models),
            AddField::Kind => None,
        }
    }

    fn submit_add(&mut self) -> Action {
        let kind = KINDS[self.add.kind];
        let models = self
            .add
            .models
            .split([',', ' '])
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        let provider = ProviderConfig {
            id: self.add.id.trim().into(),
            kind,
            display_name: self.add.display_name.trim().into(),
            enabled: true,
            base_url: kind
                .requires_base_url()
                .then(|| self.add.base_url.trim().to_string()),
            auth_env: {
                let env = self.add.auth_env.trim();
                (!env.is_empty()).then(|| env.to_string())
            },
            models,
        };
        match self.setup.add_provider(provider) {
            Ok(()) => {
                self.dirty = true;
                self.sync_list();
                self.status = "provider added".into();
                self.screen = Screen::Models;
            }
            Err(e) => self.add.error = Some(e.to_string()),
        }
        Action::None
    }

    pub fn apply_discovery(&mut self, discovered: Vec<ProviderConfig>) {
        match self.setup.with_discovery(discovered) {
            Ok(setup) => {
                self.setup = setup;
                self.sync_list();
                self.status = "discovery refreshed (not saved)".into();
            }
            Err(e) => self.status = e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::view::render;
    use maverick_core::{ProviderConfig, ProviderKind};
    use ratatui::{backend::TestBackend, Terminal};

    fn grok_setup() -> ModelSetup {
        let mut setup = ModelSetup::default();
        setup
            .add_provider(ProviderConfig {
                id: "grok".into(),
                kind: ProviderKind::Grok,
                display_name: "Grok CLI".into(),
                enabled: true,
                base_url: None,
                auth_env: None,
                models: vec!["grok-4.6".into(), "grok-4.5".into()],
            })
            .unwrap();
        setup
    }

    fn visible(app: &mut App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        let buf = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..24u16 {
            for x in 0..80u16 {
                out.push_str(buf[(x, y)].symbol());
            }
        }
        out
    }

    #[test]
    fn enter_sets_default_and_space_toggles_planner_in_order() {
        let mut app = App::new(grok_setup(), true, false);
        assert_eq!(app.handle(Key::Enter), Action::None);
        assert_eq!(
            app.setup.default.as_ref().map(|m| m.model.as_str()),
            Some("grok-4.6")
        );
        app.handle(Key::Char(' '));
        app.handle(Key::Down);
        app.handle(Key::Char(' '));
        assert_eq!(
            app.setup
                .planner()
                .iter()
                .map(|m| m.model.as_str())
                .collect::<Vec<_>>(),
            ["grok-4.6", "grok-4.5"]
        );
        app.handle(Key::Char(' '));
        assert_eq!(
            app.setup
                .planner()
                .iter()
                .map(|m| m.model.as_str())
                .collect::<Vec<_>>(),
            ["grok-4.6"]
        );
        let text = visible(&mut app);
        assert!(text.contains("grok-4.6"));
        assert!(text.contains("default"));
        assert!(text.contains("planner"));
        assert!(text.contains("Grok CLI"));
        assert!(text.contains("MAVERICK"));
        assert!(text.contains("Selected"));
    }

    #[test]
    fn add_form_accepts_openai_compatible_without_language_branches() {
        let mut app = App::new(ModelSetup::default(), false, false);
        app.handle(Key::Char('a'));
        assert_eq!(app.screen, Screen::Add);
        let add_text = visible(&mut app);
        assert!(add_text.contains("OpenAI-compatible"));
        assert!(add_text.contains("Do not paste secrets"));
        for c in "local-llm".chars() {
            app.add.field = AddField::Id;
            app.handle(Key::Char(c));
        }
        app.add.field = AddField::DisplayName;
        for c in "Local".chars() {
            app.handle(Key::Char(c));
        }
        app.add.field = AddField::Models;
        for c in "llama3.2".chars() {
            app.handle(Key::Char(c));
        }
        app.handle(Key::Enter);
        assert_eq!(app.screen, Screen::Models);
        assert_eq!(app.setup.providers[0].kind, ProviderKind::OpenAiCompatible);
        assert_eq!(app.setup.providers[0].models, ["llama3.2"]);
        let text = visible(&mut app);
        assert!(text.contains("local-llm/llama3.2"));
    }

    #[test]
    fn empty_state_explains_how_to_start() {
        let mut app = App::new(ModelSetup::default(), false, false);
        let text = visible(&mut app);
        assert!(text.contains("No models yet"));
        assert!(text.contains("OpenAI-compatible"));
        assert!(text.contains("Grok CLI is not on PATH"));
    }

    #[test]
    fn help_and_quit_are_overlays_on_the_model_list() {
        let mut app = App::new(grok_setup(), true, false);
        app.handle(Key::Enter);
        app.handle(Key::Char('?'));
        let help = visible(&mut app);
        assert!(help.contains("grok-4.6"));
        assert!(help.contains("Help"));
        assert!(help.contains("set default model"));
        app.handle(Key::Esc);
        assert_eq!(app.screen, Screen::Models);
        app.handle(Key::Char('q'));
        let quit = visible(&mut app);
        assert_eq!(app.screen, Screen::ConfirmQuit);
        assert!(quit.contains("unsaved changes"));
        assert!(quit.contains("grok-4.6"));
        app.handle(Key::Esc);
        assert_eq!(app.screen, Screen::Models);
    }

    #[test]
    fn tab_skips_fields_that_the_kind_does_not_use() {
        let mut app = App::new(ModelSetup::default(), false, false);
        app.handle(Key::Char('a'));
        assert_eq!(app.add.provider_kind(), ProviderKind::OpenAiCompatible);
        app.handle(Key::Tab);
        assert_eq!(app.add.field, AddField::Id);
        app.add.field = AddField::Kind;
        for _ in 0..5 {
            app.handle(Key::Up);
        }
        assert_eq!(app.add.provider_kind(), ProviderKind::Grok);
        assert!(!app.add.visible_fields().contains(&AddField::BaseUrl));
        assert!(!app.add.visible_fields().contains(&AddField::AuthEnv));
        app.add.field = AddField::Kind;
        app.handle(Key::Tab);
        app.handle(Key::Tab);
        app.handle(Key::Tab);
        assert_eq!(app.add.field, AddField::Models);
        app.handle(Key::BackTab);
        assert_eq!(app.add.field, AddField::DisplayName);
    }
}
