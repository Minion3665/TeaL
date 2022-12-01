use std::io::{self, stdout};

use crate::database;
use crossterm::{self, event::{EnableMouseCapture, Event, KeyEvent, KeyCode, read, DisableMouseCapture}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, self}, ExecutableCommand};
use sqlx::Error;
use tui::{
    backend::CrosstermBackend,
    symbols::block,
    widgets::{self, Block, Borders},
    Terminal,
};

#[derive(PartialEq)]
pub enum DisplayingTasksStates {
    Normal,
    Create,
}

#[derive(PartialEq)]
pub enum States {
    DisplayingTasks(DisplayingTasksStates),
}

pub async fn display_tasks(db: &mut database::Database) -> Result<(), Error> {
    let tasks = db.list_tasks().await?;

    let mut list_items = Vec::new();

    for task in tasks {
        list_items.push(widgets::ListItem::new(format!(
            "[{}]: {}",
            task.id, task.description
        )))
    }

    let list = widgets::List::new(list_items);

    crossterm::terminal::enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let mut backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Your tasks").borders(Borders::ALL);

        let inner_area = block.inner(size);

        f.render_widget(block, size);
        f.render_widget(list, inner_area);
    })?;

    loop {
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => break,
                _ => continue
            }
            _ => continue
        }
    }

    execute!(stdout(), LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

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
