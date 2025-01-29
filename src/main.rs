use iced::{Element, Length, Task};
use pliced::widget::ChartWidget;
use plotters::{
    prelude::PathElement,
    series::LineSeries,
    style::{Color, IntoFont, BLACK, RED, WHITE},
};

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
        let data = self.data.to_vec();
        ChartWidget::new(move |chart| {
            let mut chart = chart
                .caption("My Chart", ("sans-serif", 50).into_font())
                .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
                .unwrap();
            chart.configure_mesh().draw().unwrap();

            chart
                .draw_series(LineSeries::new(data.iter().cloned(), &RED))
                .unwrap()
                .label("y = x^2")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

            chart
                .configure_series_labels()
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK)
                .draw()
                .unwrap();
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
