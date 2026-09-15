use super::{
    app::{Action, App, Key},
    config::save_setup,
    discover::{discover, Probe, SystemProbe},
    resolve, ConfigPaths,
};
use anyhow::Result;
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;

pub fn run(paths: ConfigPaths) -> Result<()> {
    let setup = resolve(&paths)?;
    let grok_cli = SystemProbe.binary_exists("grok");
    let xai_key = SystemProbe.env("XAI_API_KEY").is_some();
    let mut app = App::new(setup, grok_cli, xai_key);
    ratatui::run(|terminal: &mut DefaultTerminal| loop {
        terminal.draw(|frame| super::app::render(frame, &mut app))?;
        let Some(key) = event::read()?.as_key_press_event() else {
            continue;
        };
        let mapped = match key.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Tab => Key::Tab,
            KeyCode::Backspace => Key::Backspace,
            _ => continue,
        };
        match app.handle(mapped) {
            Action::None => {}
            Action::Quit => break Ok::<(), std::io::Error>(()),
            Action::Save => match save_setup(&paths.user, &app.setup) {
                Ok(()) => {
                    app.dirty = false;
                    app.status = format!("saved {}", paths.user.display());
                    app.screen = super::app::Screen::Models;
                }
                Err(e) => app.status = e.to_string(),
            },
            Action::Rediscover => match discover(&SystemProbe) {
                Ok(discovered) => {
                    app.grok_cli = SystemProbe.binary_exists("grok");
                    app.xai_key = SystemProbe.env("XAI_API_KEY").is_some();
                    app.apply_discovery(discovered);
                }
                Err(e) => app.status = e.to_string(),
            },
        }
    })?;
    Ok(())
}
