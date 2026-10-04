use super::{
    catalog::{Entry, Scope},
    detect::{Agent, Detection, Status},
    install::command_for,
};
use ratatui::widgets::TableState;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    List,
    Review,
    Results,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Install,
    Redetect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    NoCli,
    Active(String),
    Disabled(String),
    Blocked(String),
    Stale(String),
    Open,
    Marked,
}

impl Cell {
    fn markable(&self) -> bool {
        matches!(self, Cell::Open | Cell::Stale(_) | Cell::Marked)
    }
}

pub struct Planned {
    pub label: String,
    pub argv: Vec<String>,
}

pub struct App {
    pub catalog: Vec<Entry>,
    pub detection: Detection,
    pub scopes: Vec<Scope>,
    pub marked: BTreeSet<(usize, Agent)>,
    pub row: usize,
    pub col: usize,
    pub screen: Screen,
    pub status: String,
    pub results: Vec<(String, Result<(), String>)>,
    pub table_state: TableState,
}

impl App {
    pub fn new(catalog: Vec<Entry>, detection: Detection) -> Self {
        let scopes = catalog.iter().map(|e| e.scope).collect();
        let mut app = Self {
            catalog,
            detection,
            scopes,
            marked: BTreeSet::new(),
            row: 0,
            col: 0,
            screen: Screen::List,
            status: String::new(),
            results: vec![],
            table_state: TableState::default(),
        };
        app.sync();
        app.status = app.ready_status();
        app
    }

    pub fn agent(&self) -> Agent {
        Agent::ALL[self.col]
    }

    pub fn entry(&self) -> &Entry {
        &self.catalog[self.row]
    }

    pub fn cell(&self, row: usize, agent: Agent) -> Cell {
        let entry = &self.catalog[row];
        if !self.detection.agent(agent).on_path {
            return Cell::NoCli;
        }
        let status = self.detection.status(agent, &entry.name);
        match status {
            Some(Status::Active(place)) => return Cell::Active(place.clone()),
            Some(Status::Disabled(place)) => return Cell::Disabled(place.clone()),
            _ => {}
        }
        let missing = self.detection.missing(&entry.name);
        if !missing.is_empty() {
            return Cell::Blocked(format!("needs {}", missing.join(", ")));
        }
        if let Err(why) = command_for(agent, entry, self.scopes[row]) {
            return Cell::Blocked(why);
        }
        if self.marked.contains(&(row, agent)) {
            return Cell::Marked;
        }
        match status {
            Some(Status::Stale(why)) => Cell::Stale(why.clone()),
            _ => Cell::Open,
        }
    }

    pub fn plan(&self) -> Vec<Planned> {
        self.marked
            .iter()
            .filter_map(|&(row, agent)| {
                let entry = &self.catalog[row];
                let argv = command_for(agent, entry, self.scopes[row]).ok()?;
                Some(Planned {
                    label: format!("{} → {}", entry.name, agent.label()),
                    argv,
                })
            })
            .collect()
    }

    pub fn set_detection(&mut self, detection: Detection) {
        self.detection = detection;
        // Drop marks that stopped being installable (installed elsewhere meanwhile, CLI gone…).
        let still: BTreeSet<_> = self
            .marked
            .iter()
            .copied()
            .filter(|&(row, agent)| self.marked_cell_ok(row, agent))
            .collect();
        self.marked = still;
        self.status = self.ready_status();
    }

    pub fn finish_install(
        &mut self,
        results: Vec<(String, Result<(), String>)>,
        detection: Detection,
    ) {
        self.marked.clear();
        self.results = results;
        self.set_detection(detection);
        self.screen = Screen::Results;
    }

    fn marked_cell_ok(&self, row: usize, agent: Agent) -> bool {
        self.cell(row, agent) == Cell::Marked
    }

    fn ready_status(&self) -> String {
        let active = (0..self.catalog.len())
            .flat_map(|row| Agent::ALL.map(|agent| self.cell(row, agent)))
            .filter(|cell| matches!(cell, Cell::Active(_)))
            .count();
        match self.marked.len() {
            0 => format!("{active} active · space to mark what you want"),
            n => format!("{active} active · {n} marked · enter to review"),
        }
    }

    fn sync(&mut self) {
        self.table_state.select(Some(self.row));
        self.table_state.select_column(Some(self.col + 2));
    }

    pub fn handle(&mut self, key: Key) -> Action {
        match self.screen {
            Screen::List => self.handle_list(key),
            Screen::Review => match key {
                Key::Char('y') | Key::Enter => Action::Install,
                _ => {
                    self.screen = Screen::List;
                    Action::None
                }
            },
            Screen::Results | Screen::Help => {
                self.screen = Screen::List;
                Action::None
            }
        }
    }

    fn handle_list(&mut self, key: Key) -> Action {
        let rows = self.catalog.len();
        match key {
            Key::Char('q') | Key::Esc => return Action::Quit,
            Key::Up | Key::Char('k') => self.row = (self.row + rows - 1) % rows,
            Key::Down | Key::Char('j') => self.row = (self.row + 1) % rows,
            Key::Left | Key::Char('h') => self.col = (self.col + 2) % 3,
            Key::Right | Key::Char('l') | Key::Char('\t') => self.col = (self.col + 1) % 3,
            Key::Char(' ') => {
                if let Some(refusal) = self.toggle(self.row, self.agent()) {
                    self.status = refusal;
                    return Action::None;
                }
            }
            Key::Char('a') => self.toggle_row(),
            Key::Char('s') => {
                self.scopes[self.row] = self.scopes[self.row].toggle();
                let row = self.row;
                self.marked.retain(|&(r, agent)| {
                    r != row || command_for(agent, &self.catalog[r], self.scopes[r]).is_ok()
                });
            }
            Key::Char('r') => return Action::Redetect,
            Key::Char('?') => {
                self.screen = Screen::Help;
                return Action::None;
            }
            Key::Enter => {
                if self.marked.is_empty() {
                    self.status = "nothing marked yet · space marks a cell, a marks the row".into();
                } else {
                    self.screen = Screen::Review;
                }
                self.sync();
                return Action::None;
            }
            _ => {}
        }
        self.sync();
        self.status = self.ready_status();
        Action::None
    }

    /// Marks or unmarks one cell; returns why when the cell cannot be marked.
    fn toggle(&mut self, row: usize, agent: Agent) -> Option<String> {
        let cell = self.cell(row, agent);
        if !cell.markable() {
            return Some(match cell {
                Cell::Active(place) => format!("already active in {} ({place})", agent.label()),
                Cell::Disabled(_) => format!("configured but disabled in {}", agent.label()),
                Cell::Blocked(why) => why,
                Cell::NoCli => format!("{} is not on PATH", agent.bin()),
                _ => String::new(),
            });
        }
        if !self.marked.remove(&(row, agent)) {
            self.marked.insert((row, agent));
        }
        None
    }

    fn toggle_row(&mut self) {
        let row = self.row;
        let open: Vec<Agent> = Agent::ALL
            .into_iter()
            .filter(|&agent| self.cell(row, agent).markable())
            .collect();
        let all_marked = open.iter().all(|&a| self.marked.contains(&(row, a)));
        for agent in open {
            if all_marked {
                self.marked.remove(&(row, agent));
            } else {
                self.marked.insert((row, agent));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::{
        catalog::catalog,
        detect::{detect, Paths},
        view::render,
    };
    use crate::setup::discover::MapProbe;
    use ratatui::{backend::TestBackend, Terminal};
    use std::collections::HashMap;

    /// Claude has serena active; Codex and Grok are on PATH with nothing configured.
    fn app() -> App {
        let home = tempfile::tempdir().unwrap();
        std::fs::write(
            home.path().join(".claude.json"),
            r#"{"mcpServers": {"serena": {}}}"#,
        )
        .unwrap();
        let list = |bin: &str| {
            (
                (
                    bin.to_string(),
                    vec!["mcp".into(), "list".into(), "--json".into()],
                ),
                "[]".to_string(),
            )
        };
        let probe = MapProbe {
            binaries: ["claude", "codex", "grok", "npx", "uvx"]
                .map(String::from)
                .into_iter()
                .collect(),
            outputs: HashMap::from([list("codex"), list("grok")]),
            env: HashMap::new(),
        };
        let paths = Paths {
            home: home.path().into(),
            cwd: home.path().into(),
        };
        let catalog = catalog();
        let detection = detect(&probe, &paths, &catalog);
        App::new(catalog, detection)
    }

    fn row_of(app: &App, name: &str) -> usize {
        app.catalog.iter().position(|e| e.name == name).unwrap()
    }

    fn go_to(app: &mut App, name: &str) {
        while app.entry().name != name {
            app.handle(Key::Down);
        }
    }

    #[test]
    fn active_and_blocked_cells_cannot_be_marked() {
        let mut app = app();
        go_to(&mut app, "serena");
        app.handle(Key::Char(' '));
        assert!(app.marked.is_empty());
        assert!(app.status.contains("already active"), "{}", app.status);

        go_to(&mut app, "github");
        app.handle(Key::Char(' '));
        assert!(app.marked.is_empty());
        assert_eq!(app.status, "needs GITHUB_PAT");
    }

    #[test]
    fn a_marks_only_open_cells_and_toggles_back() {
        let mut app = app();
        go_to(&mut app, "serena");
        app.handle(Key::Char('a'));
        let serena = row_of(&app, "serena");
        assert_eq!(
            app.marked,
            BTreeSet::from([(serena, Agent::Codex), (serena, Agent::Grok)])
        );
        app.handle(Key::Char('a'));
        assert!(app.marked.is_empty());
    }

    #[test]
    fn scope_toggle_flows_into_the_review_plan() {
        let mut app = app();
        go_to(&mut app, "linear");
        assert_eq!(app.scopes[app.row], Scope::Project);
        app.handle(Key::Char(' '));
        app.handle(Key::Char('s'));
        app.handle(Key::Enter);
        assert_eq!(app.screen, Screen::Review);
        let plan = app.plan();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].label, "linear → Claude");
        assert!(plan[0].argv.contains(&"user".to_string()));
        assert_eq!(app.handle(Key::Char('y')), Action::Install);
    }

    #[test]
    fn enter_without_marks_stays_on_the_list() {
        let mut app = app();
        assert_eq!(app.handle(Key::Enter), Action::None);
        assert_eq!(app.screen, Screen::List);
        assert!(app.status.contains("nothing marked"));
    }

    #[test]
    fn renders_matrix_and_review_overlay() {
        let mut app = app();
        go_to(&mut app, "figma");
        app.handle(Key::Char(' '));
        let screen = draw(&mut app);
        assert!(screen.contains("Claude"));
        assert!(screen.contains("figma"));
        assert!(screen.contains("[✓]"));
        app.handle(Key::Enter);
        let screen = draw(&mut app);
        assert!(screen.contains("claude mcp add-json"), "{screen}");
    }

    fn draw(app: &mut App) -> String {
        let (w, h) = (120u16, 30u16);
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        let buf = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }
}
