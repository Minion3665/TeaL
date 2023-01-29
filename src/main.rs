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

    db.add_task("This is a task", None).await?;
    let task2 = db.add_task("This is another task", None).await?;
    db.add_task("This is child task", Some(&task2)).await?;
    let task2_child2 = db.add_task("This is another child task", Some(&task2)).await?; 
    db.add_task("This is grandchild task", Some(&task2_child2)).await?;

    let mut terminal = ui::setup()?;

    let mut state = ui::States::DisplayingTasks(ui::DisplayingTasksStates::Normal, DisplayingTasksData {
        selected_task: None,
        command_palette_text: "Welcome to TeaL! Press 'n' to add a task, 'd' to remove a task or 'h' for more help".to_owned(),
        search_string: None,
        mode: "List".to_owned(),
    });
    loop {
        state = ui::display_state(state, &mut terminal, &mut db).await?;
        if state == ui::States::Quitting {
            break;
        }
    }

    teardown(&mut terminal)
}
