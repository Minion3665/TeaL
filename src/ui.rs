use color_eyre::Report;
use eyre::Result;
use std::{
    collections::HashSet,
    io::{stdout, Stdout},
};

use crate::{
    database::{self, Database, FlatTaskTreeElement, Task},
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
    text::{Span, Spans},
    widgets::{self, Block, Borders, ListState, Paragraph},
    Frame, Terminal,
};

#[derive(PartialEq)]
pub enum DisplayingTasksStates {
    Normal,
    Create,
    Search,
}

pub trait StateData {
    fn get_command_palette_text(&self) -> &str;
}

#[derive(PartialEq, Clone)]
pub struct DisplayingTasksData {
    pub selected_task: Option<i64>,
    pub command_palette_text: String,
    pub search_string: Option<String>,
}

#[derive(PartialEq, Clone)]
pub struct DisplayingTaskFullscreenData {
    pub command_palette_text: String,
    pub task_id: i64,
    pub selected_task: Option<i64>,
}

impl StateData for DisplayingTasksData {
    fn get_command_palette_text(&self) -> &str {
        return &self.command_palette_text;
    }
}

impl StateData for DisplayingTaskFullscreenData {
    fn get_command_palette_text(&self) -> &str {
        return &self.command_palette_text;
    }
}

#[derive(PartialEq)]
pub enum States {
    Quitting,
    DisplayingTasks(DisplayingTasksStates, DisplayingTasksData),
    DisplayingTaskFullscreen(DisplayingTaskFullscreenData),
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

/// Draw the command palette and mode line at the bottom of the screen, returning the remaining screen space
fn draw_status_lines(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &States) -> Rect {
    let mut total_size = frame.size();
    if total_size.height < 2 {
        return total_size; // No space for status lines
    }

    let state_data: Option<&dyn StateData> = if let States::DisplayingTasks(_, state_data) = state {
        Some(state_data)
    } else if let States::DisplayingTaskFullscreen(state_data) = state {
        Some(state_data)
    } else {
        None
    };

    if let Some(state_data) = state_data {
        let command_palette = Paragraph::new(state_data.get_command_palette_text().clone());
        let command_palette_area = Rect {
            x: 0,
            y: total_size.height - 1,
            width: total_size.width,
            height: 1,
        };

        frame.render_widget(command_palette, command_palette_area);
        total_size.height -= 1;
    }

    let mode = match state {
        States::DisplayingTasks(inner_state, state_data) => match inner_state {
            DisplayingTasksStates::Normal => {
                if state_data.search_string.is_some() {
                    "List (searching)"
                } else {
                    "List"
                }
            }
            DisplayingTasksStates::Create => "Append",
            DisplayingTasksStates::Search => "Search",
        },
        States::DisplayingTaskFullscreen(_) => "Task",
        States::Quitting => return frame.size(),
    };

    let mode_line_text = vec![Span::styled(
        format!(" {} ", mode),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];

    let mode_line = Paragraph::new(Spans::from(mode_line_text));
    let mode_line_area = Rect {
        x: 0,
        y: total_size.height - 1,
        width: total_size.width,
        height: 1,
    };
    frame.render_widget(mode_line, mode_line_area);
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
    let block = Block::default()
        .title("┤ Your tasks ├")
        .borders(Borders::ALL);

    let mut list_items = Vec::new();

    let filtered_tasks;
    let owned_filtered_tasks;
    if already_filtered {
        filtered_tasks = tasks;
    } else {
        owned_filtered_tasks = filter_tasks(tasks, state_data);
        filtered_tasks = &owned_filtered_tasks;
    }
    if filtered_tasks.is_empty() {
        let warning = widgets::Paragraph::new(Span::styled(
            " There's nothing here, try removing your filters or press `n` to add a new task",
            Style::default().add_modifier(Modifier::ITALIC),
        ))
        .block(block);
        frame.render_widget(warning, remaining_space);
        return;
    }
    for task in filtered_tasks {
        let mut text_parts = vec![Span::from(format!(" {}", task.description))];

        if let Some(parent_id) = task.parent {
            text_parts.push(Span::styled(
                format!(" (child of task {})", parent_id),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))
        }
        list_items.push(widgets::ListItem::new(Spans::from(text_parts)));
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

enum BoxDrawing {
    EndOfList,
    Indented,
    Equal,
    Dedented,
}

pub async fn display_task_fullscreen(
    db: &mut database::Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state_data: DisplayingTaskFullscreenData,
) -> Result<States, Report> {
    let task_tree = db.list_subtasks(state_data.task_id).await?;

    loop {
        terminal.draw(|frame| {
            let remaining_space =
                draw_status_lines(frame, &States::DisplayingTaskFullscreen(state_data.clone()));

            let description = widgets::Paragraph::new(task_tree.description.clone())
                .style(Style::default().add_modifier(Modifier::UNDERLINED));
            frame.render_widget(
                description,
                Rect {
                    x: 1,
                    y: 1,
                    width: task_tree.description.len().try_into().unwrap(),
                    height: 1,
                },
            );

            let task_list_border = widgets::Block::default()
                .borders(Borders::ALL)
                .title("┤ Subtasks ├");

            let flat_task_tree = vec![None]
                .into_iter()
                .chain(task_tree.clone().into_iter().map(Some))
                .chain(vec![None].into_iter())
                .collect::<Vec<Option<FlatTaskTreeElement>>>();

            let mut all_indent_lines: Vec<HashSet<usize>> = vec![HashSet::new()];

            for line in task_tree.clone().into_iter().skip(1) {
                let mut previous_indent_lines = all_indent_lines[all_indent_lines.len() - 1].clone();

                if line.last_under_parent {
                    previous_indent_lines.remove(&line.level);
                } else {
                    previous_indent_lines.insert(line.level);
                }

                all_indent_lines.push(previous_indent_lines);
            }

            let task_list = widgets::List::new(
                flat_task_tree
                    .windows(3)
                    .zip(all_indent_lines)
                    .map(|lines| {
                        let ([line_before, line, line_after], indent_lines) = lines else { unreachable!() };
                        // Windows(3) must *always* return 3 elements here
                        let line = line.as_ref().unwrap();

                        let box_drawing_top = match line_before {
                            None => BoxDrawing::EndOfList,
                            Some(line_before) => {
                                if line_before.level == line.level {
                                    BoxDrawing::Equal
                                } else if line_before.level > line.level {
                                    BoxDrawing::Indented
                                } else {
                                    BoxDrawing::Dedented
                                }
                            }
                        };

                        let box_drawing_bottom = match line_after {
                            None => BoxDrawing::EndOfList,
                            Some(line_after) => {
                                if line_after.level == line.level {
                                    BoxDrawing::Equal
                                } else if line_after.level > line.level {
                                    BoxDrawing::Indented
                                } else {
                                    BoxDrawing::Dedented
                                }
                            }
                        };

                        let box_drawing_character = match (box_drawing_top, box_drawing_bottom, line.last_under_parent) {
                            (BoxDrawing::EndOfList, BoxDrawing::Dedented, _) => unreachable!(),
                            (BoxDrawing::EndOfList, _, _) => "",
                            (
                                _,
                                BoxDrawing::Indented,
                                false
                            ) => "├─",
                            (
                                _,
                                BoxDrawing::EndOfList | BoxDrawing::Indented | BoxDrawing::Dedented,
                                _
                            ) => "└─",
                            (_, BoxDrawing::Equal, _) => "├─",
                        };

                        Spans::from(vec![
                            Span::raw(format!(
                                "{}{}{} ",
                                if line.level == 0 { "" } else { "   " },
                                (0..line.level).skip(1).map(
                                    |level| {
                                        if indent_lines.contains(&level) {
                                            "│    "
                                        } else {
                                            "     "
                                        }
                                    }
                                ).collect::<Vec<&str>>().join(""),
                                box_drawing_character,
                            )),
                            Span::styled(
                                line.task.description.clone(),
                                if line.level == 0 {
                                    Style::default().fg(Color::Cyan)
                                } else {
                                    Style::default()
                                },
                            ),
                        ])
                    })
                    .map(|line| widgets::ListItem::new(line))
                    .collect::<Vec<widgets::ListItem>>(),
            )
            .block(task_list_border);

            frame.render_stateful_widget(
                task_list,
                Rect {
                    x: remaining_space.x,
                    y: remaining_space.y + 5,
                    width: remaining_space.width,
                    height: remaining_space.height - 5,
                },
                &mut ListState::default(),
            );
        })?;

        match read()? {
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => break,
                _ => continue,
            },
            _ => continue,
        }
    }

    Ok(States::DisplayingTasks(
        DisplayingTasksStates::Normal,
        DisplayingTasksData {
            selected_task: Some(state_data.task_id),
            command_palette_text: "".to_owned(),
            search_string: None,
        },
    ))
}

pub async fn display_tasks(
    db: &mut database::Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut state_data: DisplayingTasksData,
) -> Result<States, Error> {
    let mut filtered_tasks = filter_tasks(
        &db.list_tasks(state_data.search_string.is_some()).await?,
        &state_data,
    );

    loop {
        terminal.draw(|frame| {
            let remaining_space = draw_status_lines(
                frame,
                &States::DisplayingTasks(DisplayingTasksStates::Normal, state_data.clone()),
            );
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
                    if filtered_tasks.len() < 1 {
                        continue;
                    }
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
                    if filtered_tasks.len() < 1 {
                        continue;
                    }
                    state_data.selected_task =
                        match task_index_from_id(&filtered_tasks, state_data.selected_task) {
                            None => Some(filtered_tasks[filtered_tasks.len() - 1].id),
                            Some(0) => Some(filtered_tasks[filtered_tasks.len() - 1].id),
                            Some(index) => Some(filtered_tasks[index - 1].id),
                        }
                }
                KeyCode::Char('d') => {
                    let removed_task_index =
                        task_index_from_id(&filtered_tasks, state_data.selected_task);
                    match removed_task_index {
                        None => continue,
                        Some(index) => {
                            db.remove_task(filtered_tasks[index].id).await?;
                            filtered_tasks = filter_tasks(
                                &db.list_tasks(state_data.search_string.is_some()).await?,
                                &state_data,
                            );
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
                KeyCode::Enter => {
                    if let Some(selected_task) = state_data.selected_task {
                        return Ok(States::DisplayingTaskFullscreen(
                            DisplayingTaskFullscreenData {
                                command_palette_text: "Press 'q' to return to the task list"
                                    .to_owned(),
                                task_id: selected_task,
                                selected_task: None,
                            },
                        ));
                    }
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
    let tasks = db.list_tasks(true).await?;

    state_data.command_palette_text = "/".to_owned();

    loop {
        state_data.search_string = Some(
            state_data.command_palette_text[1..state_data.command_palette_text.len()].to_owned(),
        );
        terminal.draw(|frame| {
            let remaining_space = draw_status_lines(
                frame,
                &States::DisplayingTasks(DisplayingTasksStates::Search, state_data.clone()),
            );
            draw_tasks(
                &tasks,
                frame,
                remaining_space,
                state_data.selected_task,
                &state_data,
                false,
            );

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
    let prev_tasks = db.list_tasks(state_data.search_string.is_some()).await?;

    let mut task = String::new();

    state_data.command_palette_text =
        "Press <ENTER> to finish adding the task, or <ESCAPE> to cancel".to_owned();
    loop {
        terminal.draw(|frame| {
            let remaining_space = draw_status_lines(
                frame,
                &States::DisplayingTasks(DisplayingTasksStates::Create, state_data.clone()),
            );
            draw_tasks(
                &prev_tasks,
                frame,
                remaining_space,
                None,
                &state_data,
                false,
            );

            let block = Block::default().title("┤ New task ├").borders(Borders::ALL);

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

    let new_task = db.add_task(&task, None).await?;
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
        States::DisplayingTaskFullscreen(state_data) => {
            Ok(display_task_fullscreen(db, terminal, state_data).await?)
        }
        States::Quitting => panic!("display_state called when the application is already quitting"),
    }
}
