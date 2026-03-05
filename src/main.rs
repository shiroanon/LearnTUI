pub mod app;
pub mod content;
pub mod models;
pub mod store;
pub mod tui;
pub mod ui;
pub mod markdown;

use app::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    let mut terminal = tui::init()?;

    // create app and run it
    let mut app = App::new()?;
    let res = tui::run_app(&mut terminal, &mut app);

    // restore terminal
    tui::restore()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
