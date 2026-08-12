use dirs;
use iced::{
    Element, Length, Task,
    widget::{button, column, container, row, scrollable, text, text_input},
};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;

#[derive(Debug, Clone, sqlx::FromRow)]
struct Note {
    id: i64,
    text: String,
}

#[derive(Clone)]
struct App {
    db: Option<Arc<SqlitePool>>,
    notes: Vec<Note>,
    input_value: String,
}

#[derive(Debug, Clone)]
enum ActionMessages {
    DbConnect(Arc<SqlitePool>),
    LoadNotes(Result<Vec<Note>, String>),
    InputChanged(String),
    AddNote,
    NoteAdded(Result<Note, String>),
    DeleteNote(i64),
    ActionDone(Result<(), String>),
}

impl App {
    fn new() -> (Self, Task<ActionMessages>) {
        let app = App {
            db: None,
            notes: Vec::new(),
            input_value: String::new(),
        };
        (
            app,
            Task::perform(connect_to_db(), ActionMessages::DbConnect),
        )
    }
    fn update(&mut self, actions: ActionMessages) -> Task<ActionMessages> {
        match actions {
            ActionMessages::DbConnect(pool) => {
                self.db = Some(pool.clone());
                Task::perform(
                    async move { load_notes(&pool).await },
                    ActionMessages::LoadNotes,
                )
            }
            ActionMessages::LoadNotes(Ok(notes)) => {
                self.notes = notes;
                Task::none()
            }
            ActionMessages::LoadNotes(Err(e)) => {
                println!("Error loading notes: {}", e);
                Task::none()
            }
            ActionMessages::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            ActionMessages::AddNote => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };
                if self.input_value.trim().is_empty() {
                    return Task::none();
                };
                let text = std::mem::take(&mut self.input_value);
                Task::perform(
                    async move { add_note(&db, &text).await },
                    ActionMessages::NoteAdded,
                )
            }
            ActionMessages::NoteAdded(Ok(note)) => {
                self.notes.push(note);
                Task::none()
            }
            ActionMessages::NoteAdded(Err(e)) => {
                println!("Error adding the note: {}", e);
                Task::none()
            }
            ActionMessages::DeleteNote(id) => {
                let Some(db) = self.db.clone() else {
                    return Task::none();
                };
                self.notes.retain(|t| t.id != id);
                Task::perform(
                    async move { delete_note(&db, id).await },
                    ActionMessages::ActionDone,
                )
            }
            ActionMessages::ActionDone(Err(e)) => {
                println!("DB action failed: {e}");
                Task::none()
            }
            ActionMessages::ActionDone(Ok(())) => Task::none(),
        }
    }
    fn view(&self) -> Element<'_, ActionMessages> {
        if self.db.is_none() {
            return container(text("Loading...")).center(Length::Fill).into();
        }
        let input = text_input("What need doing?", &self.input_value)
            .on_input(ActionMessages::InputChanged)
            .on_submit(ActionMessages::AddNote);
        let notes: Element<_> = if self.notes.is_empty() {
            text("No note yet").into()
        } else {
            column(self.notes.iter().map(|note| {
                row![
                    text(&note.text).width(Length::Fill),
                    button("Delete").on_press(ActionMessages::DeleteNote(note.id)),
                ]
                .spacing(10)
                .into()
            }))
            .spacing(8)
            .into()
        };

        container(
            column![input, scrollable(notes)]
                .spacing(20)
                .padding(20)
                .max_width(500),
        )
        .center_x(Length::Fill)
        .into()
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Notes App")
        .resizable(true)
        .run()
}

async fn connect_to_db() -> Arc<SqlitePool> {
    let db_path = dirs::home_dir()
        .expect("Failed to get home directory")
        .join("notes.db");

    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to sqlite");

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS notes(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL
        )
    ",
    )
    .execute(&pool)
    .await
    .unwrap();

    Arc::new(pool)
}

async fn load_notes(pool: &SqlitePool) -> Result<Vec<Note>, String> {
    sqlx::query_as("SELECT id, text FROM notes")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn add_note(pool: &SqlitePool, text: &str) -> Result<Note, String> {
    let id: i64 = sqlx::query_scalar("INSERT into notes (text) VALUES (?1) RETURNING id")
        .bind(text)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Note {
        id: id,
        text: text.to_string(),
    })
}

async fn delete_note(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM notes WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
