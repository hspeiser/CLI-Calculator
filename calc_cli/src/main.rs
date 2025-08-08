use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Position};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, BorderType};
use ratatui::Terminal;

use calc_core::Engine;
use regex::Regex;

#[derive(Debug, Parser)]
struct Args {
    /// Optional file to open (one cell per line)
    #[arg(short, long)]
    file: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusPane { Left, Right }

struct App {
    engine: Engine,
    lines: Vec<String>,
    outputs: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    focus: FocusPane,
    unit_regex: Regex,
    keyword_regex: Regex,
}

impl App {
    fn new(initial: String) -> Self {
        let mut app = Self {
            engine: Engine::new(),
            lines: Vec::new(),
            outputs: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            focus: FocusPane::Left,
            unit_regex: Regex::new(r"^(k|M|m|u|μ|n|p)?(Ω|ohm|V|A|F|H|S|Hz|m|s|kg|K|mol|cd|rad|deg|°)$").unwrap(),
            keyword_regex: Regex::new(r"^(to|sin|cos|tan|asin|acos|atan|sin_deg|cos_deg|tan_deg|pi|i|j|π)$").unwrap(),
        };
        app.lines = if initial.is_empty() { vec![String::new()] } else { initial.lines().map(|s| s.to_string()).collect() };
        app.recompute();
        app
    }

    fn recompute(&mut self) {
        self.outputs.clear();
        let mut last_display = String::new();
        for line in &self.lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.outputs.push(String::new());
                continue;
            }
            let expanded = if trimmed.contains("Ans") { trimmed.replace("Ans", &last_display) } else { trimmed.to_string() };
            match self.engine.eval_cell(&expanded) {
                Ok(out) => { let disp = out.value.display(); last_display = disp.clone(); self.outputs.push(disp) }
                Err(e) => self.outputs.push(format!("error: {}", e)),
            }
        }
    }

    fn move_cursor_to(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.lines.len().saturating_sub(1));
        self.cursor_col = col.min(self.lines.get(self.cursor_row).map(|s| s.len()).unwrap_or(0));
    }

    fn handle_mouse(&mut self, me: MouseEvent, area_left: Rect, area_right: Rect) {
        match me.kind {
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                let (x, y) = (me.column.saturating_sub(1) as u16, me.row.saturating_sub(1) as u16);
                if area_left.contains(Position { x, y }) {
                    self.focus = FocusPane::Left;
                    let rel_y = y.saturating_sub(area_left.y);
                    let rel_x = x.saturating_sub(area_left.x).saturating_sub(5);
                    let row = rel_y as usize;
                    let col = rel_x as usize;
                    self.move_cursor_to(row, col);
                } else if area_right.contains(Position { x, y }) {
                    self.focus = FocusPane::Right;
                }
            }
            _ => {}
        }
    }

    fn render_highlighted_line(&self, idx: usize, content: &str, _is_left: bool) -> Line<'_> {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(format!("{:>3}  ", idx + 1), Style::default().fg(ratatui::style::Color::DarkGray)));
        let mut token = String::new();
        let mut flush_token = |spans: &mut Vec<Span>, token: &mut String, style: Style| {
            if !token.is_empty() { spans.push(Span::styled(token.clone(), style)); token.clear(); }
        };
        for ch in content.chars() {
            if ch.is_alphanumeric() || matches!(ch, '_' | 'Ω' | 'μ' | 'π' | '°') {
                token.push(ch);
            } else {
                if !token.is_empty() {
                    let style = if token.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        Style::default().fg(ratatui::style::Color::Cyan)
                    } else if self.keyword_regex.is_match(&token) {
                        Style::default().fg(ratatui::style::Color::Yellow).add_modifier(Modifier::BOLD)
                    } else if self.unit_regex.is_match(&token) {
                        Style::default().fg(ratatui::style::Color::Magenta)
                    } else { Style::default() };
                    flush_token(&mut spans, &mut token, style);
                }
                let style = if "+-*/%^(),=//".contains(ch) { Style::default().fg(ratatui::style::Color::LightBlue) } else { Style::default() };
                spans.push(Span::styled(ch.to_string(), style));
            }
        }
        if !token.is_empty() {
            let style = if token.chars().all(|c| c.is_ascii_digit() || c == '.') {
                Style::default().fg(ratatui::style::Color::Cyan)
            } else if self.keyword_regex.is_match(&token) {
                Style::default().fg(ratatui::style::Color::Yellow).add_modifier(Modifier::BOLD)
            } else if self.unit_regex.is_match(&token) {
                Style::default().fg(ratatui::style::Color::Magenta)
            } else { Style::default() };
            spans.push(Span::styled(token.clone(), style));
        }
        Line::from(spans)
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let initial = if let Some(path) = args.file { std::fs::read_to_string(path).unwrap_or_default() } else { String::new() };
    let mut app = App::new(initial);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(f.size());

            let left_block = Block::default()
                .borders(Borders::ALL)
                .border_style(if app.focus == FocusPane::Left { Style::default().fg(Color::Green) } else { Style::default() })
                .title(Span::styled("Input", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)))
                .border_type(BorderType::Rounded);

            let right_block = Block::default()
                .borders(Borders::ALL)
                .border_style(if app.focus == FocusPane::Right { Style::default().fg(Color::Green) } else { Style::default() })
                .title(Span::styled("Output", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)))
                .border_type(BorderType::Rounded);

            let left_text: Vec<Line> = app
                .lines
                .iter()
                .enumerate()
                .map(|(i, l)| app.render_highlighted_line(i, l, true))
                .collect();
            let left = Paragraph::new(left_text).block(left_block).wrap(Wrap { trim: false });
            f.render_widget(left, chunks[0]);

            // Place visible cursor in the left pane where we're typing
            if app.focus == FocusPane::Left {
                let inner_x = chunks[0].x.saturating_add(1); // account for border
                let inner_y = chunks[0].y.saturating_add(1);
                let cursor_x = inner_x.saturating_add(5).saturating_add(app.cursor_col as u16);
                let cursor_y = inner_y.saturating_add(app.cursor_row as u16);
                let max_x = chunks[0].x + chunks[0].width.saturating_sub(2);
                let max_y = chunks[0].y + chunks[0].height.saturating_sub(2);
                let cx = cursor_x.min(max_x);
                let cy = cursor_y.min(max_y);
                f.set_cursor(cx, cy);
            }

            let right_text: Vec<Line> = app
                .outputs
                .iter()
                .enumerate()
                .map(|(i, o)| {
                    let mut spans = vec![Span::styled(format!("{:>3}  ", i + 1), Style::default().fg(ratatui::style::Color::DarkGray))];
                    let style = if o.starts_with("error:") { Style::default().fg(ratatui::style::Color::Red) } else { Style::default().fg(ratatui::style::Color::White) };
                    spans.push(Span::styled(o.clone(), style));
                    Line::from(spans)
                })
                .collect();
            let right = Paragraph::new(right_text).block(right_block).wrap(Wrap { trim: false });
            f.render_widget(right, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char(c) => {
                        if app.focus == FocusPane::Left {
                            if app.cursor_row >= app.lines.len() { app.lines.push(String::new()); }
                            app.lines[app.cursor_row].insert(app.cursor_col, c);
                            app.cursor_col += 1;
                            app.recompute();
                        }
                    }
                    KeyCode::Backspace => {
                        if app.focus == FocusPane::Left && app.cursor_row < app.lines.len() {
                            if app.cursor_col > 0 {
                                app.lines[app.cursor_row].remove(app.cursor_col - 1);
                                app.cursor_col -= 1;
                                app.recompute();
                            } else if app.cursor_row > 0 {
                                let prev_row = app.cursor_row - 1;
                                let tail = app.lines.remove(app.cursor_row);
                                let prev_len = app.lines[prev_row].len();
                                app.lines[prev_row].push_str(&tail);
                                app.cursor_row = prev_row;
                                app.cursor_col = prev_len;
                                app.recompute();
                            }
                        }
                    }
                    KeyCode::Delete => {
                        if app.focus == FocusPane::Left && app.cursor_row < app.lines.len() {
                            if app.cursor_col < app.lines[app.cursor_row].len() {
                                app.lines[app.cursor_row].remove(app.cursor_col);
                                app.recompute();
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if app.focus == FocusPane::Left {
                            let new_row = app.cursor_row + 1;
                            let rest = app.lines[app.cursor_row].split_off(app.cursor_col);
                            app.lines.insert(new_row, rest);
                            app.cursor_row = new_row;
                            app.cursor_col = 0;
                            app.recompute();
                        }
                    }
                    KeyCode::Left => { if app.cursor_col > 0 { app.cursor_col -= 1; } }
                    KeyCode::Right => { if app.cursor_col < app.lines.get(app.cursor_row).map(|s| s.len()).unwrap_or(0) { app.cursor_col += 1; } }
                    KeyCode::Up => { if app.cursor_row > 0 { app.cursor_row -= 1; app.cursor_col = app.cursor_col.min(app.lines[app.cursor_row].len()); } }
                    KeyCode::Down => { if app.cursor_row + 1 < app.lines.len() { app.cursor_row += 1; app.cursor_col = app.cursor_col.min(app.lines[app.cursor_row].len()); } }
                    KeyCode::Tab => { app.focus = if app.focus == FocusPane::Left { FocusPane::Right } else { FocusPane::Left }; }
                    KeyCode::Esc => break,
                    _ => {}
                },
                Event::Mouse(me) => {
                    let size = terminal.get_frame().size();
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                        .split(size);
                    app.handle_mouse(me, chunks[0], chunks[1]);
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(LeaveAlternateScreen)?;
    stdout.execute(DisableMouseCapture)?;
    Ok(())
}
