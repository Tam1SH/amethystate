use iced::widget::{button, column, row, text};
use iced::{Element, Task};
use rpstate::rpstate;
use rpstate::store::builder::StoreBuilder;
use std::sync::Arc;

#[rpstate(prefix = "iced_settings")]
pub struct SettingsState {
    #[state(default = "127.0.0.1".to_string())]
    pub host: String,

    #[state(default = 8080)]
    pub port: u16,
}

struct App {
    data: SettingsState_Persistent,
}

#[derive(Debug, Clone)]
enum Message {
    IncrementPort,
    DecrementPort,
    UseLocalhost,
}

impl App {
    fn new(store: Arc<rpstate::DefaultStore>) -> (Self, Task<Message>) {
        let data = SettingsState::load(&store).expect("failed to load settings");
        (Self { data }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IncrementPort => {
                let _ = self.data.mutate(|d| d.port = d.port.saturating_add(1));
            }
            Message::DecrementPort => {
                let _ = self.data.mutate(|d| d.port = d.port.saturating_sub(1));
            }
            Message::UseLocalhost => {
                let _ = self.data.mutate(|d| d.host = "localhost".to_string());
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text("rpstate + iced").size(28),
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
        .title("rpstate iced settings")
        .run()
}
