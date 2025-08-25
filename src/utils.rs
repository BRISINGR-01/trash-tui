use fuse_rust::SearchResult;

use ratatui::{DefaultTerminal, widgets::Row};

use crate::{
    trash_entry::TrashEntry,
    ui::{layout, make_row_widget},
};

pub enum Mode {
    ListView,
    Filtering,
    Sorting,
}

pub enum SortMode {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
}
pub enum Choice {
    Restore,
    Delete,
    Empty,
    Override,
}

pub fn compute_list_size(terminal: &mut DefaultTerminal) -> usize {
    let mut s: usize = 0;
    let _ = terminal.draw(|frame| {
        let [_, first, _] = layout(&Mode::ListView).areas(frame.area());
        s = (
            first.as_size().height - 2
            // border
        ) as usize;
    });

    s
}

pub fn with_search<'a>(items: &'a Vec<TrashEntry>, search: &str) -> Vec<Row<'a>> {
    if search.is_empty() {
        return items
            .iter()
            .map(|item| make_row_widget(&item, None))
            .collect();
    }

    let fuse = fuse_rust::Fuse {
        is_case_sensitive: false,
        ..Default::default()
    };

    let mut results = fuse
        .search_text_in_iterable(search, items.iter().map(|item| item.display_name.clone()))
        .into_iter()
        .filter(|result| result.score < 1f64)
        .collect::<Vec<SearchResult>>();

    results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

    return results
        .into_iter()
        .map(|result| make_row_widget(&items[result.index], Some(result.ranges)))
        .collect::<Vec<Row>>();
}
