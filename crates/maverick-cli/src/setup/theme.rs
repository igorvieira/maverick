use maverick_core::ProviderKind;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType},
};

pub const ACCENT: Color = Color::Yellow;
pub const MUTED: Color = Color::DarkGray;
pub const OK: Color = Color::Green;
pub const BAD: Color = Color::Red;
pub const PLANNER: Color = Color::Cyan;

pub fn accent() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn ok() -> Style {
    Style::default().fg(OK)
}

pub fn bad() -> Style {
    Style::default().fg(BAD)
}

pub fn planner() -> Style {
    Style::default().fg(PLANNER)
}

pub fn selected() -> Style {
    Style::default().fg(Color::Black).bg(ACCENT)
}

pub fn error() -> Style {
    Style::default().fg(BAD)
}

pub fn panel(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MUTED))
        .title(format!(" {title} "))
        .title_style(accent())
}

pub fn overlay(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .title(format!(" {title} "))
        .title_style(accent())
}

pub fn kind_label(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Grok => "Grok CLI",
        ProviderKind::XaiApi => "xAI API",
        ProviderKind::Claude => "Claude CLI",
        ProviderKind::Codex => "Codex CLI",
        ProviderKind::Ollama => "Ollama",
        ProviderKind::OpenAiCompatible => "OpenAI-compatible",
    }
}

pub fn kind_hint(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Grok => {
            "Local grok binary. Models come from discovery, not from pasting keys."
        }
        ProviderKind::XaiApi => "Direct xAI HTTP API. auth_env is the variable NAME (XAI_API_KEY).",
        ProviderKind::Claude => "Anthropic Claude CLI on PATH.",
        ProviderKind::Codex => "OpenAI Codex CLI on PATH.",
        ProviderKind::Ollama => "Local Ollama. Default base_url is http://127.0.0.1:11434/v1.",
        ProviderKind::OpenAiCompatible => {
            "Any OpenAI-compatible endpoint. Do not paste secrets into this form."
        }
    }
}
