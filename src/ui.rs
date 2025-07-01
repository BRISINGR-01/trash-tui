use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Wrap,
    },
};

use crate::{list::ListContainer, trash_entry::TrashEntry, utils::Mode};
use std::{cmp::min, ops::Range};

const SECONDARY_COLOR: Color = Color::LightBlue;
const TERTIARY_COLOR: Color = Color::LightBlue;

pub struct Message {
    pub text: String,
    pub is_error: bool,
}

impl Message {
    pub fn error(text: String) -> Self {
        Message {
            text,
            is_error: true,
        }
    }

    pub fn info(text: String) -> Self {
        Message {
            text,
            is_error: false,
        }
    }
}

pub fn layout(input_mode: &Mode) -> Layout {
    let input_size = if matches!(input_mode, Mode::Filtering) {
        3
    } else {
        0
    };

    Layout::vertical([
        Constraint::Length(input_size),
        Constraint::Fill(2),
        Constraint::Length(1),
    ])
}

pub fn render_empty_list(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Trash is empty")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(fg(SECONDARY_COLOR))
                    .title("Trash TUI")
                    .bold(),
            )
            .alignment(Alignment::Center),
        area,
    );
}

pub fn render_list(frame: &mut Frame, area: Rect, items: Vec<Row>, state: &mut TableState) {
    frame.render_stateful_widget(
        Table::new(
            items,
            [
                Constraint::Fill(1),
                Constraint::Length("%Y-%m-%d %H:%M:%S".len() as u16 + 3),
                // + 3 is for padding.
            ],
        )
        .row_highlight_style(fg(Color::White).bg(SECONDARY_COLOR).bold())
        .highlight_symbol(">> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(fg(SECONDARY_COLOR))
                .title("Trash TUI")
                .bold(),
        ),
        area,
        state,
    );
}

pub fn render_scrollbar(frame: &mut Frame, area: Rect, list: &ListContainer) {
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .style(fg(SECONDARY_COLOR));

    frame.render_stateful_widget(
        scrollbar,
        area,
        &mut ScrollbarState::default()
            .content_length(list.items.len())
            .position(list.state.selected().unwrap_or(0)),
    );
}

pub fn render_search_input(frame: &mut Frame, area: Rect, input: &tui_input::Input) {
    let input_line = Line::from(vec![
        Span::from("  ").dim(),
        Span::from(input.value()).bold(),
    ])
    .style(fg(TERTIARY_COLOR));

    let paragraph = Paragraph::new(input_line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(fg(SECONDARY_COLOR)),
        )
        .style(fg(Color::White));

    let x = input.visual_cursor() + 4; // padding for icon + 1

    frame.set_cursor_position((area.x + x as u16, area.y + 1));
    frame.render_widget(paragraph, area);
}

pub fn render_footer(frame: &mut Frame, area: Rect, mode: &Mode) {
    fn special(str: &str) -> Span {
        Span::from(str).style(fg(TERTIARY_COLOR)).bold()
    }

    let footer = match mode {
        Mode::Sorting => Line::from(vec![
            Span::from("Sort by: "),
            special("d"),
            Span::from(" - date, "),
            special("D"),
            Span::from(" - date descending, "),
            special("n"),
            Span::from(" - name, "),
            special("N"),
            Span::from(" - name descending, "),
        ]),
        _ => Line::from(vec![
            special("◄ ▲ ▼ ►"),
            Span::from(" - move, "),
            special("<q>"),
            Span::from(" - quit, "),
            special("<enter>"),
            Span::from(" - restore, "),
            special("<f>"),
            Span::from(" - search, "),
            special("<d>"),
            Span::from(" - delete, "),
            special("<D>"),
            Span::from(" - empty trash"),
        ]),
    };

    frame.render_widget(footer, area);
}

pub fn render_choice_popup(frame: &mut Frame, question: &str) {
    let w = 30;
    let h = 7;

    let x = frame.area().x + (frame.area().width.saturating_sub(w)) / 2;
    let y = frame.area().y + (frame.area().height.saturating_sub(h)) / 2;
    let area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(fg(SECONDARY_COLOR))
        .title("Confirm ");

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::from(question).bold()]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[ Enter ]", fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("[ Esc ]", fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
    ])
    .block(block)
    .alignment(Alignment::Center);

    frame.render_widget(text, area);
}

pub fn render_message(frame: &mut Frame, message: &Message) {
    let text = Paragraph::new(message.text.to_string())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(fg(SECONDARY_COLOR)),
        )
        .style(fg(match message.is_error {
            true => Color::Red,
            false => TERTIARY_COLOR,
        }))
        .wrap(Wrap { trim: true });

    let message_len = (
        message.text.len() + 2
        // borders
    ) as u16;
    let w = min(
        (frame.area().width as f32 / 1.2) as u16, // maximum width
        message_len,
    );
    let h = min(
        (frame.area().height as f32 / 1.2) as u16, // maximum height
        2 // borders
         + (message_len / w),  // number of lines
    );

    let x = frame.area().x + (frame.area().width.saturating_sub(w));
    let y = frame.area().y;
    let area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, area);
    frame.render_widget(text, area);
}

pub fn make_row_widget<'a>(item: &'a TrashEntry, ranges: Option<Vec<Range<usize>>>) -> Row<'a> {
    let date = Span::from(item.date.format("%Y-%m-%d %H:%M:%S").to_string()).fg(TERTIARY_COLOR);

    match ranges {
        Some(result) if !result.is_empty() => {
            let mut characters = Vec::new();

            let mut i = 0;
            for range in result.iter() {
                characters
                    .push(Span::from(item.name[i..range.start].to_string()).fg(TERTIARY_COLOR));
                i = range.end;

                characters.push(
                    Span::from(item.name[range.start..range.end].to_string())
                        .bold()
                        .underlined()
                        .fg(TERTIARY_COLOR),
                );
            }
            characters
                .push(Span::from(item.name[i..item.name.len()].to_string()).fg(TERTIARY_COLOR));

            Row::new(vec![Line::from(characters), Line::from(date)])
        }
        _ => Row::new(vec![Span::from(item.name.clone()).fg(TERTIARY_COLOR), date]),
    }
}

fn fg(color: Color) -> Style {
    Style::default().fg(color)
}
