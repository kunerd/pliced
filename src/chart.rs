use std::ops::Range;

use iced::{
    advanced::{graphics::geometry, layout, widget::tree, Widget},
    mouse, Element, Length, Renderer, Size,
};
use plotters::{
    chart::ChartBuilder,
    series::LineSeries,
    style::{Color as _, ShapeStyle, TextStyle, BLUE, GREEN},
};
use plotters_backend::{text_anchor::Pos, BackendColor};

use crate::{backend, widget::ChartWidget, Program};

pub struct Chart<Data = Vec<(f32, f32)>>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    raw_chart: RawChart<Data>,
    width: Length,
    height: Length,
}

impl<Data> Chart<Data>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    const DEFAULT_SIZE: f32 = 100.0;

    pub fn new() -> Self {
        let raw_chart = RawChart::default();
        Self {
            raw_chart,
            width: Length::Fixed(Self::DEFAULT_SIZE),
            height: Length::Fixed(Self::DEFAULT_SIZE),
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();

        self
    }

    pub fn heigth(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();

        self
    }

    pub fn data(mut self, series: Series<Data>) -> Self {
        match &series {
            Series::Line { color: _, data } => {
                let x_min = data
                    .clone()
                    .into_iter()
                    .min_by(|(x1, _), (x2, _)| x1.total_cmp(x2))
                    .map(|(x, _)| x);

                let x_max = data
                    .clone()
                    .into_iter()
                    .max_by(|(x1, _), (x2, _)| x1.total_cmp(x2))
                    .map(|(x, _)| x);

                if let (Some(min), Some(max)) = (x_min, x_max) {
                    self.raw_chart.x_range = min..max;
                }

                let y_min = data
                    .clone()
                    .into_iter()
                    .min_by(|(_, y1), (_, y2)| y1.total_cmp(y2))
                    .map(|(_, y)| y);
                let y_max = data
                    .clone()
                    .into_iter()
                    .max_by(|(_, y1), (_, y2)| y1.total_cmp(y2))
                    .map(|(_, y)| y);

                if let (Some(min), Some(max)) = (y_min, y_max) {
                    self.raw_chart.y_range = min..max;
                }
            }
        }

        self.raw_chart.series = Some(series);

        self
    }
}

impl<Message, Data> Widget<Message, iced::Theme, iced::Renderer> for Chart<Data>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
    Renderer: iced::advanced::Renderer + geometry::Renderer,
{
    fn state(&self) -> tree::State {
        tree::State::new(())
        //tree::State::new(<RawChart<Data> as Program<Message>>::State::default())
    }

    fn size(&self) -> iced::Size<iced::Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let widget: ChartWidget<'_, RawChart<_>, Message> =
            ChartWidget::new(self.raw_chart.clone());

        widget.draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
}

#[derive(Clone)]
struct RawChart<Data>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    x_range: Range<f32>,
    y_range: Range<f32>,
    series: Option<Series<Data>>,
}

impl<Data> Default for RawChart<Data>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    fn default() -> Self {
        Self {
            x_range: 0.0..10.0,
            y_range: 0.0..10.0,
            series: None,
        }
    }
}

impl<Message, Data> Program<Message> for RawChart<Data>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        chart: &mut ChartBuilder<backend::IcedChartBackend<Renderer>>,
        theme: &iced::Theme,
        _bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) {
        let mut chart = chart
            .x_label_area_size(10)
            .margin(20)
            .build_cartesian_2d(self.x_range.clone(), self.y_range.clone())
            .unwrap();

        let text_color = Color(theme.palette().text);
        let label_style = TextStyle {
            font: "sans".into(),
            color: text_color.into(),
            pos: Pos::default(),
        };

        chart
            .configure_mesh()
            .label_style(label_style)
            .x_labels(10)
            .bold_line_style(GREEN.mix(0.1))
            .light_line_style(BLUE.mix(0.1))
            .draw()
            .unwrap();

        if let Some(Series::Line { data, color }) = &self.series {
            let style: ShapeStyle = color.into();
            chart
                .draw_series(LineSeries::new(data.clone(), style))
                .unwrap();
        }
    }
}

#[derive(Clone)]
pub enum Series<Data>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    Line { color: Color, data: Data },
}

#[derive(Clone, Copy)]
pub struct Color(iced::Color);

impl From<iced::Color> for Color {
    fn from(color: iced::Color) -> Self {
        Self(color)
    }
}

impl From<Color> for plotters::style::RGBAColor {
    fn from(color: Color) -> Self {
        let color = color.0.into_rgba8();
        Self(color[0], color[1], color[2], color[3] as f64 / 256.0)
    }
}

impl From<Color> for ShapeStyle {
    fn from(color: Color) -> Self {
        ShapeStyle {
            color: color.into(),
            filled: true,
            stroke_width: 2,
        }
    }
}

impl From<&Color> for ShapeStyle {
    fn from(color: &Color) -> Self {
        ShapeStyle {
            color: (*color).into(),
            filled: true,
            stroke_width: 2,
        }
    }
}

impl From<Color> for BackendColor {
    fn from(color: Color) -> Self {
        let color: plotters::style::RGBAColor = color.into();
        color.to_backend_color()
    }
}

impl<'a, Message, Data> From<Chart<Data>> for Element<'a, Message, iced::Theme, iced::Renderer>
where
    Data: 'a + IntoIterator<Item = (f32, f32)> + Clone,
{
    fn from(chart: Chart<Data>) -> Self {
        Element::new(chart)
    }
}
