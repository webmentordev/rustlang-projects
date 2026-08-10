use iced::{Element, Task};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Counter")
        .run()
}

struct App {
    db: Option<Arc<SqlitePool>>,
    counter_value: i32,
    is_loading: bool,
}

#[derive(Debug, Clone)]
enum Message {
    DbConnected(Arc<SqlitePool>),
    Increment,
    Decrement,
    DbValueFetched(Result<i32, String>),
    DbValueSaved(Result<(), String>),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            db: None,
            counter_value: 0,
            is_loading: true,
        };

        let connect_task = Task::perform(setup_db(), Message::DbConnected);

        (app, connect_task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DbConnected(pool) => {
                self.db = Some(pool.clone());
                Task::perform(
                    async move { fetch_counter_from_db(&pool).await },
                    Message::DbValueFetched,
                )
            }
            Message::Increment => {
                self.counter_value += 1;
                self.is_loading = true;
                self.save_current()
            }
            Message::Decrement => {
                self.counter_value -= 1;
                self.is_loading = true;
                self.save_current()
            }
            Message::DbValueFetched(Ok(val)) => {
                self.counter_value = val;
                self.is_loading = false;
                Task::none()
            }
            Message::DbValueFetched(Err(e)) => {
                println!("Error loading data: {e}");
                self.is_loading = false;
                Task::none()
            }
            Message::DbValueSaved(Ok(())) => {
                self.is_loading = false;
                Task::none()
            }
            Message::DbValueSaved(Err(e)) => {
                println!("Error saving data: {e}");
                self.is_loading = false;
                Task::none()
            }
        }
    }

    fn save_current(&self) -> Task<Message> {
        let Some(db) = self.db.clone() else {
            return Task::none();
        };
        let val = self.counter_value;
        Task::perform(
            async move { save_counter_to_db(&db, val).await },
            Message::DbValueSaved,
        )
    }

    fn view(&self) -> Element<Message> {
        use iced::widget::{button, center, column, row, text};

        if self.is_loading || self.db.is_none() {
            return center(text("Syncing with database...")).into();
        }

        let content = column![
            text(format!("Counter Value: {}", self.counter_value)).size(30),
            row![
                button("-").on_press(Message::Decrement),
                button("+").on_press(Message::Increment),
            ]
            .spacing(10)
        ]
        .spacing(20);

        center(content).into()
    }
}

async fn setup_db() -> Arc<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("database.db")
        .await
        .expect("Failed to connect to SQLite");

    sqlx::query("CREATE TABLE IF NOT EXISTS counts (id INTEGER PRIMARY KEY, val INTEGER)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO counts (id, val) VALUES (1, 0) ON CONFLICT DO NOTHING")
        .execute(&pool)
        .await
        .unwrap();

    Arc::new(pool)
}

async fn fetch_counter_from_db(pool: &SqlitePool) -> Result<i32, String> {
    let row: (i32,) = sqlx::query_as("SELECT val FROM counts WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.0)
}

async fn save_counter_to_db(pool: &SqlitePool, value: i32) -> Result<(), String> {
    sqlx::query("UPDATE counts SET val = ?1 WHERE id = 1")
        .bind(value)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
