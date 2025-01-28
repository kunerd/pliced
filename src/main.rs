use iced::{widget::text, Element, Length, Task};
use widget::ChartWidget;

mod backend;
mod utils;
mod widget;

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view).run()
}

#[derive(Debug, Clone)]
enum Message {}

#[derive(Debug, Default)]
struct App;

impl App {
    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, _msg: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sample_rate = 10f32;
        let series = (0..100)
            .map(|t| {
                let sample_clock = t as f32 % sample_rate;
                (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
            })
            .collect();

        ChartWidget::new(series)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
