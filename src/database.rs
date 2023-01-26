use std::collections::HashMap;

use color_eyre::Report;
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

#[derive(Debug, Clone)]
pub struct TaskTree {
    pub id: i64,
    pub description: String,
    pub complete: bool,
    pub children: Vec<TaskTree>,
}

impl TaskTree {
    pub fn from_tasks(tasks: Vec<Task>) -> eyre::Result<Self> {
        let mut root: Option<Task> = None;

        let mut children: HashMap<i64, Vec<Task>> = HashMap::default();

        for task in tasks {
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
            None => Err(Report::msg("No root task found")),
            Some(root) => Ok(Self::from_task_and_children(&root, &children)),
        }
    }

    fn from_task_and_children(task: &Task, children: &HashMap<i64, Vec<Task>>) -> Self {
        Self::from_task_and_task_trees(
            task,
            children
                .get(&task.id)
                .unwrap()
                .into_iter()
                .map(|task| TaskTree::from_task_and_children(task, &children))
                .collect(),
        )
    }

    fn from_task_and_task_trees(task: &Task, children: Vec<TaskTree>) -> Self {
        TaskTree {
            id: task.id,
            description: task.description.clone(),
            complete: task.complete,
            children,
        }
    }
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        Ok(Self {
            connection: SqliteConnection::connect("sqlite::memory:").await?,
        })
    }

    pub async fn setup(&mut self) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query(include_str!("./create.sql"))
            .execute(&mut self.connection)
            .await
    }

    pub async fn add_task(
        &mut self,
        task: &str,
        parent: Option<&Task>,
    ) -> Result<Task, sqlx::Error> {
        let parent_id = match parent {
            None => None,
            Some(parent) => Some(parent.id),
        };
        sqlx::query_as!(
            Task,
            "INSERT INTO tasks VALUES (null, ?, false, ?) 
            RETURNING id,
                      description as 'description!',
                      complete, 
                      parent",
            task,
            parent_id,
        )
        .fetch_one(&mut self.connection)
        .await
    }

    pub async fn remove_task(&mut self, index: i64) -> Result<u64, sqlx::Error> {
        Ok(sqlx::query!("DELETE FROM tasks WHERE id = ?", index)
            .execute(&mut self.connection)
            .await?
            .rows_affected())
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
        TaskTree::from_tasks(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_task_test() {
        let mut db = Database::new().await.unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);

        db.add_task("A test task", None).await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn remove_task_test() {
        let mut db = Database::new().await.unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);

        db.add_task("A test task", None).await.unwrap();

        for task in db.list_tasks(true).await.unwrap() {
            println!("Task got ID {}", task.id);
        }

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 1);

        assert_eq!(db.remove_task(1).await.unwrap(), 1);
        db.remove_task(0).await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn list_tasks_with_children_test() {
        let mut db = Database::new().await.unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);

        let task = db.add_task("A test task", None).await.unwrap();
        db.add_task("A child task", Some(&task)).await.unwrap();

        assert_eq!(db.list_tasks(true).await.unwrap().len(), 2);
        assert_eq!(db.list_tasks(false).await.unwrap().len(), 1);

        db.remove_task(task.id).await.unwrap();

        // Ensure that subtasks are properly cascade-deleted
        assert_eq!(db.list_tasks(true).await.unwrap().len(), 0);
    }
}
