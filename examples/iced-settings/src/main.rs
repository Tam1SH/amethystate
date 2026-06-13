use iced::widget::{button, column, row, text};
use iced::{Element, Task};
use amethystate::{amethystate, StoreBuilder};
use std::sync::Arc;

#[amethystate(prefix = "iced_settings", mode = "persistent")]
pub struct SettingsState {
    #[amestate(default = "127.0.0.1".to_string())]
    pub host: String,

    #[amestate(default = 8080)]
    pub port: u16,
}

struct App {
    data: SettingsState,
}

#[derive(Debug, Clone)]
enum Message {
    IncrementPort,
    DecrementPort,
    UseLocalhost,
}

impl App {
    fn new(store: Arc<amethystate::DefaultStore>) -> (Self, Task<Message>) {
        let data = SettingsState::load(&store).expect("failed to load settings");
        (Self { data }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IncrementPort => {
                let _ = self.data.mutate_lazy(|d| d.port = d.port.saturating_add(1));
            }
            Message::DecrementPort => {
                let _ = self.data.mutate_lazy(|d| d.port = d.port.saturating_sub(1));
            }
            Message::UseLocalhost => {
                let _ = self.data.mutate_lazy(|d| d.host = "localhost".to_string());
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text("amethystate + iced").size(28),
            text(format!("host: {}", self.data.host)),
            text(format!("port: {}", self.data.port)),
            row![
                button("-").on_press(Message::DecrementPort),
                button("+").on_press(Message::IncrementPort),
                button("localhost").on_press(Message::UseLocalhost),
            ]
            .spacing(8),
        ]
        .padding(20)
        .spacing(12)
        .into()
    }
}

fn main() -> iced::Result {
    let store = StoreBuilder::new("./iced-settings.redb")
        .build()
        .expect("failed to open store");

    iced::application(move || App::new(store.clone()), App::update, App::view)
        .title("amethystate iced settings")
        .run()
}
