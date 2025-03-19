struct PointSeries<'a> {
    data: Box<dyn SeriesData + 'a>,
}

impl<'a> PointSeries<'a> {
    fn new(data: impl SeriesData + 'a) -> Self {
        Self {
            data: Box::new(data),
        }
    }
}

trait SeriesData {
    fn data_iter(&self) -> Box<dyn Iterator<Item = (f32, f32)> + '_>;
}

impl<T> SeriesData for T
where
    T: IntoIterator + Clone,
    T::Item: Into<(f32, f32)>,
{
    fn data_iter(&self) -> Box<dyn Iterator<Item = (f32, f32)> + '_> {
        Box::new(self.clone().into_iter().map(Into::into))
    }
}

struct DataPoint {
    x: f32,
    y: f32,
}

struct Chart<'a> {
    series: Vec<PointSeries<'a>>,
}

impl<'a> Chart<'a> {
    fn new() -> Self {
        Self { series: vec![] }
    }

    fn push_series(mut self, series: PointSeries<'a>) -> Self {
        self.series.push(series);
        self
    }

    fn draw(&self) {
        for s in &self.series {
            for p in s.data.data_iter() {
                let (x, y) = p.into();
                println!("x: {x}, y: {y}");
            }
        }
    }
}

fn main() {
    let data = vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }];

    let point_series_1 = PointSeries::new(&data);

    let data = vec![(0.0, 0.0), (1.0, 1.0)];
    let point_series_2 = PointSeries::new(data.iter().copied());

    let chart = Chart::new()
        .push_series(point_series_1)
        .push_series(point_series_2);

    chart.draw();
}

impl From<DataPoint> for (f32, f32) {
    fn from(point: DataPoint) -> (f32, f32) {
        (point.x, point.y)
    }
}

impl From<&DataPoint> for (f32, f32) {
    fn from(point: &DataPoint) -> (f32, f32) {
        (point.x, point.y)
    }
}
