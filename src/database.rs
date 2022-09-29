use sqlx::{SqliteConnection, Connection, Error, sqlite::SqliteQueryResult};

// Schema is on <app.dbdesigner.net>

pub struct Database {
    connection: SqliteConnection
}

#[derive(Debug)]
pub struct Task {
    pub id: i64,  // IMPORTANT: This begins from 1, *not* 0
    pub description: String
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            connection: SqliteConnection::connect("sqlite::memory:").await?,
        })
    }

    pub async fn setup(&mut self) -> Result<SqliteQueryResult, Error> {
        sqlx::query("CREATE TABLE tasks (
            id integer PRIMARY KEY AUTOINCREMENT,
            description text NOT NULL
        )").execute(&mut self.connection).await
    }

    pub async fn add_task(&mut self, task: &str) -> Result<SqliteQueryResult, Error> {
        sqlx::query!("INSERT INTO tasks VALUES (null, ?)", task).execute(&mut self.connection).await
    }

    pub async fn remove_task(&mut self, index: i32) -> Result<u64, Error> {
        Ok(sqlx::query!("DELETE FROM tasks WHERE id = ?", index).execute(&mut self.connection).await?.rows_affected())
    }

    pub async fn list_tasks(&mut self) -> Result<Vec<Task>, Error>{
        sqlx::query_as!(Task, "SELECT * FROM tasks").fetch_all(&mut self.connection).await
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
