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

// pub fn point_series<ID, Data>(data: Data) -> PointSeries<ID, Data>
// where
//     ID: Clone,
//     //Data: IntoIterator + Clone,
//     //Data::Item: Into<(f32, f32)>,
// {
//     PointSeries::new(data)
// }
