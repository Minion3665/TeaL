use eyre::Result;

use crate::{
    database::{self, Database, FlatTaskTreeElement, ToFlatTaskTreeElement},
    sorting,
};

use self::argument_parser::parse_ids;
use self::rendering::render_table;

mod argument_parser;
mod rendering;

pub async fn run(mut db: Database, args: Vec<String>) -> Result<()> {
    let args = argument_parser::parse_args(args.iter().map(String::as_str).collect())?;
    match args.subcommand[..] {
        ["list" | "ls"] => {
            let search = args.args.get("search");

            let mut tasks = db.list_tasks(false).await?;

            if let Some(term) = search {
                println!("Searching for '{}'", &term.join(" "));
                tasks = sorting::search(&term.join(" "), tasks);
            }

            let mut task_tree_elements: Vec<database::FlatTaskTreeElement> = vec![];

            for task in tasks {
                task_tree_elements.append(&mut db.list_subtasks(task.id).await?.into());
            }

            let table_string = render_table(task_tree_elements, args.flags.get("raw").is_some());

            println!("{}", table_string);
        }
        ["add" | "create"] => {
            let task_name = args.args.get("name").unwrap_or(&Vec::default()).join(" ");
            let parent = args.args.get("parent");

            let parent_id = if parent.is_some() {
                let all_parent_ids = match parse_ids(parent) {
                    Ok(all_parent_ids) => all_parent_ids,
                    Err(error) => {
                        println!("{}, please run '{} help add' for help", error, args.command);
                        return Ok(());
                    }
                };

                if all_parent_ids.len() != 1 {
                    println!(
                        "Only one parent can be specified, please run '{} help add' for help",
                        args.command
                    );
                }

                Some(all_parent_ids[0])
            } else {
                None
            };

            if task_name.is_empty() {
                println!(
                    "Task name cannot be empty, please run '{} help add' for help",
                    args.command
                );
                return Ok(());
            }

            let task = db.add_task(task_name.as_str(), parent_id).await;

            match task {
                Ok(task) => {
                    println!(
                        "{}",
                        render_table(
                            vec![FlatTaskTreeElement {
                                level: 0,
                                last_under_parent: false,
                                task,
                                parent_ids: match parent_id {
                                    Some(parent_id) => vec![parent_id],
                                    None => vec![],
                                }
                            }],
                            args.flags.get("raw").is_some()
                        )
                    )
                }
                Err(error) => {
                    if let sqlx::Error::Database(error) = error {
                        match error.message() {
                            "FOREIGN KEY constraint failed" => {
                                println!("The task you set as a parent task doesn't exist")
                            }
                            _ => return Err(error.into()),
                        }
                        return Ok(());
                    }

                    return Err(error.into());
                }
            };
        }
        ["remove" | "del"] => {
            let task_ids = args.args.get("id").or_else(|| args.args.get("i"));
            let parsed_task_ids = parse_ids(task_ids);

            match parsed_task_ids {
                Ok(parsed_task_ids) => {
                    for id in parsed_task_ids {
                        let deleted_tasks = db.remove_task(id).await?;
                        let number_of_deleted_tasks = deleted_tasks.len();
                        let flat_task_tree = match deleted_tasks.try_to_flat_task_tree_element() {
                            Ok(flat_task_tree) => flat_task_tree,
                            Err(_) => {
                                println!(
                                    "Task {} doesn't exist, please run '{} list' to view all of your tasks", 
                                    id,
                                    args.command
                                );
                                return Ok(());
                            }
                        };
                        let table = render_table(flat_task_tree, args.flags.get("raw").is_some());
                        println!("Deleted {} tasks:", number_of_deleted_tasks);
                        println!("{}", table);
                    }
                }
                Err(error) => println!("{}, please run {} for help", error, args.command),
            }
        }
        _ => {
            println!(
                "Command '{}' doesn't exist, please run '{} help' for help",
                args.subcommand.join(" "),
                args.command
            )
        }
    }
    Ok(())
}
