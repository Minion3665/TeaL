use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::EnterAlternateScreen,
};
use eyre::Result;
use ui::teardown;
use std::io::stdout;
use tokio;
use tui::{backend::CrosstermBackend, Terminal};
mod database;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    run().await
}

async fn run() -> Result<()> {
    let mut db = database::Database::new().await?;

    db.setup().await?;

    db.add_task("This is a task").await?;
    db.add_task("This is another task").await?;

    let mut terminal = ui::setup()?;

    let mut state = ui::States::DisplayingTasks(ui::DisplayingTasksStates::Normal);
    loop {
        state = ui::display_state(
            state,
            &mut terminal,
            &mut db,
        ).await?;
        if state == ui::States::Quitting {
            break;
        }
    }

    teardown(&mut terminal)
}
