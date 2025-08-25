use std::{fs, io};

use crossterm::event::{Event, KeyCode, KeyEvent};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::App,
    io::{delete_item, empty_bin, restore_item},
    ui::Message,
    utils::{Choice, Mode, SortMode},
};

impl App {
    pub fn handle_key_press(&mut self, key: KeyEvent, event: &Event) -> Result<bool, io::Error> {
        // Clear message on any key press
        if self.message.is_some() {
            self.message = None;
        }

        if self.choice_popup.is_some() {
            return self.handle_choice_action(key);
        }

        match self.mode {
            Mode::ListView => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
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
                        self.choice_popup = Some(Choice::Restore);
                    }
                }
                KeyCode::Char('d') => {
                    if self.list_container.get_slected_item().is_some() {
                        self.choice_popup = Some(Choice::Delete);
                    }
                }
                KeyCode::Char('e') => {
                    self.choice_popup = Some(Choice::Empty);
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
                    KeyCode::Char('N') => SortMode::NameAsc,
                    KeyCode::Char('n') => SortMode::NameDesc,
                    KeyCode::Char('d') => SortMode::DateAsc,
                    KeyCode::Char('D') => SortMode::DateDesc,
                    _ => SortMode::DateAsc,
                };
                self.list_container.sort(&self.sort_mode);
            }
        }

        Ok(false)
    }

    pub fn handle_choice_action(&mut self, key: KeyEvent) -> Result<bool, std::io::Error> {
        let choice = self.choice_popup.take().unwrap();

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
                                    self.choice_popup = Some(Choice::Override);
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
                        self.choice_popup = None;
                    }
                    Choice::Delete => {
                        let item = self.list_container.get_slected_item().unwrap();

                        self.message = Some(match delete_item(item) {
                            Ok(()) => Message::info("Item deleted successfully".to_string()),
                            Err(e) => Message::error(format!("Error deleting item: {}", e)),
                        });
                        self.choice_popup = None;
                    }
                    Choice::Empty => {
                        self.message = Some(match empty_bin() {
                            Ok(()) => Message::info("Trash emptied successfully".to_string()),
                            Err(e) => Message::error(format!("Error emptying trash: {}", e)),
                        });
                        self.choice_popup = None;
                    }
                };

                self.list_container.refresh(&self.sort_mode);
            }
            KeyCode::Char('n') | KeyCode::Esc | _ => {
                self.choice_popup = None;
            }
        }

        return Ok(false);
    }
}
