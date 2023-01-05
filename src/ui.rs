use eyre::Result;
use std::io::{self, stdout, Stdout};

use crate::database::{self, Database, Task};
use crossterm::{
    self,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use sqlx::Error;
use tui::{
    backend::CrosstermBackend,
    symbols::block,
    widgets::{self, Block, Borders},
    Frame, Terminal, layout::Rect,
};

#[derive(PartialEq)]
pub enum DisplayingTasksStates {
    Normal,
    Create,
}

#[derive(PartialEq)]
pub enum States {
    Quitting,
    DisplayingTasks(DisplayingTasksStates),
}

pub fn setup() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    execute!(&mut stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn draw_tasks(tasks: &Vec<Task>, frame: &mut Frame<CrosstermBackend<Stdout>>) {
    let mut list_items = Vec::new();

    for task in tasks {
        list_items.push(widgets::ListItem::new(format!(
            "[{}]: {}",
            task.id, task.description
        )))
    }

    let list = widgets::List::new(list_items);

    let size = frame.size();
    let block = Block::default().title("Your tasks").borders(Borders::ALL);

    let inner_area = block.inner(size);

    frame.render_widget(block, size);
    frame.render_widget(list, inner_area);
}

pub fn teardown(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub async fn display_tasks(
    db: &mut database::Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<States, Error> {
    let tasks = db.list_tasks().await?;

    terminal.draw(|frame| draw_tasks(&tasks, frame))?;

    loop {
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('n') => {
                    return Ok(States::DisplayingTasks(DisplayingTasksStates::Create))
                }
                KeyCode::Char('q') => return Ok(States::Quitting),
                _ => continue,
            },
            _ => continue,
        }
    }
}

pub async fn ask_for_tasks(
    db: &mut database::Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<States, Error> {
    let mut task = String::new();
    let mut prev_tasks = db.list_tasks().await?;

    let mut task = String::new();

    loop {
        terminal.draw(|frame| {
            draw_tasks(&prev_tasks, frame);


            let block = Block::default().title("New Task").borders(Borders::ALL);
            let text_widget = widgets::Paragraph::new(task.clone());

            let width = 22;
            let height = 3;

            let terminal_size = frame.size();

            let size = Rect {
                x: (terminal_size.width - width) / 2,
                y: (terminal_size.height - height) / 2,
                width,
                height
            };

            let inner_area = block.inner(size);
            frame.render_widget(block, size);
            frame.render_widget(text_widget, inner_area);
        })?;

        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Enter => break,
                KeyCode::Char(char) => task.push(char),
                KeyCode::Esc => return Ok(States::DisplayingTasks(DisplayingTasksStates::Normal)),
                KeyCode::Backspace => {
                    task.pop();
                    ()
                }
                _ => continue,
            },
            _ => continue,
        }
    }

    db.add_task(&task).await?;

    Ok(States::DisplayingTasks(DisplayingTasksStates::Normal))
}

pub async fn display_state(
    state: States,
    terminal: &mut tui::Terminal<CrosstermBackend<Stdout>>,
    db: &mut Database,
) -> Result<States> {
    match state {
        States::DisplayingTasks(DisplayingTasksStates::Normal) => {
            Ok(display_tasks(db, terminal).await?)
        }
        States::DisplayingTasks(DisplayingTasksStates::Create) => {
            Ok(ask_for_tasks(db, terminal).await?)
        }
        States::Quitting => panic!("display_state called when the application is already quitting"),
    }
}
