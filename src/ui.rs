use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};
use crate::models::Lesson;

pub fn ui(f: &mut Frame, app: &mut App) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(vertical_chunks[0]);

    render_list(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_progress(f, app, vertical_chunks[1]);
}

fn render_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .lessons
        .iter()
        .map(|l| {
            let is_completed = app.progress.completed_files.contains(&l.path);
            let prefix = if is_completed { "[x] " } else { "[ ] " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(if is_completed { Color::Green } else { Color::Gray })),
                Span::raw(&l.title),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(" Modules ")
        .border_style(if matches!(app.mode, Mode::List) {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let list = List::new(items)
        .block(list_block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_lesson_index));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Content (Space: Focus, c: Complete, Up/Down: Scroll) ")
        .border_style(if matches!(app.mode, Mode::Content) {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let text = if let Some(lesson) = app.lessons.get(app.selected_lesson_index) {
        Text::from(lesson.text_lines.clone())
    } else {
        Text::from("No lessons loaded.")
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    f.render_widget(paragraph, area);
}

fn render_progress(f: &mut Frame, app: &App, area: Rect) {
    let total = app.lessons.len();
    let completed = app.progress.completed_files.len();
    
    let ratio = if total > 0 {
        completed as f64 / total as f64
    } else {
        0.0
    };

    let gauge = ratatui::widgets::Gauge::default()
        .block(Block::default().title(" Overall Progress ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
        .ratio(ratio)
        .label(format!("{} / {} Modules Completed", completed, total));

    f.render_widget(gauge, area);
}
