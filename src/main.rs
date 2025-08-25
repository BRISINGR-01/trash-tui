mod app;
mod events;
mod io;
mod list;
mod trash_entry;
mod ui;
mod utils;

use crate::app::App;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::new(&mut terminal).run(&mut terminal);
    ratatui::restore();
    result
}
