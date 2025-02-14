use iced::Color;

#[derive(Clone)]
pub enum Series {
    Line(LineSeries),
    Point(PointSeries),
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

impl From<LineSeries> for Series {
    fn from(line_series: LineSeries) -> Self {
        Self::Line(line_series)
    }
}

#[derive(Clone)]
pub struct PointSeries {
    pub data: Vec<(f32, f32)>,
    pub color: Color,
}

impl PointSeries {
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

impl From<PointSeries> for Series {
    fn from(point_series: PointSeries) -> Self {
        Self::Point(point_series)
    }
}

pub fn line_series(iter: impl IntoIterator<Item = (f32, f32)>) -> LineSeries {
    LineSeries::new(iter)
}

pub fn point_series(iter: impl IntoIterator<Item = (f32, f32)>) -> PointSeries {
    PointSeries::new(iter)
}
