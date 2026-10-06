use super::{
    app::{Action, App, Key},
    catalog::catalog,
    detect::{detect, Paths},
    install,
};
use crate::setup::discover::SystemProbe;
use anyhow::Result;
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;

pub fn run() -> Result<()> {
    let paths = Paths::current();
    let entries = catalog();
    let detection = detect(&SystemProbe, &paths, &entries);
    let mut app = App::new(entries, detection);
    ratatui::run(|terminal: &mut DefaultTerminal| loop {
        terminal.draw(|frame| super::view::render(frame, &mut app))?;
        let Some(key) = event::read()?.as_key_press_event() else {
            continue;
        };
        let mapped = match key.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Tab => Key::Char('\t'),
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            _ => continue,
        };
        match app.handle(mapped) {
            Action::None => {}
            Action::Quit => break Ok::<(), std::io::Error>(()),
            Action::Redetect => app.set_detection(detect(&SystemProbe, &paths, &app.catalog)),
            Action::Install => {
                let plan = app.plan();
                app.status = format!("installing {} server(s)…", plan.len());
                app.screen = super::app::Screen::List;
                terminal.draw(|frame| super::view::render(frame, &mut app))?;
                let results = plan
                    .into_iter()
                    .map(|step| {
                        let result = install::run(&step.argv);
                        (step.label, result)
                    })
                    .collect();
                let detection = detect(&SystemProbe, &paths, &app.catalog);
                app.finish_install(results, detection);
            }
        }
    })?;
    Ok(())
}
