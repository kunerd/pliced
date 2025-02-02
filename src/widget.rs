use core::f32;
use std::marker::PhantomData;
use std::ops::Range;

use crate::backend::{self, IcedChartBackend};
use crate::event::{self, Event};
use crate::program::Program;

use iced::advanced::graphics::geometry;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout, mouse, renderer, Clipboard, Layout, Shell, Widget};
use iced::widget::canvas;
use iced::widget::text::Shaping;
use iced::{mouse::Cursor, Element, Length, Rectangle, Size};
use iced::{Renderer, Vector};
use plotters::prelude::*;
use plotters::style::Color as _;
use plotters_backend::text_anchor::Pos;
use plotters_backend::BackendColor;

pub type ChartBuilderFn<Renderer = iced::Renderer> =
    Box<dyn for<'a, 'b> Fn(&mut ChartBuilder<'a, 'b, IcedChartBackend<'b, Renderer>>)>;

pub struct Chart<'a, Message, P = Attributes, Theme = iced::Theme, Renderer = iced::Renderer>
where
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    program: P,
    width: Length,
    height: Length,
    shaping: Shaping,
    cache: Option<&'a geometry::Cache<Renderer>>,
    message_: PhantomData<Message>,
    theme_: PhantomData<Theme>,
    renderer_: PhantomData<Renderer>,
}

impl<Message, Theme, Renderer> Chart<'_, Message, Attributes, Theme, Renderer>
where
    Renderer: geometry::Renderer,
    Attributes: Program<Message, Theme, Renderer>,
{
    pub fn new() -> Self {
        let program = Attributes::default();

        Self {
            program,
            width: Length::Fill,
            height: Length::Fill,
            shaping: Default::default(),
            cache: None,
            message_: PhantomData,
            theme_: PhantomData,
            renderer_: PhantomData,
        }
    }

    pub fn x_range(mut self, range: Range<f32>) -> Self {
        self.program.x_range = Some(range);

        self
    }

    pub fn y_range(mut self, range: Range<f32>) -> Self {
        self.program.y_range = Some(range);

        self
    }

    pub fn push_series(mut self, series: impl Into<Series>) -> Self {
        let series = series.into();

        let x_min_cur = self
            .program
            .x_range
            .as_ref()
            .map_or(f32::INFINITY, |range| range.start);
        let x_max_cur = self
            .program
            .x_range
            .as_ref()
            .map_or(f32::NEG_INFINITY, |range| range.end);
        let y_min_cur = self
            .program
            .y_range
            .as_ref()
            .map_or(f32::INFINITY, |range| range.start);
        let y_max_cur = self
            .program
            .y_range
            .as_ref()
            .map_or(f32::NEG_INFINITY, |range| range.end);

        let (x_min, x_max, y_min, y_max) = {
            let iter = match &series {
                Series::Line(line_series) => line_series.data.iter(),
                Series::Point(point_series) => point_series.data.iter(),
            };

            iter.fold(
                (x_min_cur, x_max_cur, y_min_cur, y_max_cur),
                |(x_min, x_max, y_min, y_max), (cur_x, cur_y)| {
                    (
                        x_min.min(*cur_x),
                        x_max.max(*cur_x),
                        y_min.min(*cur_y),
                        y_max.max(*cur_y),
                    )
                },
            )
        };

        self.program.x_range = Some(x_min..x_max);
        self.program.y_range = Some(y_min..y_max);

        self.program.series.push(series);

        self
    }

    pub fn extend_series(self, series_list: impl IntoIterator<Item = Series> + Clone) -> Self {
        series_list.into_iter().fold(self, Self::push_series)
    }
}

impl<'a, Message, P, Theme, Renderer> Chart<'a, Message, P, Theme, Renderer>
where
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    pub fn from_program(program: P) -> Self {
        Self {
            program,
            width: Length::Fill,
            height: Length::Fill,
            shaping: Default::default(),
            cache: None,
            message_: PhantomData,
            theme_: PhantomData,
            renderer_: PhantomData,
        }
    }

    /// set width
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// set height
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// set text shaping
    pub fn text_shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    pub fn with_cache(mut self, cache: &'a geometry::Cache<Renderer>) -> Self {
        self.cache = Some(cache);
        self
    }
}

impl<P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Chart<'_, Message, P, Theme, Renderer>
where
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<P::State>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(P::State::default())
    }

    #[inline]
    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        layout::Node::new(size)
    }

    #[inline]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state = tree.state.downcast_ref::<P::State>();

        let geometry = if let Some(cache) = &self.cache {
            cache.draw(renderer, bounds.size(), |frame| {
                let root = IcedChartBackend::new(frame, self.shaping).into_drawing_area();
                let mut chart_builder = ChartBuilder::on(&root);

                self.program
                    .draw(state, &mut chart_builder, theme, bounds, cursor);

                root.present().unwrap();
            })
        } else {
            let mut frame = canvas::Frame::new(renderer, bounds.size());
            let root = IcedChartBackend::new(&mut frame, self.shaping).into_drawing_area();
            let mut chart_builder = ChartBuilder::on(&root);

            self.program
                .draw(state, &mut chart_builder, theme, bounds, cursor);

            root.present().unwrap();

            frame.into_geometry()
        };

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_geometry(geometry);
        });
    }

    #[inline]
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _rectangle: &Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();

        let chart_event = match event {
            iced::Event::Mouse(mouse_event) => Some(Event::Mouse(mouse_event)),
            iced::Event::Touch(touch_event) => Some(Event::Touch(touch_event)),
            iced::Event::Keyboard(keyboard_event) => Some(Event::Keyboard(keyboard_event)),
            iced::Event::Window(_) => None,
        };

        if let Some(chart_event) = chart_event {
            let state = tree.state.downcast_mut::<P::State>();

            let (event_status, message) = self.program.update(state, chart_event, bounds, cursor);

            if let Some(message) = message {
                shell.publish(message);
            }

            return event_status;
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        self.program.mouse_interaction(state, bounds, cursor)
    }
}

impl<'a, Message, Theme, Renderer> Default for Chart<'a, Message, Attributes, Theme, Renderer>
where
    Renderer: 'a + geometry::Renderer,
    Attributes: Program<Message, Theme, Renderer>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, P, Message, Theme, Renderer> From<Chart<'a, Message, P, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + geometry::Renderer,
    P: 'a + Program<Message, Theme, Renderer>,
{
    fn from(
        chart: Chart<'a, Message, P, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(chart)
    }
}

#[derive(Clone, Default)]
pub struct Attributes {
    x_range: Option<Range<f32>>,
    y_range: Option<Range<f32>>,
    series: Vec<Series>,
}

impl Attributes {
    const X_RANGE_DEFAULT: Range<f32> = 0.0..10.0;
    const Y_RANGE_DEFAULT: Range<f32> = 0.0..10.0;
}

impl<Message> Program<Message> for Attributes {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        chart: &mut ChartBuilder<backend::IcedChartBackend<Renderer>>,
        theme: &iced::Theme,
        _bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) {
        let x_range = self
            .x_range
            .as_ref()
            .cloned()
            .unwrap_or(Attributes::X_RANGE_DEFAULT);

        let y_range = self
            .y_range
            .as_ref()
            .cloned()
            .unwrap_or(Attributes::Y_RANGE_DEFAULT);

        let mut chart = chart
            .x_label_area_size(10)
            .margin(20)
            .build_cartesian_2d(x_range, y_range)
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
            //.bold_line_style(GREEN.mix(0.1))
            //.light_line_style(BLUE.mix(0.1))
            .draw()
            .unwrap();

        for s in &self.series {
            match s {
                Series::Line(line_series) => {
                    chart
                        .draw_series(plotters::series::LineSeries::from(line_series))
                        .unwrap();
                }
                Series::Point(point_series) => {
                    chart
                        .draw_series(plotters::series::PointSeries::of_element(
                            point_series.data.iter().copied(),
                            5,
                            ShapeStyle::from(&RED).filled(),
                            &|coord, size, style| {
                                EmptyElement::at(coord) + Circle::new((0, 0), size, style)
                            },
                        ))
                        .unwrap();
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum Series {
    Line(LineSeries),
    Point(PointSeries),
}

#[derive(Clone)]
pub struct LineSeries {
    data: Vec<(f32, f32)>,
    color: Color,
}

impl LineSeries {
    pub fn new(iter: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Self {
            data: iter.into_iter().collect(),
            color: Color(iced::Color::BLACK),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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
    data: Vec<(f32, f32)>,
    color: Color,
}

impl PointSeries {
    pub fn new(iter: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Self {
            data: iter.into_iter().collect(),
            color: Color(iced::Color::BLACK),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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

impl<Backend> From<&LineSeries> for plotters::series::LineSeries<Backend, (f32, f32)>
where
    Backend: plotters::backend::DrawingBackend,
{
    fn from(series: &LineSeries) -> Self {
        let style: ShapeStyle = series.color.into();
        Self::new(series.data.clone(), style)
    }
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
