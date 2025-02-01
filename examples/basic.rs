extern crate pliced;

use pliced::widget::{line_series, point_series, Chart};

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
                .push_series(
                    line_series(self.data.iter().copied())
                        .color(iced::Color::from_rgb8(255, 0, 0).into()),
                )
                .push_series(
                    line_series(self.data.iter().copied().map(|(x, y)| (x, y * 0.5)))
                        .color(iced::Color::from_rgb8(0, 255, 0).into()),
                )
                .push_series(point_series(
                    self.data.iter().copied().map(|(x, y)| (x + 0.5, y * 2.0)),
                )),
        )
        .into()
    }
}
