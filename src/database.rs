use platform_dirs::AppDirs;
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
};

use color_eyre::Report;
use crossterm::style::Stylize;
use tabled::Tabled;

use sqlx::{sqlite::SqliteQueryResult, Connection, SqliteConnection};

// Schema is on <app.dbdesigner.net>

pub struct Database {
    connection: SqliteConnection,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: i64, // IMPORTANT: This begins from 1, *not* 0
    pub description: String,
    pub complete: bool,
    pub parent: Option<i64>,
}
// See also: https://www.geeksforgeeks.org/recursive-join-in-sql/

impl From<Task> for i64 {
    fn from(task: Task) -> Self {
        task.id
    }
}

impl Tabled for FlatTaskTreeElement {
    const LENGTH: usize = 3;

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "Number".cyan().bold().to_string().into(),
            "Task".cyan().bold().to_string().into(),
            "Done?".cyan().bold().to_string().into(),
        ]
    }
    fn fields(&self) -> Vec<Cow<str>> {
        vec![
            (self
                .parent_ids
                .clone()
                .into_iter()
                .map(|id| id.to_string() + ".")
                .collect::<Vec<String>>()
                .join("")
                + &self.task.id.to_string())
                .into(),
            self.task.description.to_owned().into(),
            if self.task.complete {
                "Done".green().to_string().into()
            } else {
                "Not done".red().to_string().into()
            },
        ]
    }
}

impl From<&TaskTree> for Task {
    fn from(item: &TaskTree) -> Self {
        Self {
            id: item.id,
            description: item.description.clone(),
            complete: item.complete,
            parent: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskTree {
    pub id: i64,
    pub description: String,
    pub complete: bool,
    pub children: Vec<TaskTree>,
    pub level: usize,
}

struct TaskAndChildMap<'a, 'b> {
    task: &'a Task,
    child_map: &'b HashMap<i64, Vec<Task>>,
    level: usize,
}
impl<'a, 'b> From<TaskAndChildMap<'a, 'b>> for TaskTree {
    fn from(task_and_child_map: TaskAndChildMap<'a, 'b>) -> Self {
        TaskAndChildTrees {
            task: task_and_child_map.task,
            children: task_and_child_map
                .child_map
                .get(&task_and_child_map.task.id)
                .unwrap_or(&Vec::<Task>::default())
                .iter()
                .map(|task| {
                    TaskAndChildMap {
                        task,
                        child_map: task_and_child_map.child_map,
                        level: task_and_child_map.level + 1,
                    }
                    .into()
                })
                .collect(),
            level: task_and_child_map.level,
        }
        .into()
    }
}

struct TaskAndChildTrees<'a> {
    task: &'a Task,
    children: Vec<TaskTree>,
    level: usize,
}
impl<'a> From<TaskAndChildTrees<'a>> for TaskTree {
    fn from(task_and_tree: TaskAndChildTrees<'a>) -> Self {
        TaskTree {
            id: task_and_tree.task.id,
            description: task_and_tree.task.description.clone(),
            complete: task_and_tree.task.complete,
            children: task_and_tree.children,
            level: task_and_tree.level,
        }
    }
}

impl TryFrom<Vec<Task>> for TaskTree {
    type Error = Report;

    fn try_from(tasks: Vec<Task>) -> eyre::Result<Self> {
        let mut root: Option<Task> = None;

        let mut children: HashMap<i64, Vec<Task>> = HashMap::default();

        let mut task_ids = Vec::<i64>::new();

        for task in tasks.clone() {
            task_ids.push(task.id);
            match task.parent {
                None => {
                    if root.is_some() {
                        return Err(Report::msg("Multiple root tasks found"));
                    }
                    root = Some(task);
                }
                Some(parent) => {
                    children.entry(parent).or_insert(Vec::default()).push(task);
                }
            }
        }

        match root {
            None => {
                for task in tasks {
                    if let Some(parent) = task.parent {
                        if !task_ids.contains(&parent) {
                            return Ok(TaskAndChildMap {
                                task: &task,
                                child_map: &children,
                                level: 0,
                            }
                            .into());
                        }
                    }
                }
                Err(Report::msg("No root task found"))
            }
            Some(root) => Ok(TaskAndChildMap {
                task: &root,
                child_map: &children,
                level: 0,
            }
            .into()),
        }
    }
}

impl IntoIterator for TaskTree {
    type Item = FlatTaskTreeElement;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::<Self::Item>::from(self).into_iter()
    }
}

pub struct FlatTaskTreeElement {
    pub level: usize,
    pub last_under_parent: bool,
    pub task: Task,
    pub parent_ids: Vec<i64>,
}

struct TaskTreeElement {
    pub last_under_parent: bool,
    pub task_tree: TaskTree,
    pub parent_ids: Vec<i64>,
}

impl From<TaskTree> for Vec<FlatTaskTreeElement> {
    fn from(tree: TaskTree) -> Self {
        TaskTreeElement {
            last_under_parent: true,
            task_tree: tree,
            parent_ids: vec![],
        }
        .into()
    }
}

pub(super) trait ToFlatTaskTreeElement {
    fn try_to_flat_task_tree_element(self) -> eyre::Result<Vec<FlatTaskTreeElement>>;
}
impl ToFlatTaskTreeElement for Vec<Task> {
    fn try_to_flat_task_tree_element(self) -> eyre::Result<Vec<FlatTaskTreeElement>> {
        let task_tree = TaskTree::try_from(self)?;
        Ok(task_tree.into())
        // We can't just implement TryFrom here as we can only implement traits that are from a
        // different crate on types from this crate. As we don't define Vec in this crate, it isn't
        // possible to implement TryFrom for Vec<Task>
    }
}

impl From<TaskTreeElement> for Vec<FlatTaskTreeElement> {
    fn from(tree: TaskTreeElement) -> Self {
        let mut parent_ids = tree.parent_ids.clone();
        parent_ids.push(tree.task_tree.id);

        let mut result: Vec<FlatTaskTreeElement> = vec![FlatTaskTreeElement {
            level: tree.task_tree.level,
            task: Task::from(&tree.task_tree),
            last_under_parent: tree.last_under_parent,
            parent_ids: tree.parent_ids,
        }];

        let children_length = tree.task_tree.children.len();
        for (index, child) in tree.task_tree.children.into_iter().enumerate() {
            result.append(
                &mut TaskTreeElement {
                    task_tree: child,
                    last_under_parent: index + 1 == children_length,
                    parent_ids: parent_ids.to_owned(),
                }
                .into(),
            );
        }

        result
    }
}

impl Database {
    /// Create a database object with a connection to an SQLite database.
    /// The database file will be created if it doesn't exist, along with any parent directories
    ///
    /// # Arguments
    /// * `path` - The path to the SQLite database file  
    ///            If the file doesn't exist, it will be created along with any missing parent directories  
    ///            If you want to open an in-memory database, you can pass `"sqlite::memory:"` as the path
    ///            If you pass None, a database will be created in the application's data directory. This is platform-dependant but generally it is ~/.local/share/TeaL/TeaL.db in Linux
    ///
    ///
    pub async fn new(path: Option<String>) -> Result<Self, sqlx::Error> {
        let path_str = if path == Some("sqlite::memory:".to_owned()) {
            "sqlite::memory:".to_owned()
        } else {
            let path = if let Some(path) = path {
                PathBuf::from(path)
            } else {
                AppDirs::new(Some("TeaL"), true)
                    .unwrap()
                    .data_dir
                    .join("TeaL.db")
            };

            if !path.try_exists()? {
                fs::create_dir_all(path.parent().unwrap())?;

                let file = File::create(path.as_path())?;
                drop(file); // drop the file so it is closed
            }
            path.to_str()
                .expect("Your paths contain non-unicode characters")
                .to_owned()
        };

        Ok(Self {
            connection: SqliteConnection::connect(&path_str).await?,
        })
    }

    pub async fn setup(&mut self) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query(include_str!("./create.sql"))
            .execute(&mut self.connection)
            .await
    }

    pub async fn add_task(&mut self, task: &str, parent: Option<i64>) -> Result<Task, sqlx::Error> {
        sqlx::query_as!(
            Task,
            "INSERT INTO tasks VALUES (null, ?, false, ?) 
            RETURNING id,
                      description as 'description!',
                      complete, 
                      parent",
            task,
            parent,
        )
        .fetch_one(&mut self.connection)
        .await
    }

    /// Removes a task from the database by ID, and returns how many rows were affected
    /// This will be 0 if the task was not found
    /// This will be 1 if the task was found and removed
    /// If the task had children, they may be cascade deleted which will be reflected in the return value
    pub async fn remove_task(&mut self, task_id: i64) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as!(
            Task,
            "WITH RECURSIVE subtask_tree AS (
                SELECT *
                FROM tasks
                WHERE id = ?
            UNION ALL
                SELECT subtasks.*
                FROM tasks subtasks
            INNER JOIN subtask_tree ON subtask_tree.id = subtasks.parent
        )
        DELETE FROM tasks WHERE id IN (SELECT id FROM subtask_tree)
            RETURNING id as 'id!',
                      description as 'description!',
                      complete as 'complete!', 
                      parent",
            task_id
        )
        .fetch_all(&mut self.connection)
        .await
        // Special thanks to https://stackoverflow.com/a/10381384/12293760 for supplying a way to
        // delete rather than just select the tasks from the recursive subtree
        //
        // Note that we can't just ON DELETE CASCADE as that doesn't let us return the deleted
        // tasks. This is the only way I found to do both in a single query.
    }

    pub async fn set_completion(
        &mut self,
        index: i64,
        completed: bool,
    ) -> Result<Task, sqlx::Error> {
        let task = sqlx::query!(
            "UPDATE tasks SET complete = ? WHERE id = ?
                        RETURNING *",
            completed,
            index
        )
        .fetch_one(&mut self.connection)
        .await?;

        match (task.id, task.description, task.complete, task.parent) {
            (Some(id), Some(description), Some(complete), parent) => Ok(Task {
                id,
                description,
                complete,
                parent,
            }),
            _ => Err(sqlx::Error::RowNotFound),
        }
    }

    pub async fn list_tasks(&mut self, include_children: bool) -> Result<Vec<Task>, sqlx::Error> {
        if include_children {
            sqlx::query_as!(Task, "SELECT * FROM tasks")
                .fetch_all(&mut self.connection)
                .await
        } else {
            sqlx::query_as!(Task, "SELECT * FROM tasks WHERE parent IS NULL")
                // == null is invalid (https://www.sqlitetutorial.net/sqlite-is-null/)
                .fetch_all(&mut self.connection)
                .await
        }
    }

    pub async fn list_subtasks(&mut self, task_id: i64) -> eyre::Result<TaskTree> {
        let tasks = sqlx::query_as!(
            Task,
            "WITH RECURSIVE subtask_tree AS (
                SELECT *
                FROM tasks
                WHERE id = ?
            UNION ALL
                SELECT subtasks.*
                FROM tasks subtasks
            INNER JOIN subtask_tree ON subtask_tree.id = subtasks.parent
        )
        SELECT * FROM subtask_tree",
            task_id
        )
        .fetch_all(&mut self.connection)
        .await?;
        tasks.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_task_test() {
        let mut db = Database::new(Some("sqlite::memory:".to_owned()))
            .await
            .unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);

        db.add_task("A test task", None).await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn remove_task_test() {
        let mut db = Database::new(Some("sqlite::memory:".to_owned()))
            .await
            .unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);

        db.add_task("A test task", None).await.unwrap();

        for task in db.list_tasks(true).await.unwrap() {
            println!("Task got ID {}", task.id);
        }

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 1);

        assert_eq!(db.remove_task(1).await.unwrap().len(), 1);
        assert_eq!(db.remove_task(1).await.unwrap().len(), 0);

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn list_tasks_with_children_test() {
        let mut db = Database::new(Some("sqlite::memory:".to_owned()))
            .await
            .unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);

        let task = db.add_task("A test task", None).await.unwrap();
        db.add_task("A child task", Some(task.id)).await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 2);
        assert_eq!(db.list_tasks(false).await.unwrap().len(), 1);

        db.remove_task(task.id).await.unwrap();

        // Ensure that subtasks are properly cascade-deleted
        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);
    }

    #[tokio::test]
    #[should_panic]
    /// This is a regression test, previously this was the single way to add a cycle to the 'tree'.
    /// As that's now patched we should never be able to get cycles so long as the user doesn't
    /// manually edit the database (e.g. with an SQL client). Even so, they would not be able to
    /// add a task that refers to itself as that is protected by a trigger in the SQLite database
    /// itself.
    async fn disallow_self_referencing_parent_test() {
        let mut db = Database::new(Some("sqlite::memory:".to_owned()))
            .await
            .unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);

        db.add_task("A test task", Some(1)).await.unwrap();
    }
}
