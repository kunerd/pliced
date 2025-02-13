extern crate pliced;

use iced::Theme;
use pliced::custom_chart::Chart;
use pliced::widget::{line_series, point_series};

use iced::{widget::container, Element, Length, Task};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::new)
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
        let palette = self.theme().palette();
        container(
            Chart::new()
                .width(Length::Fill)
                .height(Length::Fill)
                //.x_range(-6.0..-1.0)
                //.y_range(-5.0..-1.0)
                .push_series(line_series(self.data.iter().copied()).color(palette.primary))
                .push_series(
                    line_series(self.data.iter().copied().map(|(x, y)| (x, y * 0.5)))
                        .color(palette.success),
                )
                //.push_series(
                //    point_series(self.data.iter().copied().map(|(x, y)| (x + 0.5, y * 2.0)))
                //        .color(palette.danger),
                //),
        )
        .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
