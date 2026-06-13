use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use amethystate::{amethystate, DefaultStore, StoreBuilder};
use std::io::{stdout, Stdout};
use std::sync::Arc;

#[amethystate(prefix = "tui_settings", mode = "persistent")]
pub struct TuiSettings {
    #[amestate(default = "Anonymous".to_string())]
    pub username: String,
    #[amestate(default = "Dark".to_string())]
    pub theme: String,
    #[amestate(default = true)]
    pub enable_notifications: bool,
    #[amestate(default = 5)]
    pub refresh_interval_secs: u32,
}

const THEMES: &[&str] = &["Dark", "Light", "Nord", "Gruvbox"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedField {
    Username,
    Theme,
    Notifications,
    RefreshRate,
}

impl SelectedField {
    fn next(self) -> Self {
        match self {
            Self::Username => Self::Theme,
            Self::Theme => Self::Notifications,
            Self::Notifications => Self::RefreshRate,
            Self::RefreshRate => Self::Username,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Username => Self::RefreshRate,
            Self::Theme => Self::Username,
            Self::Notifications => Self::Theme,
            Self::RefreshRate => Self::Notifications,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let store: Arc<DefaultStore> = StoreBuilder::new("./ratatui-settings").build()?;
    let mut state = TuiSettings::load(&store)?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = app_result {
        eprintln!("Application error: {err}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut TuiSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut selected = SelectedField::Username;
    let mut edit_mode = false;
    let mut input_buffer = String::new();

    loop {
        let current_username = &state.username;
        let current_theme = &state.theme;
        let current_notifications = state.enable_notifications;
        let current_refresh = state.refresh_interval_secs;

        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(6),
                    Constraint::Length(3),
                ])
                .split(size);

            let header = Paragraph::new("amethystate (Persistence-only) + Ratatui Settings Example")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(header, chunks[0]);

            let mut list_spans = Vec::new();

            let username_style = if selected == SelectedField::Username {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let username_display = if edit_mode && selected == SelectedField::Username {
                format!("{} <Editing: Enter to save, Esc to cancel>", input_buffer)
            } else {
                current_username.clone()
            };
            list_spans.push(Line::from(vec![
                Span::styled("  Username:      ", username_style),
                Span::raw(username_display),
            ]));

            let theme_style = if selected == SelectedField::Theme {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            list_spans.push(Line::from(vec![
                Span::styled("  Theme:         ", theme_style),
                Span::raw(format!("{}  (Space/Enter to toggle)", current_theme)),
            ]));

            let notif_style = if selected == SelectedField::Notifications {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            list_spans.push(Line::from(vec![
                Span::styled("  Notifications: ", notif_style),
                Span::raw(format!("{}  (Space/Enter to toggle)", if current_notifications { "ON" } else { "OFF" })),
            ]));

            let refresh_style = if selected == SelectedField::RefreshRate {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            list_spans.push(Line::from(vec![
                Span::styled("  Refresh Rate:  ", refresh_style),
                Span::raw(format!("{}s  (Left/Right arrows to change)", current_refresh)),
            ]));

            let form_block = Block::default()
                .title(" Application Settings ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let form_p = Paragraph::new(list_spans)
                .block(form_block)
                .wrap(Wrap { trim: true });
            f.render_widget(form_p, chunks[1]);

            let info_text = vec![
                Line::from(vec![
                    Span::styled("Operation Mode:   ", Style::default().fg(Color::Magenta)),
                    Span::raw("Persistence-only mode"),
                ]),
                Line::from(vec![
                    Span::styled("Save Method:      ", Style::default().fg(Color::Magenta)),
                    Span::raw("mutate_lazy() - deferred background write to disk"),
                ]),
                Line::from(vec![
                    Span::raw("All reactive subscriptions and Field wrappers are removed."),
                ]),
                Line::from(vec![
                    Span::raw("Data is mutated directly in RAM, minimizing overhead."),
                ]),
            ];
            let info_block = Block::default()
                .title(" Persistence-only Features ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta));
            let info_p = Paragraph::new(info_text).block(info_block);
            f.render_widget(info_p, chunks[2]);

            let help_text = if edit_mode {
                " [Input...]  [Enter] Save   [Esc] Cancel edit "
            } else {
                " [▲/▼] Navigate   [Space/Enter/◄/►] Change Value   [Esc] Exit "
            };
            let help = Paragraph::new(help_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(help, chunks[3]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if edit_mode {
                        match key.code {
                            KeyCode::Enter => {
                                let trimmed = input_buffer.trim().to_string();
                                if !trimmed.is_empty() {
                                    state.mutate_lazy(|d| d.username = trimmed)?;
                                }
                                edit_mode = false;
                            }
                            KeyCode::Esc => {
                                edit_mode = false;
                            }
                            KeyCode::Backspace => {
                                input_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                if input_buffer.len() < 24 {
                                    input_buffer.push(c);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc => {
                                return Ok(());
                            }
                            KeyCode::Up => {
                                selected = selected.prev();
                            }
                            KeyCode::Down | KeyCode::Tab => {
                                selected = selected.next();
                            }
                            KeyCode::Char(' ') | KeyCode::Enter => match selected {
                                SelectedField::Username => {
                                    edit_mode = true;
                                    input_buffer = state.username.clone();
                                }
                                SelectedField::Theme => {
                                    let current_idx = THEMES.iter().position(|&t| t == state.theme).unwrap_or(0);
                                    let next_idx = (current_idx + 1) % THEMES.len();
                                    state.mutate_lazy(|d| d.theme = THEMES[next_idx].to_string())?;
                                }
                                SelectedField::Notifications => {
                                    state.mutate_lazy(|d| d.enable_notifications = !d.enable_notifications)?;
                                }
                                _ => {}
                            },
                            KeyCode::Left => {
                                if selected == SelectedField::RefreshRate && state.refresh_interval_secs > 1 {
                                    state.mutate_lazy(|d| d.refresh_interval_secs -= 1)?;
                                }
                            }
                            KeyCode::Right => {
                                if selected == SelectedField::RefreshRate && state.refresh_interval_secs < 10 {
                                    state.mutate_lazy(|d| d.refresh_interval_secs += 1)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}