use std::{fs, io};

use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, Frame};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    files::{delete_item, empty_bin, restore_item},
    list::ListContainer,
    ui::{
        Message, layout, render_choice_popup, render_empty_list, render_footer, render_list,
        render_message, render_scrollbar, render_search_input,
    },
    utils::{Choice, Mode, SortMode, compute_list_size, with_search},
};

pub struct App {
    input: Input,
    mode: Mode,
    sort_mode: SortMode,
    choice: Option<Choice>,
    message: Option<Message>,
    list_container: ListContainer,
}

impl App {
    pub fn new(terminal: &mut DefaultTerminal) -> Self {
        App {
            mode: Mode::ListView,
            input: Input::default(),
            list_container: ListContainer::new(compute_list_size(terminal), &SortMode::DateAsc),
            message: None,
            choice: None,
            sort_mode: SortMode::DateAsc,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match self.handle_events() {
                Ok(true) => return Ok(()),
                Err(e) => return Err(e.into()),
                _ => {}
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [input_area, list_area, bottom_area] = layout(&self.mode).areas(frame.area());

        render_footer(frame, bottom_area, &self.mode);

        if matches!(self.mode, Mode::Filtering) {
            render_search_input(frame, input_area, &self.input)
        }

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

        if let Some(choice) = &self.choice {
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

    fn handle_events(&mut self) -> Result<bool, io::Error> {
        let event = event::read()?;

        if let Some(key) = event.as_key_press_event() {
            if self.message.is_some() {
                self.message = None; // Clear message on any key press
            }

            if self.choice.is_some() {
                return self.handle_choice_action(key);
            }

            match self.mode {
                Mode::ListView => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Esc => {
                        if matches!(self.mode, Mode::Filtering) {
                            self.mode = Mode::ListView;
                        } else {
                            return Ok(true);
                        }
                    }
                    KeyCode::Char('s') => self.mode = Mode::Sorting,
                    KeyCode::Char('f') => self.mode = Mode::Filtering,
                    KeyCode::Down | KeyCode::Char('j') => self.list_container.next(),
                    KeyCode::Up | KeyCode::Char('k') => self.list_container.prev(),
                    KeyCode::PageDown | KeyCode::Right | KeyCode::Char('l') => {
                        self.list_container.scroll_next()
                    }
                    KeyCode::PageUp | KeyCode::Left | KeyCode::Char('h') => {
                        self.list_container.scroll_prev()
                    }
                    KeyCode::Enter => {
                        if self.list_container.get_slected_item().is_some() {
                            self.choice = Some(Choice::Restore);
                        }
                    }
                    KeyCode::Char('d') => {
                        if self.list_container.get_slected_item().is_some() {
                            self.choice = Some(Choice::Delete);
                        }
                    }
                    KeyCode::Char('e') => {
                        if !self.list_container.items.is_empty() {
                            self.choice = Some(Choice::Empty);
                        }
                    }
                    _ => {}
                },
                Mode::Filtering => match key.code {
                    KeyCode::Enter => self.mode = Mode::ListView,
                    KeyCode::Esc => {
                        self.input.reset();
                        self.mode = Mode::ListView;
                        if self.list_container.get_slected_item().is_none() {
                            self.list_container.next();
                        }
                    }
                    KeyCode::Up => self.list_container.prev(),
                    KeyCode::Down => self.list_container.next(),
                    KeyCode::PageDown => self.list_container.scroll_next(),
                    KeyCode::PageUp => self.list_container.scroll_prev(),
                    _ => {
                        self.input.handle_event(&event);
                    }
                },
                Mode::Sorting => {
                    self.mode = Mode::ListView;
                    self.sort_mode = match key.code {
                        KeyCode::Char('n') => SortMode::NameAsc,
                        KeyCode::Char('N') => SortMode::NameDesc,
                        KeyCode::Char('d') => SortMode::DateAsc,
                        KeyCode::Char('D') => SortMode::DateDesc,
                        _ => SortMode::DateAsc,
                    };
                    self.list_container.sort(&self.sort_mode);
                }
            }
        };
        Ok(false)
    }

    fn handle_choice_action(&mut self, key: KeyEvent) -> Result<bool, std::io::Error> {
        let choice = self.choice.take().unwrap();

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('y') | KeyCode::Enter => {
                match choice {
                    Choice::Restore | Choice::Override => {
                        let item = self.list_container.get_slected_item().unwrap();

                        if matches!(choice, Choice::Override) {
                            match fs::exists(&item.restore_location) {
                                Ok(false) => {}
                                Ok(true) => {
                                    self.choice = Some(Choice::Override);
                                    return Ok(false);
                                }
                                Err(e) => {
                                    self.message = Some(Message::error(format!(
                                        "Error checking file existence: {}",
                                        e
                                    )));
                                    return Ok(false);
                                }
                            }
                        }

                        self.message = Some(match restore_item(item) {
                            Ok(()) => Message::info("Item restored successfully".to_string()),
                            Err(e) => Message::error(format!("Error restoring item: {}", e)),
                        });
                        self.choice = None;
                    }
                    Choice::Delete => {
                        let item = self.list_container.get_slected_item().unwrap();

                        self.message = Some(match delete_item(item) {
                            Ok(()) => Message::info("Item deleted successfully".to_string()),
                            Err(e) => Message::error(format!("Error deleting item: {}", e)),
                        });
                        self.choice = None;
                    }
                    Choice::Empty => {
                        self.message = Some(match empty_bin() {
                            Ok(()) => Message::info("Trash emptied successfully".to_string()),
                            Err(e) => Message::error(format!("Error emptying trash: {}", e)),
                        });
                        self.choice = None;
                    }
                };

                self.list_container.refresh(&self.sort_mode);
            }
            KeyCode::Char('n') | KeyCode::Esc | _ => {
                self.choice = None;
            }
        }

        return Ok(false);
    }
}
