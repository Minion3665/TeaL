use eyre::Result;
use std::io::{stdout, Stdout};

use crate::{
    database::{self, Database, Task},
    sorting::search,
};
use crossterm::{
    self,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use sqlx::Error;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{self, Block, Borders, ListState, Paragraph},
    Frame, Terminal,
};

#[derive(PartialEq)]
pub enum DisplayingTasksStates {
    Normal,
    Create,
    Search,
}

#[derive(PartialEq)]
pub struct DisplayingTasksData {
    pub selected_task: Option<i64>,
    pub command_palette_text: String,
    pub search_string: Option<String>,
}

#[derive(PartialEq)]
pub enum States {
    Quitting,
    DisplayingTasks(DisplayingTasksStates, DisplayingTasksData),
}

pub fn setup() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    execute!(&mut stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

/// Provided a list of tasks and a task ID, do a linear search to find a task which has the correct
/// ID. As IDs should be unique this is guaranteed to find all tasks with a given ID.
fn task_index_from_id(tasks: &Vec<Task>, id: Option<i64>) -> Option<usize> {
    for (index, task) in tasks.into_iter().enumerate() {
        if let Some(task_id) = id {
            if task.id == task_id {
                return Some(index);
            }
        }
    }

    None
}

/// Draw the command palette at the bottom of the screen, returning the remaining screen space
fn draw_command_palette(
    frame: &mut Frame<CrosstermBackend<Stdout>>,
    state_data: &DisplayingTasksData,
) -> Rect {
    let mut total_size = frame.size();

    let widget = Paragraph::new(state_data.command_palette_text.clone());
    let area = Rect {
        x: 0,
        y: total_size.height - 1,
        width: total_size.width,
        height: 1,
    };
    frame.render_widget(widget, area);

    total_size.height -= 1;
    total_size
}

fn filter_tasks(tasks: &Vec<Task>, state_data: &DisplayingTasksData) -> Vec<Task> {
    let mut filtered_tasks = tasks.to_owned();

    if let Some(ref query) = state_data.search_string {
        filtered_tasks = search(&query, filtered_tasks);
    }

    filtered_tasks
}

fn draw_tasks(
    tasks: &Vec<Task>,
    frame: &mut Frame<CrosstermBackend<Stdout>>,
    remaining_space: Rect,
    selected: Option<i64>,
    state_data: &DisplayingTasksData,
    already_filtered: bool,
) {
    let block = Block::default().title("Your tasks").borders(Borders::ALL);
    let mut list_items = Vec::new();

    let filtered_tasks;
    let owned_filtered_tasks;
    if already_filtered {
        filtered_tasks = tasks;
    } else {
        owned_filtered_tasks = filter_tasks(tasks, state_data);
        filtered_tasks = &owned_filtered_tasks;
    }
    for task in filtered_tasks {
        list_items.push(widgets::ListItem::new(format!(
            "[{}]: {}",
            task.id, task.description
        )))
    }

    let list = widgets::List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(block);

    let mut state = ListState::default();
    if let Some(task_id) = selected {
        state.select(task_index_from_id(filtered_tasks, Some(task_id)));
    }
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
    mut state_data: DisplayingTasksData,
) -> Result<States, Error> {
    let mut filtered_tasks = filter_tasks(&db.list_tasks().await?, &state_data);

    loop {
        terminal.draw(|frame| {
            let remaining_space = draw_command_palette(frame, &state_data);
            draw_tasks(
                &filtered_tasks,
                frame,
                remaining_space,
                state_data.selected_task,
                &state_data,
                true,
            )
        })?;
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('n') => {
                    return Ok(States::DisplayingTasks(
                        DisplayingTasksStates::Create,
                        state_data,
                    ))
                }
                KeyCode::Char('q') => return Ok(States::Quitting),
                KeyCode::Char('j') | KeyCode::Down => {
                    if filtered_tasks.len() < 1 { continue; }
                    state_data.selected_task =
                        match task_index_from_id(&filtered_tasks, state_data.selected_task) {
                            None => Some(filtered_tasks[0].id),
                            Some(index) => {
                                if index + 1 >= filtered_tasks.len() {
                                    Some(filtered_tasks[0].id)
                                } else {
                                    Some(filtered_tasks[index + 1].id)
                                }
                            }
                        }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if filtered_tasks.len() < 1 { continue; }
                    state_data.selected_task =
                        match task_index_from_id(&filtered_tasks, state_data.selected_task) {
                            None => Some(filtered_tasks[filtered_tasks.len() - 1].id),
                            Some(0) => Some(filtered_tasks[filtered_tasks.len() - 1].id),
                            Some(index) => Some(filtered_tasks[index - 1].id),
                        }
                }
                KeyCode::Char('d') => {
                    let removed_task_index = task_index_from_id(&filtered_tasks, state_data.selected_task);
                    match removed_task_index {
                        None => continue,
                        Some(index) => {
                            db.remove_task(filtered_tasks[index].id).await?;
                            filtered_tasks = filter_tasks(&db.list_tasks().await?, &state_data);
                            state_data.selected_task = if filtered_tasks.len() == 0 {
                                None
                            } else if index == filtered_tasks.len() {
                                Some(filtered_tasks[filtered_tasks.len() - 1].id)
                            } else {
                                Some(filtered_tasks[index].id)
                            };
                        }
                    };
                }
                KeyCode::Char('/') => {
                    return Ok(States::DisplayingTasks(
                        DisplayingTasksStates::Search,
                        state_data,
                    ));
                }
                _ => continue,
            },
            _ => continue,
        }
    }
}

pub async fn search_tasks(
    db: &mut database::Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut state_data: DisplayingTasksData,
) -> Result<States> {
    let tasks = db.list_tasks().await?;

    state_data.command_palette_text = "/".to_owned();

    loop {
        state_data.search_string = Some(
            state_data.command_palette_text[1..state_data.command_palette_text.len()].to_owned(),
        );
        terminal.draw(|frame| {
            let remaining_space = draw_command_palette(frame, &state_data);
            draw_tasks(&tasks, frame, remaining_space, state_data.selected_task, &state_data, false);

            if let Ok(cursor_x) = state_data.command_palette_text.len().try_into() {
                frame.set_cursor(cursor_x, frame.size().height - 1);
            }
        })?;

        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Enter => break,
                KeyCode::Char(char) => state_data.command_palette_text.push(char),
                KeyCode::Esc => {
                    state_data.command_palette_text = "".to_owned();
                    state_data.search_string = None;
                    break;
                }
                KeyCode::Backspace => {
                    if state_data.command_palette_text.len() > 1 {
                        state_data.command_palette_text.pop();
                    }
                    continue;
                }
                _ => continue,
            },
            _ => continue,
        }
    }

    if let Some(search_string) = &state_data.search_string {
        if search_string.len() == 0 {
            state_data.search_string = None;
            state_data.command_palette_text = "".to_owned();
        }
    }

    Ok(States::DisplayingTasks(
        DisplayingTasksStates::Normal,
        state_data,
    ))
}

pub async fn ask_for_tasks(
    db: &mut database::Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut state_data: DisplayingTasksData,
) -> Result<States, Error> {
    let prev_tasks = db.list_tasks().await?;

    let mut task = String::new();

    state_data.command_palette_text =
        "Press <ENTER> to finish adding the task, or <ESCAPE> to cancel".to_owned();
    loop {
        terminal.draw(|frame| {
            let remaining_space = draw_command_palette(frame, &state_data);
            draw_tasks(&prev_tasks, frame, remaining_space, None, &state_data, false);

            let block = Block::default().title("New Task").borders(Borders::ALL);

            let width = 42;
            let height = 3;

            let x = (remaining_space.width - width) / 2;
            let y = (remaining_space.height - height) / 2;

            let size = Rect {
                x,
                y,
                width,
                height,
            };

            let inner_area = block.inner(size);
            let task_length = task.len();
            let displayed_text = if task_length + 1 > inner_area.width.into() {
                "...".to_owned()
                    + (task
                        .clone()
                        .split_at(task_length - <u16 as Into<usize>>::into(inner_area.width - 4))
                        .1)
            } else {
                task.clone()
            };
            if let Ok(text_length) = <usize as TryInto<u16>>::try_into(displayed_text.len()) {
                frame.set_cursor(x + text_length + 1, y + 1);
            }

            let text_widget = widgets::Paragraph::new(displayed_text);
            frame.render_widget(block, size);
            frame.render_widget(text_widget, inner_area);

        })?;

        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Enter => {
                    state_data.command_palette_text = "".to_owned();
                    break;
                }
                KeyCode::Char(char) => task.push(char),
                KeyCode::Esc => {
                    state_data.command_palette_text = "".to_owned();
                    return Ok(States::DisplayingTasks(
                        DisplayingTasksStates::Normal,
                        state_data,
                    ));
                }
                KeyCode::Backspace => {
                    task.pop();
                }
                _ => continue,
            },
            _ => continue,
        }
    }

    let new_task = db.add_task(&task).await?;
    state_data.selected_task = Some(new_task.id);

    Ok(States::DisplayingTasks(
        DisplayingTasksStates::Normal,
        state_data,
    ))
}

pub async fn display_state(
    state: States,
    terminal: &mut tui::Terminal<CrosstermBackend<Stdout>>,
    db: &mut Database,
) -> Result<States> {
    match state {
        States::DisplayingTasks(DisplayingTasksStates::Normal, state_data) => {
            Ok(display_tasks(db, terminal, state_data).await?)
        }
        States::DisplayingTasks(DisplayingTasksStates::Create, state_data) => {
            Ok(ask_for_tasks(db, terminal, state_data).await?)
        }
        States::DisplayingTasks(DisplayingTasksStates::Search, state_data) => {
            Ok(search_tasks(db, terminal, state_data).await?)
        }
        States::Quitting => panic!("display_state called when the application is already quitting"),
    }
}
