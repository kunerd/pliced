use iced::Color;

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
    fn draw(&self);
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

// pub fn line_series<Data>(data: Data) -> LineSeries<Data> {
//     LineSeries::new(data)
// }

// pub fn point_series<ID, Data>(data: Data) -> PointSeries<ID, Data>
// where
//     ID: Clone,
//     //Data: IntoIterator + Clone,
//     //Data::Item: Into<(f32, f32)>,
// {
//     PointSeries::new(data)
// }
