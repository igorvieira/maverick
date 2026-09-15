use maverick_core::{ModelRef, ModelSetup, ProviderConfig, ProviderKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame,
};

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
    Backspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Save,
    Rediscover,
}

#[derive(Debug, Clone)]
pub struct AddForm {
    pub kind: usize,
    pub field: usize,
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
            field: 0,
            id: String::new(),
            display_name: String::new(),
            base_url: "http://127.0.0.1:11434/v1".into(),
            auth_env: String::new(),
            models: String::new(),
            error: None,
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
            status: "enter default · space planner · a add · g grok · s save · q quit".into(),
            grok_cli,
            xai_key,
            add: AddForm::default(),
        };
        app.sync_list();
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
                if matches!(key, Key::Esc | Key::Char('q') | Key::Enter) {
                    self.screen = Screen::Models;
                }
                Action::None
            }
            Screen::ConfirmQuit => match key {
                Key::Char('y') | Key::Enter => Action::Quit,
                Key::Char('s') => Action::Save,
                _ => {
                    self.screen = Screen::Models;
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
                    self.status = "unsaved changes: y quit · s save · other cancel".into();
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
                Action::None
            }
            Key::Char('g') => {
                self.screen = Screen::Grok;
                Action::None
            }
            Key::Char('?') => {
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
                    match self.setup.set_default(model) {
                        Ok(()) => {
                            self.dirty = true;
                            self.status = "default updated".into();
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
                self.screen = Screen::Models;
                Action::None
            }
            Key::Tab => {
                self.add.field = (self.add.field + 1) % 6;
                Action::None
            }
            Key::Up | Key::Down if self.add.field == 0 => {
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
                Action::None
            }
            Key::Enter => self.submit_add(),
            Key::Backspace => {
                self.edit_field().pop();
                Action::None
            }
            Key::Char(c) if self.add.field > 0 && !c.is_control() => {
                self.edit_field().push(c);
                Action::None
            }
            _ => Action::None,
        }
    }

    fn edit_field(&mut self) -> &mut String {
        match self.add.field {
            1 => &mut self.add.id,
            2 => &mut self.add.display_name,
            3 => &mut self.add.base_url,
            4 => &mut self.add.auth_env,
            _ => &mut self.add.models,
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
                self.screen = Screen::Models;
                self.status = "provider added".into();
                self.sync_list();
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

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());
    render_header(frame, chunks[0], app);
    match app.screen {
        Screen::Models | Screen::ConfirmQuit => render_models(frame, chunks[1], app),
        Screen::Add => render_add(frame, chunks[1], app),
        Screen::Grok => render_grok(frame, chunks[1], app),
        Screen::Help => render_help(frame, chunks[1]),
    }
    let footer = match app.screen {
        Screen::ConfirmQuit => app.status.as_str(),
        _ => app.status.as_str(),
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::bordered().title("Status")),
        chunks[2],
    );
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let default = app
        .setup
        .default
        .as_ref()
        .map(|m| format!("{}/{}", m.provider, m.model))
        .unwrap_or_else(|| "none".into());
    let grok = if app.grok_cli {
        "CLI present"
    } else {
        "CLI missing"
    };
    let api = if app.xai_key {
        "XAI_API_KEY set"
    } else {
        "XAI_API_KEY unset"
    };
    let title = format!(
        "Maverick models  default {default}  planner {}  grok {grok}  {api}",
        app.setup.planner().len()
    );
    frame.render_widget(
        Paragraph::new(title).block(Block::bordered().title("Setup")),
        area,
    );
}

fn render_models(frame: &mut Frame, area: Rect, app: &mut App) {
    let planner = app.setup.planner().to_vec();
    let default = app.setup.default.clone();
    let items: Vec<ListItem> = app
        .entries()
        .into_iter()
        .map(|model| {
            let mut marks = vec![];
            if default.as_ref() == Some(&model) {
                marks.push("default");
            }
            if planner.contains(&model) {
                marks.push("planner");
            }
            let suffix = if marks.is_empty() {
                String::new()
            } else {
                format!("  [{}]", marks.join(" "))
            };
            ListItem::new(format!("{}/{}{suffix}", model.provider, model.model))
        })
        .collect();
    let list = if items.is_empty() {
        List::new(vec![ListItem::new(
            "No models yet. Press a to add a provider, or install grok/ollama.",
        )])
    } else {
        List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    };
    frame.render_stateful_widget(
        list.block(Block::bordered().title("Models")),
        area,
        &mut app.list_state,
    );
}

fn render_add(frame: &mut Frame, area: Rect, app: &App) {
    let kind = format!("{:?}", KINDS[app.add.kind]);
    let cursor = |i: usize, label: &str, value: &str| {
        let mark = if app.add.field == i { ">" } else { " " };
        Line::from(format!("{mark} {label}: {value}"))
    };
    let mut lines = vec![
        cursor(0, "kind", &kind),
        cursor(1, "id", &app.add.id),
        cursor(2, "display_name", &app.add.display_name),
        cursor(3, "base_url", &app.add.base_url),
        cursor(4, "auth_env", &app.add.auth_env),
        cursor(5, "models", &app.add.models),
        Line::from("tab fields · up/down kind · enter save · esc cancel"),
        Line::from("auth_env is the variable NAME (XAI_API_KEY). Do not paste secrets."),
    ];
    if let Some(error) = &app.add.error {
        lines.push(Line::from(Span::raw(error.clone())));
    }
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title("Add provider")),
        area,
    );
}

fn render_grok(frame: &mut Frame, area: Rect, app: &App) {
    let grok = app
        .setup
        .providers
        .iter()
        .find(|p| p.kind == ProviderKind::Grok);
    let api = app
        .setup
        .providers
        .iter()
        .find(|p| p.kind == ProviderKind::XaiApi);
    let models = grok
        .map(|p| p.models.join(", "))
        .unwrap_or_else(|| "none discovered".into());
    let lines = vec![
        Line::from(format!(
            "Grok CLI: {}",
            if app.grok_cli {
                "available"
            } else {
                "not on PATH"
            }
        )),
        Line::from(format!("CLI models: {models}")),
        Line::from(format!(
            "xAI API provider: {}",
            if api.is_some() {
                "configured"
            } else {
                "absent"
            }
        )),
        Line::from(format!(
            "XAI_API_KEY: {}",
            if app.xai_key {
                "set in environment"
            } else {
                "unset"
            }
        )),
        Line::from("Grok is a provider implementation, not the runtime's internal model."),
        Line::from("To use the API directly: export XAI_API_KEY and press r to rediscover."),
        Line::from("This screen does not launch the grok TUI."),
        Line::from("Planning can include every model marked [planner]; execution comes later."),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title("Grok")),
        area,
    );
}

fn render_help(frame: &mut Frame, area: Rect) {
    let text = "\
enter  set default model
space  toggle planner panel (order preserved)
a      add provider (OpenAI-compatible, Ollama, xAI API, CLIs)
g      Grok details
r      rediscover local CLIs (does not save)
s      save user config
q      quit
Models stay data. The kernel does not call providers in this slice.";
    frame.render_widget(
        Paragraph::new(text).block(Block::bordered().title("Help")),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
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
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
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
    }

    #[test]
    fn add_form_accepts_openai_compatible_without_language_branches() {
        let mut app = App::new(ModelSetup::default(), false, false);
        app.handle(Key::Char('a'));
        assert_eq!(app.screen, Screen::Add);
        for c in "local-llm".chars() {
            app.add.field = 1;
            app.handle(Key::Char(c));
        }
        app.add.field = 2;
        for c in "Local".chars() {
            app.handle(Key::Char(c));
        }
        app.add.field = 5;
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
}
