use sqlx::{sqlite::SqliteQueryResult, Connection, Error, SqliteConnection};

// Schema is on <app.dbdesigner.net>

pub struct Database {
    connection: SqliteConnection,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: i64, // IMPORTANT: This begins from 1, *not* 0
    pub description: String,
    pub complete: bool,
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            connection: SqliteConnection::connect("sqlite::memory:").await?,
        })
    }

    pub async fn setup(&mut self) -> Result<SqliteQueryResult, Error> {
        sqlx::query(include_str!("./create.sql"))
            .execute(&mut self.connection)
            .await
    }

    pub async fn add_task(&mut self, task: &str) -> Result<Task, Error> {
        sqlx::query_as!(
            Task,
            "INSERT INTO tasks VALUES (null, ?, false) RETURNING id, description as 'description!', complete",
            task
        )
        .fetch_one(&mut self.connection)
        .await
    }

    pub async fn remove_task(&mut self, index: i64) -> Result<u64, Error> {
        Ok(sqlx::query!("DELETE FROM tasks WHERE id = ?", index)
            .execute(&mut self.connection)
            .await?
            .rows_affected())
    }

    pub async fn list_tasks(&mut self) -> Result<Vec<Task>, Error> {
        sqlx::query_as!(Task, "SELECT * FROM tasks")
            .fetch_all(&mut self.connection)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_task_test() {
        let mut db = Database::new().await.unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks().await.unwrap().len(), 0);

        db.add_task("A test task").await.unwrap();

        assert_eq!(db.list_tasks().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn remove_task_test() {
        let mut db = Database::new().await.unwrap();

        db.setup().await.unwrap();

        assert_eq!(db.list_tasks().await.unwrap().len(), 0);

        db.add_task("A test task").await.unwrap();

        for task in db.list_tasks().await.unwrap() {
            println!("Task got ID {}", task.id);
        }

        assert_eq!(db.list_tasks().await.unwrap().len(), 1);

        assert_eq!(db.remove_task(1).await.unwrap(), 1);
        db.remove_task(0).await.unwrap();

        assert_eq!(db.list_tasks().await.unwrap().len(), 0);
    }
}
