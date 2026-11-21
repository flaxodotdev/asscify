//! Ratatui rendering for browse, showcase, and help modes.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode};

// Neutral dark theme — charcoal / zinc, no purple chrome.
const ACCENT: Color = Color::Rgb(228, 228, 231); // zinc-200
const ACCENT_DIM: Color = Color::Rgb(63, 63, 70); // zinc-700
const BORDER: Color = Color::Rgb(39, 39, 42); // zinc-800
const PANEL: Color = Color::Rgb(9, 9, 11); // zinc-950
const MUTED: Color = Color::Rgb(113, 113, 122); // zinc-500
const OK: Color = Color::Rgb(161, 161, 170); // zinc-400
const HIGHLIGHT_BG: Color = Color::Rgb(24, 24, 27); // zinc-900
const KEY: Color = Color::Rgb(250, 250, 250); // near-white for keycaps

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    match app.mode {
        Mode::Help => draw_help(frame, area),
        Mode::Showcase => draw_showcase(frame, app, area),
        Mode::Browse => draw_browse(frame, app, area),
    }
}

fn draw_browse(frame: &mut Frame, app: &mut App, area: Rect) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(4),
        ])
        .split(area);

    draw_header(
        frame,
        root[0],
        "asscify",
        "image → ASCII  ·  dark terminal gallery",
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(26), Constraint::Percentage(74)])
        .split(root[1]);

    draw_file_list(frame, app, body[0]);
    draw_preview(frame, app, body[1], "preview");
    draw_footer(frame, app, root[2], browse_hints());
}

fn draw_showcase(frame: &mut Frame, app: &mut App, area: Rect) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(4),
        ])
        .split(area);

    let title = if app.images.is_empty() {
        "showcase  ·  procedural orb  ·  drop images in this folder"
    } else {
        "showcase  ·  fullscreen ASCII stage"
    };
    draw_header(frame, root[0], "asscify", title);
    draw_preview(frame, app, root[1], "ASCII STAGE");
    draw_footer(frame, app, root[2], showcase_hints());
}

fn draw_header(frame: &mut Frame, area: Rect, title: &str, subtitle: &str) {
    let line = Line::from(vec![
        Span::styled(" ▍ ", Style::default().fg(ACCENT)),
        Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(subtitle, Style::default().fg(MUTED)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL));
    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_file_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = if app.images.is_empty() {
        vec![ListItem::new(Span::styled(
            "  (no images here)",
            Style::default().fg(MUTED).add_modifier(Modifier::ITALIC),
        ))]
    } else {
        app.images
            .iter()
            .map(|p| {
                let name = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("?")
                    .to_string();
                ListItem::new(Line::from(vec![
                    Span::styled("· ", Style::default().fg(ACCENT_DIM)),
                    Span::raw(name),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(" images ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("({}) ", app.images.len()),
                        Style::default().fg(MUTED),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .style(Style::default().bg(PANEL).fg(Color::White)),
        )
        .highlight_style(
            Style::default()
                .bg(HIGHLIGHT_BG)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_preview(frame: &mut Frame, app: &mut App, area: Rect, title: &str) {
    let inner = Block::default().borders(Borders::ALL).inner(area);
    // Scale ASCII to the live panel size before painting the title.
    app.fit_to_panel(inner.width, inner.height);

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                format!(" {title} "),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "[{} · {}×{} · zoom {:.0}%{}{}] ",
                    app.opts.style.name(),
                    app.opts.width,
                    app.preview.as_ref().map(|a| a.height).unwrap_or(0),
                    app.zoom * 100.0,
                    if app.opts.color { " · color" } else { "" },
                    if app.opts.invert { " · inv" } else { "" },
                ),
                Style::default().fg(MUTED),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    if let Some(err) = &app.preview_error {
        let p = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, inner);
        return;
    }

    let Some(art) = &app.preview else {
        let p = Paragraph::new("select an image  ·  or press s for showcase")
            .style(Style::default().fg(MUTED))
            .centered();
        frame.render_widget(p, inner);
        return;
    };

    let art_h = art.height as u16;
    let art_w = art.width.min(u16::MAX as usize) as u16;
    let pad_y = inner.height.saturating_sub(art_h) / 2;
    let pad_x = inner.width.saturating_sub(art_w) / 2;

    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(pad_y),
            Constraint::Length(art_h.min(inner.height)),
            Constraint::Min(0),
        ])
        .split(inner);

    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(pad_x),
            Constraint::Length(art_w.min(inner.width)),
            Constraint::Min(0),
        ])
        .split(v[1]);

    let lines = art.as_compact_lines();
    let p = Paragraph::new(lines)
        .scroll((app.scroll, 0))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(p, h[1]);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect, hints: Line<'static>) {
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER))
            .style(Style::default().bg(PANEL)),
        area,
    );

    let inner = Block::default().borders(Borders::ALL).inner(area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let status = if let Some(f) = app.flash_text() {
        Line::from(Span::styled(
            format!(" {f} "),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::styled(
            format!(" {} ", app.status),
            Style::default().fg(OK),
        ))
    };

    frame.render_widget(Paragraph::new(status), rows[0]);
    frame.render_widget(Paragraph::new(hints), rows[1]);
}

fn browse_hints() -> Line<'static> {
    hint_line(&[
        ("↑↓", "nav"),
        ("+/-", "zoom"),
        ("0", "fit"),
        ("[]", "style"),
        ("c", "color"),
        ("s", "showcase"),
        ("w", "SAVE"),
        ("?", "help"),
        ("q", "quit"),
    ])
}

fn showcase_hints() -> Line<'static> {
    hint_line(&[
        ("esc", "back"),
        ("+/-", "zoom"),
        ("0", "fit"),
        ("[]", "style"),
        ("c", "color"),
        ("w", "SAVE .ascii.txt"),
        ("?", "help"),
        ("q", "quit"),
    ])
}

fn hint_line(items: &[(&'static str, &'static str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, label)) in items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }
        spans.push(Span::styled(
            format!(" {key} "),
            Style::default()
                .fg(Color::Black)
                .bg(KEY)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(MUTED),
        ));
    }
    Line::from(spans)
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(72, 84, area);
    frame.render_widget(Clear, popup);

    let text = vec![
        Line::from(Span::styled(
            "asscify — how to use",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "SAVE",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        help_line("w", "write current ASCII to <name>.ascii.txt in this folder"),
        help_line("", "browse mode uses the image stem; showcase → showcase.ascii.txt"),
        Line::from(""),
        Line::from(Span::styled(
            "NAVIGATION",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        help_line("↑ ↓ / j k", "move through images in the list"),
        help_line("s / Enter", "fullscreen showcase stage"),
        help_line("esc", "leave showcase (back to list)"),
        help_line("r", "rescan the directory for new images"),
        Line::from(""),
        Line::from(Span::styled(
            "LOOK",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        help_line("+ / -", "zoom in or out (art auto-fits the panel)"),
        help_line("0", "reset zoom to fill the panel"),
        help_line("[ / ]", "cycle styles: classic blocks detailed minimal binary"),
        help_line("c", "toggle truecolor"),
        help_line("i", "invert luminance"),
        Line::from(""),
        Line::from(Span::styled(
            "CLI",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        help_line("", "asscify convert photo.png -w 100 -s blocks"),
        help_line("", "asscify showcase -w 80"),
        Line::from(""),
        Line::from(Span::styled(
            "press ? or esc to close",
            Style::default().fg(MUTED),
        )),
    ];

    let block = Block::default()
        .title(" help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT_DIM))
        .style(Style::default().bg(PANEL).fg(Color::White));

    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
        popup,
    );
}

fn help_line(key: &str, desc: &str) -> Line<'static> {
    if key.is_empty() {
        Line::from(Span::styled(
            format!("  {desc}"),
            Style::default().fg(MUTED),
        ))
    } else {
        Line::from(vec![
            Span::styled(
                format!("  {key:<14}"),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(desc.to_string(), Style::default().fg(OK)),
        ])
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
