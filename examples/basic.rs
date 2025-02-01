extern crate pliced;

use pliced::widget::{Chart, Series};

use iced::{widget::container, Element, Length, Task};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view).run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {}

#[derive(Debug, Default)]
struct App {
    data: Vec<(f32, f32)>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = (-50..=50)
            .map(|x| x as f32 / 50.0)
            .map(|x| (x, x * x))
            .collect();

        (Self { data }, Task::none())
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, _msg: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            Chart::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .series(Series::Line {
                    color: iced::Color::from_rgba8(255, 0, 0, 0.5).into(),
                    data: self.data.clone(),
                }),
        )
        .into()
    }
}
