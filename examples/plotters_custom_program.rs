use iced::{Element, Renderer, Task};
use pliced::plotters::{Chart, IcedChartBackend, Program};
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
        Chart::from_program(self).into()
    }
}

impl Program<Message> for App {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        chart: &mut plotters::prelude::ChartBuilder<IcedChartBackend<Renderer>>,
        _theme: &iced::Theme,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) {
        let mut chart = chart
            .caption("y=x^2", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(LineSeries::new(self.data.iter().cloned(), &RED))
            .unwrap()
            .label("y = x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();
    }
}
