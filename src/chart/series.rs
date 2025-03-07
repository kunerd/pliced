use std::ops::RangeInclusive;

use iced::{
    widget::canvas::{self, path::lyon_path::geom::euclid::Transform2D, Path, Stroke},
    Color, Point, Vector,
};

use super::cartesian::Plane;

// pub enum Series<ID, Data>
// where
//     ID: Clone,
//     //Data: IntoIterator + Clone,
//     //Data::Item: Into<(f32, f32)>,
// {
//     Line(LineSeries<Data>),
//     Point(PointSeries<ID, Data>),
// }

pub trait Series {
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane);
    fn x_range(&self) -> RangeInclusive<f32>;
    fn y_range(&self) -> RangeInclusive<f32>;
}

#[derive(Clone)]
pub struct LineSeries<Data> {
    pub data: Data,
    pub color: Color,
}

impl<Data> LineSeries<Data> {
    pub fn new(data: Data) -> Self {
        Self {
            data,
            color: Color::BLACK,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl<Data> Series for LineSeries<Data>
where
    Data: IntoIterator + Clone,
    Data::Item: Into<(f32, f32)>,
{
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        frame.with_save(|frame| {
            frame.translate(Vector::new(plane.x.margin_min, plane.x.margin_min));
            frame.scale_nonuniform(Vector::new(plane.x.scale, plane.y.scale));
            frame.translate(Vector::new(-plane.x.min, plane.y.max));

            let mut iter = self
                .data
                .clone()
                .into_iter()
                .map(Into::into)
                .filter(|(x, y)| {
                    x >= &plane.x.min && x <= &plane.x.max && y >= &plane.y.min && y <= &plane.y.max
                });

            let path = Path::new(|b| {
                if let Some(p) = iter.next() {
                    b.move_to(Point { x: p.0, y: p.1 });
                    iter.fold(b, |acc, p| {
                        acc.line_to(Point { x: p.0, y: p.1 });
                        acc
                    });
                }
            });

            frame.stroke(
                &path.transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                Stroke::default().with_width(2.0).with_color(self.color),
            );
        })
    }

    fn x_range(&self) -> RangeInclusive<f32> {
        let x_min_cur = f32::INFINITY;
        let x_max_cur = f32::NEG_INFINITY;

        let (x_min, x_max) = {
            self.data
                .clone()
                .into_iter()
                .map(Into::into)
                .fold((x_min_cur, x_max_cur), |(x_min, x_max), (cur_x, _)| {
                    (x_min.min(cur_x), x_max.max(cur_x))
                })
        };

        x_min..=x_max
    }

    fn y_range(&self) -> RangeInclusive<f32> {
        let y_min_cur = f32::INFINITY;
        let y_max_cur = f32::NEG_INFINITY;

        let (y_min, y_max) = {
            self.data
                .clone()
                .into_iter()
                .map(Into::into)
                .fold((y_min_cur, y_max_cur), |(y_min, y_max), (_, cur_y)| {
                    (y_min.min(cur_y), y_max.max(cur_y))
                })
        };

        y_min..=y_max
    }
}

// impl<ID, Data> From<LineSeries<Data>> for Series<ID, Data>
// where
//     ID: Clone,
//     //Data: IntoIterator + Clone,
//     //Data::Item: Into<(f32, f32)>,
// {
//     fn from(line_series: LineSeries<Data>) -> Self {
//         Self::Line(line_series)
//     }
// }

pub struct PointSeries<ID, Data>
where
    ID: Clone,
    //Data: IntoIterator + Clone,
    //Data::Item: Into<(f32, f32)>,
{
    pub data: Data,
    pub color: Color,
    pub id: Option<ID>,
    pub style_fn: Option<Box<dyn Fn(usize) -> f32>>,
}

impl<ID, Data> PointSeries<ID, Data>
where
    ID: Clone,
    //Data: IntoIterator + Clone,
    //Data::Item: Into<(f32, f32)>,
{
    pub fn new(data: Data) -> Self {
        Self {
            data,
            color: Color::BLACK,
            id: None,
            style_fn: None,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn style(mut self, style_fn: impl Fn(usize) -> f32 + 'static) -> Self {
        self.style_fn = Some(Box::new(style_fn));
        self
    }

    pub fn with_id(mut self, id: ID) -> Self {
        self.id = Some(id);
        self
    }
}

impl<ID, DATA> Series for PointSeries<ID, DATA>
where
    ID: Clone,
    DATA: IntoIterator + Clone,
    DATA::Item: Into<(f32, f32)>,
{
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        let mut iter = self
            .data
            .clone()
            .into_iter()
            .map(Into::into)
            .enumerate()
            .filter(|(_i, (x, y))| {
                x >= &plane.x.min && x <= &plane.x.max && y >= &plane.y.min && y <= &plane.y.max
            });

        const DEFAULT_RADIUS: f32 = 4.0;
        let path = Path::new(|b| {
            if let Some((i, p)) = iter.next() {
                let radius = self
                    .style_fn
                    .as_ref()
                    .map(|func| func(i))
                    .unwrap_or(DEFAULT_RADIUS);

                let point = Point {
                    x: plane.scale_to_cartesian_x(p.0),
                    y: plane.scale_to_cartesian_y(p.1),
                };

                b.circle(point, radius);
                iter.fold(b, |acc, (i, p)| {
                    let radius = self
                        .style_fn
                        .as_ref()
                        .map(|func| func(i))
                        .unwrap_or(DEFAULT_RADIUS);
                    let point = Point {
                        x: plane.scale_to_cartesian_x(p.0),
                        y: plane.scale_to_cartesian_y(p.1),
                    };
                    acc.circle(point, radius);
                    acc
                });
            }
        });
        frame.stroke(
            &path,
            Stroke::default().with_width(2.0).with_color(self.color),
        );
    }

    fn x_range(&self) -> RangeInclusive<f32> {
        let x_min_cur = f32::INFINITY;
        let x_max_cur = f32::NEG_INFINITY;

        let (x_min, x_max) = {
            self.data
                .clone()
                .into_iter()
                .map(Into::into)
                .fold((x_min_cur, x_max_cur), |(x_min, x_max), (cur_x, _)| {
                    (x_min.min(cur_x), x_max.max(cur_x))
                })
        };

        x_min..=x_max
    }

    fn y_range(&self) -> RangeInclusive<f32> {
        let y_min_cur = f32::INFINITY;
        let y_max_cur = f32::NEG_INFINITY;

        let (y_min, y_max) = {
            self.data
                .clone()
                .into_iter()
                .map(Into::into)
                .fold((y_min_cur, y_max_cur), |(y_min, y_max), (_, cur_y)| {
                    (y_min.min(cur_y), y_max.max(cur_y))
                })
        };

        y_min..=y_max
    }
}
// impl<ID, Data> From<PointSeries<ID, Data>> for Series<ID, Data>
// where
//     ID: Clone,
//     //Data: IntoIterator + Clone,
//     //Data::Item: Into<(f32, f32)>,
// {
//     fn from(point_series: PointSeries<ID, Data>) -> Self {
//         Self::Point(point_series)
//     }
// }

pub fn line_series<Data>(data: Data) -> LineSeries<Data> {
    LineSeries::new(data)
}

pub fn point_series<ID, Data>(data: Data) -> PointSeries<ID, Data>
where
    ID: Clone,
{
    PointSeries::new(data)
}
