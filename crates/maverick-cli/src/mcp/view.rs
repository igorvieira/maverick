use super::{
    app::{App, Cell as State, Screen},
    detect::Agent,
    install::{command_for, display},
};
use crate::setup::theme;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(frame.area());
    render_header(frame, chunks[0], app);
    let body = if chunks[1].width >= 100 {
        Layout::horizontal([Constraint::Length(62), Constraint::Fill(1)]).split(chunks[1])
    } else {
        Layout::vertical([Constraint::Fill(1), Constraint::Length(12)]).split(chunks[1])
    };
    render_matrix(frame, body[0], app);
    render_inspector(frame, body[1], app);
    render_footer(frame, chunks[2], app);
    match app.screen {
        Screen::Review => render_review(frame, frame.area(), app),
        Screen::Results => render_results(frame, frame.area(), app),
        Screen::Help => render_help(frame, frame.area()),
        Screen::List => {}
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![Span::styled("suggested MCP servers ", theme::accent())];
    for agent in Agent::ALL {
        let state = app.detection.agent(agent);
        let style = match (&state.error, state.on_path) {
            (Some(_), _) => theme::error(),
            (None, true) => theme::ok(),
            (None, false) => theme::muted(),
        };
        let mark = if state.on_path { "●" } else { "—" };
        spans.push(Span::styled(format!(" {mark} {} ", agent.bin()), style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(theme::panel("maverick mcp")),
        area,
    );
}

fn cell_span(state: &State) -> Span<'static> {
    match state {
        State::NoCli => Span::styled("—", theme::muted()),
        State::Active(_) => Span::styled("●", theme::ok()),
        State::Disabled(_) => Span::styled("off", theme::muted()),
        State::Blocked(_) => Span::styled("✗", theme::bad()),
        State::Stale(_) => Span::styled("! stale", theme::accent()),
        State::Open => Span::styled("○", theme::muted()),
        State::Marked => Span::styled("[✓]", theme::accent()),
    }
}

fn render_matrix(frame: &mut Frame, area: Rect, app: &mut App) {
    let header = Row::new(
        ["MCP", "scope"]
            .into_iter()
            .chain(Agent::ALL.map(Agent::label))
            .map(|h| Cell::from(h).style(theme::muted())),
    );
    let rows: Vec<Row> = app
        .catalog
        .iter()
        .enumerate()
        .map(|(row, entry)| {
            let mut cells = vec![
                Cell::from(entry.name.clone()),
                Cell::from(Span::styled(app.scopes[row].as_str(), theme::muted())),
            ];
            cells.extend(Agent::ALL.map(|agent| Cell::from(cell_span(&app.cell(row, agent)))));
            Row::new(cells)
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
        ],
    )
    .header(header)
    .block(theme::panel("catalog"))
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
    .cell_highlight_style(theme::selected())
    .highlight_symbol("› ");
    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_inspector(frame: &mut Frame, area: Rect, app: &App) {
    let entry = app.entry();
    let transport = if entry.is_http() { "HTTP" } else { "stdio" };
    let missing = app.detection.missing(&entry.name);
    let mut lines = vec![
        Line::from(entry.summary),
        Line::default(),
        Line::from(vec![
            Span::styled(format!("{transport}  "), theme::muted()),
            Span::raw(entry.target()),
        ]),
        Line::from(vec![
            Span::styled("needs  ", theme::muted()),
            if missing.is_empty() {
                Span::styled("nothing missing", theme::ok())
            } else {
                Span::styled(missing.join(", "), theme::bad())
            },
        ]),
        Line::default(),
    ];
    for agent in Agent::ALL {
        let (text, style) = match app.cell(app.row, agent) {
            State::NoCli => (format!("{} not on PATH", agent.bin()), theme::muted()),
            State::Active(place) => (format!("active ({place})"), theme::ok()),
            State::Disabled(place) => (format!("configured ({place})"), theme::muted()),
            State::Blocked(why) => (why, theme::bad()),
            State::Stale(why) => (format!("not loaded: {why}"), theme::accent()),
            State::Open => ("not installed".into(), theme::muted()),
            State::Marked => ("marked to install".into(), theme::accent()),
        };
        let pointer = if agent == app.agent() { "› " } else { "  " };
        lines.push(Line::from(vec![
            Span::styled(format!("{pointer}{:<7}", agent.label()), theme::planner()),
            Span::styled(text, style),
        ]));
    }
    if let Some(error) = &app.detection.agent(app.agent()).error {
        lines.push(Line::from(Span::styled(
            format!("{} listing failed: {error}", app.agent().bin()),
            theme::error(),
        )));
    }
    if let Ok(argv) = command_for(app.agent(), entry, app.scopes[app.row]) {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled("would run", theme::muted())));
        lines.push(Line::from(display(&argv)));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(theme::panel(&entry.name)),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let keys =
        "↑↓ MCP  ←→ agent  space mark  a row  s scope  enter review  r rescan  ? help  q quit";
    let line = Line::from(vec![
        Span::styled(keys, theme::muted()),
        Span::raw("  "),
        Span::styled(app.status.clone(), theme::accent()),
    ]);
    frame.render_widget(Paragraph::new(line).block(theme::panel("keys")), area);
}

fn render_review(frame: &mut Frame, area: Rect, app: &App) {
    let plan = app.plan();
    let mut lines = vec![
        Line::from(Span::styled(
            format!(
                "{} command(s) will run, nothing else is changed:",
                plan.len()
            ),
            theme::muted(),
        )),
        Line::default(),
    ];
    for step in &plan {
        lines.push(Line::from(Span::styled(
            step.label.clone(),
            theme::accent(),
        )));
        lines.push(Line::from(format!("  {}", display(&step.argv))));
    }
    if plan.iter().any(|s| s.argv[0] == "codex") {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "codex has no project scope: it always writes ~/.codex/config.toml",
            theme::muted(),
        )));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "y / enter install · any other key goes back",
        theme::accent(),
    )));
    let popup = popup_area(area, 100, lines.len() as u16 + 4);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(theme::overlay("review")),
        popup,
    );
}

fn render_results(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = app
        .results
        .iter()
        .map(|(label, result)| match result {
            Ok(()) => Line::from(vec![
                Span::styled("✓ ", theme::ok()),
                Span::raw(label.clone()),
            ]),
            Err(e) => Line::from(vec![
                Span::styled("✗ ", theme::bad()),
                Span::raw(format!("{label}: {e}")),
            ]),
        })
        .collect();
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "OAuth servers (figma, linear) ask to sign in the first time each agent uses them.",
        theme::muted(),
    )));
    lines.push(Line::from(Span::styled(
        "any key to continue",
        theme::accent(),
    )));
    let popup = popup_area(area, 90, lines.len() as u16 + 2);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(theme::overlay("installed")),
        popup,
    );
}

fn render_help(frame: &mut Frame, area: Rect) {
    let rows = [
        ("●", "active: the agent loads it"),
        ("○", "not installed: space marks it"),
        ("[✓]", "marked to install"),
        (
            "! stale",
            "only in a file the agent ignores; installing fixes it",
        ),
        ("off", "configured but disabled in that agent"),
        ("✗", "blocked: missing binary/env var or unsupported"),
        ("—", "agent CLI not on PATH"),
        ("s", "toggle the row's scope (user / project)"),
        ("r", "rescan the agents' configs"),
    ];
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, v)| {
            Line::from(vec![
                Span::styled(format!("{k:<9}"), theme::accent()),
                Span::raw(*v),
            ])
        })
        .collect();
    let popup = popup_area(area, 70, lines.len() as u16 + 2);
    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).block(theme::overlay("help")), popup);
}

fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Length(height.min(area.height))])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Length(width.min(area.width))])
        .flex(Flex::Center)
        .areas(area);
    area
}
