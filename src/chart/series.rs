use super::{cartesian::Plane, items};

use iced::{
    widget::canvas::{self, path::lyon_path::geom::euclid::Transform2D, Path, Stroke},
    Color, Point, Vector,
};

use std::ops::RangeInclusive;

pub trait Series<SeriesId, ItemId = usize> {
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane);
    fn items(&self) -> Option<(SeriesId, Vec<items::Entry<ItemId>>)> {
        None
    }
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

impl<Id, Data> Series<Id> for LineSeries<Data>
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

pub struct PointSeries<'a, SeriesId, Item, Data>
where
    SeriesId: Clone,
    Data: IntoIterator<Item = Item>,
{
    pub data: Data,
    pub color: Color,
    pub id: Option<SeriesId>,
    pub style_fn: Option<Box<dyn Fn(&Item) -> PointStyle + 'a>>,
}

#[derive(Debug, Clone)]
pub struct PointStyle {
    pub color: Option<iced::Color>,
    pub border: f32,
    pub radius: f32,
}

impl<'a, ID, Item, Data> PointSeries<'a, ID, Item, Data>
where
    ID: Clone,
    Data: IntoIterator<Item = Item>,
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

    pub fn style(mut self, style_fn: impl Fn(&Item) -> PointStyle + 'a) -> Self {
        self.style_fn = Some(Box::new(style_fn));
        self
    }

    pub fn with_id(mut self, id: ID) -> Self {
        self.id = Some(id);
        self
    }
}

impl<'a, Id, Item, Data> Series<Id> for PointSeries<'a, Id, Item, Data>
where
    Id: Clone,
    Data: IntoIterator<Item = Item> + Clone,
    Item: Into<(f32, f32)>,
{
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        for item in self.data.clone().into_iter() {
            let style = self
                .style_fn
                .as_ref()
                .map(|func| func(&item))
                .unwrap_or_else(PointStyle::default);

            let p = item.into();
            let point = Point {
                x: plane.scale_to_cartesian_x(p.0),
                y: plane.scale_to_cartesian_y(p.1),
            };

            let color = style.color.unwrap_or(self.color);
            frame.stroke(
                &Path::circle(point, style.radius),
                Stroke::default().with_width(style.border).with_color(color),
            );
        }
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

    fn items(&self) -> Option<(Id, Vec<items::Entry<usize>>)> {
        let id = self.id.clone()?;

        let items: Vec<_> = self
            .data
            .clone()
            .into_iter()
            .map(Into::into)
            .enumerate()
            .map(|(index, (x, y))| items::Entry::new(index, iced::Point::new(x, y)))
            .collect();

        Some((id, items))
    }
}

impl Default for PointStyle {
    fn default() -> Self {
        Self {
            color: None,
            border: 2.0,
            radius: 5.0,
        }
    }
}

pub fn line_series<Data>(data: Data) -> LineSeries<Data> {
    LineSeries::new(data)
}

pub fn point_series<'a, Id, Item, Data>(data: Data) -> PointSeries<'a, Id, Item, Data>
where
    Id: Clone,
    Data: IntoIterator<Item = Item>,
{
    PointSeries::new(data)
}
