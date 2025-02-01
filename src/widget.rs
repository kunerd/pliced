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

impl<'a, Data, Message, Theme, Renderer> Chart<'a, Message, Attributes<Data>, Theme, Renderer>
where
    Renderer: geometry::Renderer,
    Attributes<Data>: Program<Message, Theme, Renderer>,
    Data: 'a + IntoIterator<Item = (f32, f32)> + Clone,
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

    pub fn series(mut self, series: impl IntoIterator<Item = Series<Data>> + Clone) -> Self {
        let series: Vec<_> = series.into_iter().collect();

        let (x_min, x_max, y_min, y_max) = series
            .iter()
            .filter_map(|series| {
                if let Series::Line { data, .. } = series {
                    Some(data.clone().into_iter())
                } else {
                    None
                }
            })
            .flatten()
            .fold(
                (
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                ),
                |(x_min, x_max, y_min, y_max), (cur_x, cur_y)| {
                    (
                        x_min.min(cur_x),
                        x_max.max(cur_x),
                        y_min.min(cur_y),
                        y_max.max(cur_y),
                    )
                },
            );

        self.program.x_range = x_min..x_max;
        self.program.y_range = y_min..y_max;

        self.program.series = series;

        self
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

#[derive(Clone)]
pub struct Attributes<Data = Vec<(f32, f32)>>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    x_range: Range<f32>,
    y_range: Range<f32>,
    series: Vec<Series<Data>>,
}

impl<Data> Default for Attributes<Data>
where
    Data: IntoIterator<Item = (f32, f32)> + Clone,
{
    fn default() -> Self {
        Self {
            x_range: 0.0..10.0,
            y_range: 0.0..10.0,
            series: Vec::new(),
        }
    }
}

impl<Message, Data> Program<Message> for Attributes<Data>
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
            //.bold_line_style(GREEN.mix(0.1))
            //.light_line_style(BLUE.mix(0.1))
            .draw()
            .unwrap();

        for s in &self.series {
            if let Series::Line { data, color } = s {
                let style: ShapeStyle = color.into();
                chart
                    .draw_series(LineSeries::new(data.clone(), style))
                    .unwrap();
            }
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
