use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Todo List")
        .run()
}

#[derive(Clone, sqlx::FromRow)]
struct Todo {
    id: i64,
    text: String,
    done: bool,
}

struct App {
    db: Option<Arc<SqlitePool>>,
    todos: Vec<Todo>,
    input_value: String,
}

#[derive(Clone)]
enum Message {
    DbConnected(Arc<SqlitePool>),
    TodosLoaded(Result<Vec<Todo>, String>),
    InputChanged(String),
    AddTodo,
    TodoAdded(Result<Todo, String>),
    ToggleTodo(i64, bool),
    DeleteTodo(i64),
    ActionDone(Result<(), String>),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            db: None,
            todos: Vec::new(),
            input_value: String::new(),
        };

        (app, Task::perform(setup_db(), Message::DbConnected))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DbConnected(pool) => {
                self.db = Some(pool.clone());
                Task::perform(async move { load_todos(&pool).await }, Message::TodosLoaded)
            }
            Message::TodosLoaded(Ok(todos)) => {
                self.todos = todos;
                Task::none()
            }
            Message::TodosLoaded(Err(e)) => {
                println!("Error loading todos: {e}");
                Task::none()
            }
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::AddTodo => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };
                if self.input_value.trim().is_empty() {
                    return Task::none();
                }
                let text = std::mem::take(&mut self.input_value);
                Task::perform(
                    async move { add_todo(&db, &text).await },
                    Message::TodoAdded,
                )
            }
            Message::TodoAdded(Ok(todo)) => {
                self.todos.push(todo);
                Task::none()
            }
            Message::TodoAdded(Err(e)) => {
                println!("Error adding todo: {e}");
                Task::none()
            }
            Message::ToggleTodo(id, done) => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };
                if let Some(t) = self.todos.iter_mut().find(|t| t.id == id) {
                    t.done = done;
                }
                Task::perform(
                    async move { toggle_todo(&db, id, done).await },
                    Message::ActionDone,
                )
            }
            Message::DeleteTodo(id) => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };
                self.todos.retain(|t| t.id != id);
                Task::perform(
                    async move { delete_todo(&db, id).await },
                    Message::ActionDone,
                )
            }
            Message::ActionDone(Err(e)) => {
                println!("DB action failed: {e}");
                Task::none()
            }
            Message::ActionDone(Ok(())) => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if self.db.is_none() {
            return container(text("Loading...")).center(Length::Fill).into();
        }

        let input = text_input("What needs doing?", &self.input_value)
            .on_input(Message::InputChanged)
            .on_submit(Message::AddTodo)
            .padding(10);

        let todos: Element<_> = if self.todos.is_empty() {
            text("No todos yet").into()
        } else {
            column(self.todos.iter().map(|todo| {
                row![
                    checkbox(todo.done)
                        .on_toggle(move |checked| Message::ToggleTodo(todo.id, checked)),
                    text(&todo.text).width(Length::Fill),
                    button("Delete").on_press(Message::DeleteTodo(todo.id)),
                ]
                .spacing(10)
                .into()
            }))
            .spacing(8)
            .into()
        };

        container(
            column![input, scrollable(todos)]
                .spacing(20)
                .padding(20)
                .max_width(500),
        )
        .center_x(Length::Fill)
        .into()
    }
}

async fn setup_db() -> Arc<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect("sqlite://database.db?mode=rwc")
        .await
        .expect("Failed to connect to SQLite");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            done BOOLEAN NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    Arc::new(pool)
}

async fn load_todos(pool: &SqlitePool) -> Result<Vec<Todo>, String> {
    sqlx::query_as("SELECT id, text, done FROM todos ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn add_todo(pool: &SqlitePool, text: &str) -> Result<Todo, String> {
    let id: i64 = sqlx::query_scalar("INSERT INTO todos (text, done) VALUES (?1, 0) RETURNING id")
        .bind(text)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Todo {
        id,
        text: text.to_string(),
        done: false,
    })
}

async fn toggle_todo(pool: &SqlitePool, id: i64, done: bool) -> Result<(), String> {
    sqlx::query("UPDATE todos SET done = ?1 WHERE id = ?2")
        .bind(done)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn delete_todo(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
