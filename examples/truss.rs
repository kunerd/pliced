use std::ops::RangeInclusive;

use pliced::chart::{Chart, Labels, Margin, Plane, Series};

use iced::{
    Element, Length, Point, Task, Theme, Vector,
    widget::{
        canvas::{self, Path, Stroke, path::lyon_path::geom::euclid::Transform2D},
        container,
    },
};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::new)
}

#[derive(Debug)]
struct App {
    data: Vec<((f32, f32), (f32, f32))>,
}

#[derive(Debug, Clone)]
enum Message {}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = vec![
            ((0.0, 0.0), (1.0, 0.0)),
            ((1.0, 0.0), (0.5, 1.0)),
            ((0.5, 1.0), (0.5, 2.0)),
            ((0.5, 1.0), (0.0, 0.0)),
        ];

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
            Chart::<_, (), _>::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .margin(Margin {
                    top: 0.0,
                    bottom: 20.0,
                    left: 0.0,
                    right: 0.0,
                })
                .x_range(-0.5..=2.0)
                .y_range(-1.0..=3.0)
                .x_labels(Labels::default().format(&|v| format!("{v:.2}")))
                .y_labels(Labels::default().format(&|v| format!("{v:.2}")))
                .push_series(Truss {
                    data: self.data.iter().copied(),
                    width: 4.0,
                    color: palette.success,
                }),
        )
        .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}

#[derive(Clone)]
pub struct Truss<Data> {
    pub data: Data,
    pub color: iced::Color,
    pub width: f32,
}

impl<Data> Truss<Data> {
    pub fn new(data: Data) -> Self {
        Self {
            data,
            width: 2.0,
            color: iced::Color::BLACK,
        }
    }

    pub fn color(mut self, color: impl Into<iced::Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl<Id, Data> Series<Id> for Truss<Data>
where
    Data: IntoIterator + Clone,
    Data::Item: Into<((f32, f32), (f32, f32))>,
{
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        frame.with_save(|frame| {
            frame.translate(Vector::new(plane.x.margin_min, plane.x.margin_min));
            frame.scale_nonuniform(Vector::new(plane.x.scale, plane.y.scale));
            frame.translate(Vector::new(-plane.x.min, plane.y.max));

            let iter = self.data.clone().into_iter().map(Into::into);

            // TODO: clipping
            // .filter(|((x, y)| {
            //     x >= &plane.x.min && x <= &plane.x.max && y >= &plane.y.min && y <= &plane.y.max
            // });

            let path = Path::new(|b| {
                iter.fold(b, |acc, (start, end)| {
                    acc.move_to(Point {
                        x: start.0,
                        y: start.1,
                    });
                    acc.line_to(Point { x: end.0, y: end.1 });
                    acc
                });
            });

            frame.stroke(
                &path.transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                Stroke::default()
                    .with_width(self.width)
                    .with_color(self.color),
            );
        })
    }

    fn x_range(&self) -> RangeInclusive<f32> {
        //FIXME: proper range computation
        // let mut x_min_cur = f32::INFINITY;
        // let mut x_max_cur = f32::NEG_INFINITY;

        // for (from, to) in self.data.clone().into_iter().map(Into::into) {
        //     x_min_cur.min()
        // }

        // x_min..=x_max

        -1.0..=2.0
    }

    fn y_range(&self) -> RangeInclusive<f32> {
        //FIXME: proper range computation
        // let y_min_cur = f32::INFINITY;
        // let y_max_cur = f32::NEG_INFINITY;

        // let (y_min, y_max) = {
        //     self.data
        //         .clone()
        //         .into_iter()
        //         .map(Into::into)
        //         .fold((y_min_cur, y_max_cur), |(y_min, y_max), (_, cur_y)| {
        //             (y_min.min(cur_y), y_max.max(cur_y))
        //         })
        // };

        // y_min..=y_max

        -1.0..=2.0
    }
}
