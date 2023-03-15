use eyre::Result;
use std::env;
use ui::{teardown, DisplayingTasksData};

mod cli;
mod database;
mod sorting;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    run().await
}

async fn run() -> Result<()> {
    let mut db = database::Database::new(None).await?;

    db.setup().await?;

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let mut terminal = ui::setup()?;

        let mut state = ui::States::DisplayingTasks(ui::DisplayingTasksStates::Normal, DisplayingTasksData {
        selected_task: None,
        command_palette_text: "Welcome to TeaL! Press 'n' to add a task, 'd' to remove a task or 'h' for more help".to_owned(),
        search_string: None,
    });
        loop {
            state = ui::display_state(state, &mut terminal, &mut db).await?;
            if state == ui::States::Quitting {
                break;
            }
        }

        teardown(&mut terminal)
    } else {
        cli::run(db, args).await
    }
}
