use sqlx::{SqliteConnection, Connection, Error, sqlite::SqliteQueryResult};

// Schema is on <app.dbdesigner.net>

pub struct Database {
    connection: SqliteConnection
}

#[derive(Debug)]
pub struct Task {
    pub id: i64,
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

    pub fn remove_task(index: i32) {}

    pub async fn list_tasks(&mut self) -> Result<Vec<Task>, Error>{
        sqlx::query_as!(Task, "SELECT * FROM tasks").fetch_all(&mut self.connection).await
    }
}

