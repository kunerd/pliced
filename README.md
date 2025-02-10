# pliced

An experimental `Chart` widget for [iced](https://github.com/iced-rs/iced).

Currently this is in a very early stage and uses [plotters](https://github.com/plotters-rs/plotters) as backend,
but that may change.

## Usage
There are two ways to use this library. You can just use the provided chart widget:
```rust
  Chart::new()
      .width(Length::Fill)
      .height(Length::Fill)
      .x_range(-1.0..3.0)
      .y_range(-1.0..5.0)
      .push_series(
          line_series(self.data.iter().copied())
              .color(iced::Color::from_rgb8(255, 0, 0).into()),
      )
      .push_series(
          line_series(self.data.iter().copied().map(|(x, y)| (x, y * 0.5)))
              .color(iced::Color::from_rgb8(0, 255, 0).into()),
      )
      .push_series(point_series(
          self.data.iter().copied().map(|(x, y)| (x + 0.5, y * 2.0)),
      ))
```
or define a custom chart `Program` which provides full access to the underlying `plotters` `ChartBuilder`:
```rust
impl pliced::Program<Message> for App {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        chart: &mut plotters::prelude::ChartBuilder<pliced::backend::IcedChartBackend<Renderer>>,
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
```
Take a look into [examples](examples) for more information.
