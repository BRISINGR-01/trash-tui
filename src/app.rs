use color_eyre::eyre::Result;
use crossterm::event::{self};
use ratatui::{DefaultTerminal, Frame};
use tui_input::Input;

use crate::{
    list::ListContainer,
    ui::{
        Message, layout, render_choice_popup, render_empty_list, render_footer, render_list,
        render_message, render_scrollbar, render_search_input,
    },
    utils::{Choice, Mode, SortMode, compute_list_size, with_search},
};

pub struct App {
    pub input: Input,
    pub mode: Mode,
    pub sort_mode: SortMode,
    pub choice_popup: Option<Choice>,
    pub message: Option<Message>,
    pub list_container: ListContainer,
}

impl App {
    pub fn new(terminal: &mut DefaultTerminal) -> Self {
        let default_sorting = SortMode::DateAsc;

        App {
            mode: Mode::ListView,
            input: Input::default(),
            list_container: ListContainer::new(compute_list_size(terminal), &default_sorting),
            message: None,
            choice_popup: None,
            sort_mode: default_sorting,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let event = event::read()?;

            if event.is_resize() {
                self.list_container.resize(compute_list_size(terminal));
            }

            if let Some(key) = event.as_key_press_event() {
                match self.handle_key_press(key, &event) {
                    Ok(true) => return Ok(()),
                    Err(e) => return Err(e.into()),
                    _ => {}
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [input_area, list_area, bottom_area] = layout(&self.mode).areas(frame.area());

        render_footer(frame, bottom_area, &self.mode);

        if self.list_container.items.is_empty() {
            render_empty_list(frame, list_area);
        } else {
            render_list(
                frame,
                list_area,
                with_search(&self.list_container.items, self.input.value()),
                &mut self.list_container.state,
            );
        }
        render_scrollbar(frame, list_area, &self.list_container);

        if matches!(self.mode, Mode::Filtering) {
            render_search_input(frame, input_area, &self.input)
        }

        if let Some(choice) = &self.choice_popup {
            render_choice_popup(
                frame,
                match choice {
                    Choice::Restore => "Restore selected item?",
                    Choice::Delete => "Delete selected item?",
                    Choice::Empty => "Empty the trash?",
                    Choice::Override => "Override existing file?",
                },
            );
        }

        if let Some(message) = &self.message {
            render_message(frame, message);
        }
    }
}
