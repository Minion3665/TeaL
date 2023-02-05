use eyre::Result;
use ui::{teardown, DisplayingTasksData};

mod database;
mod sorting;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    run().await
}

async fn run() -> Result<()> {
    let mut db = database::Database::new().await?;

    db.setup().await?;

    let level1 = db.add_task("Level 1", None).await?;
    let level2 = db.add_task("Level 2", Some(&level1)).await?;
    let level3 = db.add_task("Level 3", Some(&level2)).await?;
    let level4 = db.add_task("Level 4", Some(&level3)).await?;
    let level5 = db.add_task("Level 5", Some(&level4)).await?;
    let level6 = db.add_task("Level 6", Some(&level5)).await?;
    let level7 = db.add_task("Level 7", Some(&level6)).await?;
    let level8 = db.add_task("Level 8", Some(&level7)).await?;
    let level9 = db.add_task("Level 9", Some(&level8)).await?;
    let level10 = db.add_task("Level 10", Some(&level9)).await?;
    let level11 = db.add_task("Level 11", Some(&level10)).await?;
    let level12 = db.add_task("Level 12", Some(&level11)).await?;
    db.add_task("Level 5.2", Some(&level4)).await?;
    db.add_task("Level 5.3", Some(&level4)).await?;
    db.add_task("Level 4.2", Some(&level3)).await?;
    db.add_task("Level 4.3", Some(&level3)).await?;

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
}
