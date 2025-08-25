use crate::{
    io::{get_trash_dirs, list_files_from_dir},
    trash_entry::TrashEntry,
    utils::SortMode,
};
use ratatui::widgets::TableState;

pub struct ListContainer {
    pub items: Vec<TrashEntry>,
    list_size: usize,
    pub state: TableState,
}

impl ListContainer {
    pub fn new(size: usize, sort_mode: &SortMode) -> Self {
        let mut list = ListContainer {
            state: TableState::default(),
            items: Vec::new(),
            list_size: size,
        };

        list.refresh(sort_mode);

        list
    }

    pub fn refresh(&mut self, sort_mode: &SortMode) {
        let (_, _, info_dir) = get_trash_dirs();
        let files = list_files_from_dir(&info_dir);

        if files.is_none() {
            panic!("Error listing files from {}", info_dir.display());
        }

        self.items = files
            .unwrap()
            .iter()
            .filter_map(|file| TrashEntry::from_trash_info(file).ok())
            .collect::<Vec<TrashEntry>>();

        self.sort(sort_mode);

        if self.state.selected().is_none() {
            self.state.select_first();
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        match self.state.selected() {
            Some(val) if val == self.items.len() - 1 => self.state.select_first(),
            _ => self.state.select_next(),
        }
    }

    pub fn prev(&mut self) {
        match self.state.selected() {
            Some(0) => self.state.select_last(),
            _ => self.state.select_previous(),
        }
    }

    pub fn scroll_next(&mut self) {
        match self.state.selected() {
            Some(val) => self.state.select(Some(val.saturating_add(self.list_size))),
            _ => self.state.select_first(),
        }
    }

    pub fn scroll_prev(&mut self) {
        match self.state.selected() {
            Some(val) => self.state.select(Some(val.saturating_sub(self.list_size))),
            _ => self.state.select_first(),
        }
    }

    pub fn get_slected_item(&self) -> Option<&TrashEntry> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
    }

    pub fn sort(&mut self, sort_mode: &SortMode) {
        match sort_mode {
            SortMode::NameAsc => self
                .items
                .sort_by(|a, b| b.display_name.cmp(&a.display_name)),
            SortMode::NameDesc => self
                .items
                .sort_by(|a, b| a.display_name.cmp(&b.display_name)),
            SortMode::DateAsc => self.items.sort_by(|a, b| b.date.cmp(&a.date)),
            SortMode::DateDesc => self.items.sort_by(|a, b| a.date.cmp(&b.date)),
        }
    }

    pub fn resize(&mut self, size: usize) {
        self.list_size = size;
    }
}
