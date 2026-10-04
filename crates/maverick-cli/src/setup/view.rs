use super::app::{AddField, App, Screen};
use super::theme;
use maverick_core::{ModelRef, ProviderConfig, ProviderKind};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(frame.area());
    render_header(frame, chunks[0], app);
    match app.screen {
        Screen::Add => render_add(frame, chunks[1], app),
        Screen::Grok => render_grok(frame, chunks[1], app),
        Screen::Models | Screen::Help | Screen::ConfirmQuit => {
            render_models(frame, chunks[1], app);
        }
    }
    render_footer(frame, chunks[2], app);
    match app.screen {
        Screen::Help => render_help_overlay(frame, frame.area()),
        Screen::ConfirmQuit => render_quit_overlay(frame, frame.area()),
        _ => {}
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let default = app
        .setup
        .default
        .as_ref()
        .map(|m| format!("{}/{}", m.provider, m.model))
        .unwrap_or_else(|| "none".into());
    let grok = if app.grok_cli {
        Span::styled("grok CLI present", theme::ok())
    } else {
        Span::styled("grok CLI missing", theme::bad())
    };
    let api = if app.xai_key {
        Span::styled("XAI_API_KEY set", theme::ok())
    } else {
        Span::styled("XAI_API_KEY unset", theme::muted())
    };
    let dirty = if app.dirty {
        Line::from(Span::styled(" unsaved ", theme::accent())).right_aligned()
    } else {
        Line::from(Span::styled(" user config ", theme::muted())).right_aligned()
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("default ", theme::muted()),
            Span::styled(default, theme::accent()),
            Span::raw("   "),
            Span::styled("planner ", theme::muted()),
            Span::styled(
                format!("{}", app.setup.planner().len()),
                theme::planner().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![grok, Span::raw("   "), api]),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(theme::panel("MAVERICK  model setup").title(dirty)),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let keys = match app.screen {
        Screen::Add => "tab next · shift-tab prev · up/down kind · enter save · esc cancel",
        Screen::Grok => "esc back · this screen does not launch grok",
        Screen::Help => "esc back",
        Screen::ConfirmQuit => "y quit · s save · esc cancel",
        Screen::Models => {
            "enter default · space planner · a add · g grok · r scan · s save · ? · q"
        }
    };
    let status = if app.status.is_empty() {
        "ready"
    } else {
        app.status.as_str()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(keys, theme::muted())))
            .block(theme::panel("Status").title(Line::from(format!(" {status} ")).right_aligned()))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_models(frame: &mut Frame, area: Rect, app: &mut App) {
    let entries = app.entries();
    if entries.is_empty() {
        frame.render_widget(empty_state(app), area);
        return;
    }
    let (list_area, detail_area) = if area.width >= 72 {
        let cols = Layout::horizontal([Constraint::Fill(3), Constraint::Fill(2)]).split(area);
        (cols[0], Some(cols[1]))
    } else {
        (area, None)
    };
    let planner = app.setup.planner().to_vec();
    let default = app.setup.default.clone();
    let items: Vec<ListItem> = entries
        .iter()
        .map(|model| ListItem::new(model_line(model, default.as_ref(), &planner)))
        .collect();
    let list = List::new(items)
        .highlight_symbol("> ")
        .highlight_style(theme::selected())
        .block(theme::panel("Models"));
    frame.render_stateful_widget(list, list_area, &mut app.list_state);
    if let Some(detail_area) = detail_area {
        render_inspector(frame, detail_area, app, &entries);
    }
}

fn empty_state(app: &App) -> Paragraph<'static> {
    let grok = if app.grok_cli {
        "Grok CLI is on PATH."
    } else {
        "Grok CLI is not on PATH."
    };
    let api = if app.xai_key {
        "XAI_API_KEY is set."
    } else {
        "XAI_API_KEY is unset."
    };
    Paragraph::new(vec![
        Line::from(Span::styled("No models yet", theme::accent())),
        Line::from(""),
        Line::from("Install grok or ollama and press r to rediscover,"),
        Line::from("or press a to add an OpenAI-compatible provider."),
        Line::from(""),
        Line::from(grok),
        Line::from(api),
        Line::from(""),
        Line::from(Span::styled(
            "Models stay data. This TUI does not call providers.",
            theme::muted(),
        )),
    ])
    .block(theme::panel("Models"))
    .wrap(Wrap { trim: true })
}

fn model_line<'a>(model: &ModelRef, default: Option<&ModelRef>, planner: &[ModelRef]) -> Line<'a> {
    let mut spans = vec![Span::raw(format!("{}/{}", model.provider, model.model))];
    if default == Some(model) {
        spans.push(Span::styled("  [default]", theme::accent()));
    }
    if planner.iter().any(|m| m == model) {
        spans.push(Span::styled(" [planner]", theme::planner()));
    }
    Line::from(spans)
}

fn render_inspector(frame: &mut Frame, area: Rect, app: &App, entries: &[ModelRef]) {
    let Some(model) = entries.get(app.selected) else {
        frame.render_widget(
            Paragraph::new("Select a model or add a provider.").block(theme::panel("Selected")),
            area,
        );
        return;
    };
    let provider = app.setup.providers.iter().find(|p| p.id == model.provider);
    let planner = app.setup.planner();
    let planner_index = planner.iter().position(|m| m == model);
    let mut lines = vec![
        Line::from(Span::styled(
            format!("{}/{}", model.provider, model.model),
            theme::accent(),
        )),
        Line::from(""),
    ];
    if let Some(provider) = provider {
        lines.extend(provider_lines(provider));
    }
    lines.push(Line::from(vec![
        Span::styled("default  ", theme::muted()),
        if app.setup.default.as_ref() == Some(model) {
            Span::styled("yes  (enter to keep)", theme::ok())
        } else {
            Span::raw("no  (enter to set)")
        },
    ]));
    lines.push(Line::from(vec![
        Span::styled("planner  ", theme::muted()),
        match planner_index {
            Some(i) => Span::styled(
                format!("yes  #{}  (space to remove)", i + 1),
                theme::planner(),
            ),
            None => Span::raw("no  (space to add)"),
        },
    ]));
    if !planner.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Planner panel", theme::planner())));
        for (i, item) in planner.iter().enumerate() {
            lines.push(Line::from(format!(
                "  {}. {}/{}",
                i + 1,
                item.provider,
                item.model
            )));
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Selected"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn provider_lines(provider: &ProviderConfig) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("provider ", theme::muted()),
            Span::raw(provider.display_name.clone()),
        ]),
        Line::from(vec![
            Span::styled("kind     ", theme::muted()),
            Span::raw(theme::kind_label(provider.kind)),
        ]),
    ];
    if let Some(url) = &provider.base_url {
        lines.push(Line::from(vec![
            Span::styled("base_url ", theme::muted()),
            Span::raw(url.clone()),
        ]));
    }
    if let Some(env) = &provider.auth_env {
        lines.push(Line::from(vec![
            Span::styled("auth_env ", theme::muted()),
            Span::raw(env.clone()),
        ]));
    }
    lines.push(Line::from(""));
    lines
}

fn render_add(frame: &mut Frame, area: Rect, app: &App) {
    let kind = theme::kind_label(app.add.provider_kind());
    let mut lines = Vec::new();
    for field in app.add.visible_fields() {
        let active = app.add.field == field;
        let (label, value, cursor) = match field {
            AddField::Kind => ("kind", kind, false),
            AddField::Id => ("id", app.add.id.as_str(), true),
            AddField::DisplayName => ("name", app.add.display_name.as_str(), true),
            AddField::BaseUrl => ("base_url", app.add.base_url.as_str(), true),
            AddField::AuthEnv => ("auth_env", app.add.auth_env.as_str(), true),
            AddField::Models => ("models", app.add.models.as_str(), true),
        };
        let mut line = field_line(active, label, value, cursor && active);
        if field == AddField::Kind {
            line.spans.push(Span::styled("  up/down", theme::muted()));
        }
        lines.push(line);
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        theme::kind_hint(app.add.provider_kind()),
        theme::muted(),
    )));
    lines.push(Line::from(Span::styled(
        "auth_env is the variable NAME. Do not paste secrets.",
        theme::muted(),
    )));
    if let Some(error) = &app.add.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(error.clone(), theme::error())));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Add provider"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn field_line<'a>(active: bool, label: &'a str, value: &'a str, cursor: bool) -> Line<'a> {
    let mark = if active { "> " } else { "  " };
    let caret = if cursor { "█" } else { "" };
    let style = if active {
        theme::accent()
    } else {
        Style::default()
    };
    Line::from(vec![
        Span::styled(mark, style),
        Span::styled(format!("{label:<10} "), theme::muted()),
        Span::styled(format!("{value}{caret}"), style),
    ])
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
    let cols = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(area);
    let cli = vec![
        Line::from(Span::styled("Grok CLI", theme::accent())),
        Line::from(""),
        Line::from(if app.grok_cli {
            Span::styled("available on PATH", theme::ok())
        } else {
            Span::styled("not on PATH", theme::bad())
        }),
        Line::from(format!("CLI models: {models}")),
        Line::from(""),
        Line::from(Span::styled(
            "Grok is a provider implementation, not the runtime's internal model.",
            theme::muted(),
        )),
        Line::from(Span::styled(
            "This screen does not launch the grok TUI.",
            theme::muted(),
        )),
    ];
    let api_lines = vec![
        Line::from(Span::styled("xAI API", theme::accent())),
        Line::from(""),
        Line::from(if api.is_some() {
            Span::styled("provider configured", theme::ok())
        } else {
            Span::styled("provider absent", theme::muted())
        }),
        Line::from(if app.xai_key {
            Span::styled("XAI_API_KEY set in environment", theme::ok())
        } else {
            Span::styled("XAI_API_KEY unset", theme::muted())
        }),
        Line::from(""),
        Line::from("To use the API directly: export XAI_API_KEY and press r."),
        Line::from("Planning can include every model marked [planner]."),
    ];
    frame.render_widget(
        Paragraph::new(cli)
            .block(theme::panel("Grok"))
            .wrap(Wrap { trim: true }),
        cols[0],
    );
    frame.render_widget(
        Paragraph::new(api_lines)
            .block(theme::panel("API"))
            .wrap(Wrap { trim: true }),
        cols[1],
    );
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup = popup_area(area, 70, 12);
    frame.render_widget(Clear, popup);
    let text = "\
enter  set default model
space  toggle planner panel (order preserved)
a      add provider (OpenAI-compatible, Ollama, xAI API, CLIs)
g      Grok details
r      rediscover local CLIs (does not save)
s      save user config
? / h  help     q  quit
Models stay data. Providers are not called from this TUI.";
    frame.render_widget(
        Paragraph::new(text)
            .block(theme::overlay("Help"))
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_quit_overlay(frame: &mut Frame, area: Rect) {
    let popup = popup_area(area, 48, 7);
    frame.render_widget(Clear, popup);
    let text = "\
unsaved changes
y  quit without saving
s  save
esc  cancel";
    frame.render_widget(Paragraph::new(text).block(theme::overlay("Quit")), popup);
}

fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let [horizontal] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas::<1>(horizontal)[0]
}
