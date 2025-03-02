use iced::Color;

#[derive(Clone)]
pub enum Series<ID>
where
    ID: Clone,
{
    Line(LineSeries),
    Point(PointSeries<ID>),
}

#[derive(Clone)]
pub struct LineSeries {
    pub data: Vec<(f32, f32)>,
    pub color: Color,
}

impl LineSeries {
    pub fn new(iter: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Self {
            data: iter.into_iter().collect(),
            color: Color::BLACK,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl<ID> From<LineSeries> for Series<ID>
where
    ID: Clone,
{
    fn from(line_series: LineSeries) -> Self {
        Self::Line(line_series)
    }
}

#[derive(Clone)]
pub struct PointSeries<ID>
where
    ID: Clone,
{
    pub data: Vec<(f32, f32)>,
    pub color: Color,
    pub id: Option<ID>,
}

impl<ID> PointSeries<ID>
where
    ID: Clone,
{
    pub fn new(iter: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Self {
            data: iter.into_iter().collect(),
            color: Color::BLACK,
            id: None,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_id(mut self, id: ID) -> Self {
        self.id = Some(id);
        self
    }
}

impl<ID> From<PointSeries<ID>> for Series<ID>
where
    ID: Clone,
{
    fn from(point_series: PointSeries<ID>) -> Self {
        Self::Point(point_series)
    }
}

pub fn line_series(iter: impl IntoIterator<Item = (f32, f32)>) -> LineSeries {
    LineSeries::new(iter)
}

pub fn point_series<ID: Clone>(iter: impl IntoIterator<Item = (f32, f32)>) -> PointSeries<ID> {
    PointSeries::new(iter)
}
