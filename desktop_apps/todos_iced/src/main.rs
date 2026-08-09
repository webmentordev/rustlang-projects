use iced::Element;
use iced::widget::{button, column, text, text_input};

#[derive(Clone)]
enum Action {
    Push,
    Pop,
    Type(String),
}

#[derive(Default)]
struct AppData {
    list: Vec<String>,
    value: String,
}

fn main() -> iced::Result {
    iced::run(AppData::update, AppData::view)
}

impl AppData {
    fn update(&mut self, action: Action) {
        match action {
            Action::Push => {
                self.list.push(self.value.to_string());
                self.value.clear();
            }
            Action::Pop => {
                self.list.pop();
            }
            Action::Type(value) => {
                self.value = value;
            }
        }
    }
    fn view(&self) -> Element<'_, Action> {
        let push = button("PUSH").on_press(Action::Push);
        let pop = button("POP").on_press(Action::Pop);
        let text_field = text_input("Type todo here...", &self.value).on_input(Action::Type);

        let mut interface = column![text_field, push, pop];

        for item in &self.list {
            interface = interface.push(text(item));
        }

        interface.into()
    }
}
