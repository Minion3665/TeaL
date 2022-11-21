use std::io;

use sqlx::Error;
use crate::database;

#[derive(PartialEq)]
pub enum DisplayingTasksStates {
    Normal,
    Create,
}

#[derive(PartialEq)]
pub enum States {
    DisplayingTasks(DisplayingTasksStates)
}

pub async fn display_tasks(db: &mut database::Database) -> Result<(), Error> {
    let tasks = db.list_tasks().await?;

    for task in tasks {
        println!("[{}]: {}", task.id, task.description);
    };

    Ok(())
}

pub async fn ask_for_tasks(db: &mut database::Database) -> Result<(), Error> {
    let mut task = String::new();

    io::stdin()
        .read_line(&mut task)
        .expect("Failed to read line");

    db.add_task(&task).await?;

    Ok(())
}

pub async fn display_state(state: States, db: &mut database::Database) -> Result<(), Error> {
    match state {
        States::DisplayingTasks(DisplayingTasksStates::Normal) => display_tasks(db).await,
        States::DisplayingTasks(DisplayingTasksStates::Create) => ask_for_tasks(db).await,
    }
}
