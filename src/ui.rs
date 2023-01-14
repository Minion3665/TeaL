use eyre::Result;
use std::io::{stdout, Stdout};

use crate::database::{self, Database, Task};
use crossterm::{
    self,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use sqlx::Error;
use tui::{
    backend::CrosstermBackend,
    widgets::{self, Block, Borders, ListState},
    Frame, Terminal, layout::Rect, style::{Style, Color, Modifier},
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

/// Draw the command palette at the bottom of the screen, returning the remaining screen space
fn draw_command_palette(frame: &mut Frame<CrosstermBackend<Stdout>>) -> Rect {
    //frame.render_widget(widget, area)

    let mut total_size = frame.size();
    total_size.height -= 1;
    total_size
}

fn draw_tasks(tasks: &Vec<Task>, frame: &mut Frame<CrosstermBackend<Stdout>>, remaining_space: Rect, selected: Option<usize>) {
    let block = Block::default().title("Your tasks").borders(Borders::ALL);
    let mut list_items = Vec::new();

    for task in tasks {
        list_items.push(widgets::ListItem::new(format!(
            "[{}]: {}",
            task.id, task.description
        )))
    }

    let list = widgets::List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(block);

    let mut state = ListState::default();
    state.select(selected);
    frame.render_stateful_widget(list, remaining_space, &mut state)
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
    let mut tasks = db.list_tasks().await?;

    let mut selected: Option<usize> = None;
    loop {
        terminal.draw(|frame| {
            let remaining_space = draw_command_palette(frame);
            draw_tasks(&tasks, frame, remaining_space, selected)
        })?;
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('n') => {
                    return Ok(States::DisplayingTasks(DisplayingTasksStates::Create))
                }
                KeyCode::Char('q') => return Ok(States::Quitting),
                KeyCode::Char('j') | KeyCode::Down => selected = match selected {
                    None => Some(0),
                    Some(index) => if index + 1 >= tasks.len() { Some(0) } else { Some(index + 1) }
                },                
                KeyCode::Char('k') | KeyCode::Up => selected = match selected {
                    None => Some(tasks.len() - 1),
                    Some(0) => Some(tasks.len() - 1),
                    Some(index) => Some(index - 1)
                },
                KeyCode::Char('d') => {
                    match selected {
                        None => continue,
                        Some(index) => db.remove_task(tasks[index].id).await?
                    };
                    tasks = db.list_tasks().await?;
                    selected = if tasks.len() == 0 {
                        None
                    } else if selected == Some(tasks.len()) {
                        Some(tasks.len() - 1)
                    } else {
                        selected
                    };
                },
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
    let _task = String::new();
    let prev_tasks = db.list_tasks().await?;

    let mut task = String::new();

    loop {
        terminal.draw(|frame| {
            let remaining_space = draw_command_palette(frame);
            draw_tasks(&prev_tasks, frame, remaining_space, None);


            let block = Block::default().title("New Task").borders(Borders::ALL);

            let width = 42;
            let height = 3;

            let size = Rect {
                x: (remaining_space.width - width) / 2,
                y: (remaining_space.height - height) / 2,
                width,
                height
            };

            let inner_area = block.inner(size);
            let task_length = task.len();
            let displayed_text = if task_length > inner_area.width.into() {
                "...".to_owned() + (task.clone().split_at(task_length - <u16 as Into<usize>>::into(inner_area.width - 3)).1)
            } else { task.clone() };
            let text_widget = widgets::Paragraph::new(displayed_text);
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
