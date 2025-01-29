// plotters-iced
//
// Iced backend for Plotters
// Copyright: 2022, Joylei <leingliu@gmail.com>
// License: MIT

use iced::advanced::graphics::geometry;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, mouse, renderer, Clipboard, Layout, Shell, Widget};
use iced::widget::canvas;
use iced::widget::text::Shaping;
use iced::Vector;
use iced::{mouse::Cursor, Element, Length, Rectangle, Size};
use plotters::prelude::*;

use crate::backend::IcedChartBackend;

pub type ChartBuilderFn<Renderer> =
    Box<dyn for<'a, 'b> Fn(&mut ChartBuilder<'a, 'b, IcedChartBackend<'b, Renderer>>)>;

/// Chart container, turns [`Chart`]s to [`Widget`]s
pub struct ChartWidget<Renderer>
where
    Renderer: geometry::Renderer,
{
    builder: ChartBuilderFn<Renderer>,
    width: Length,
    height: Length,
    shaping: Shaping,
}

impl<Renderer> ChartWidget<Renderer>
where
    Renderer: geometry::Renderer,
{
    /// create a new [`ChartWidget`]
    pub fn new(
        builder: impl for<'a, 'b> Fn(&mut ChartBuilder<'a, 'b, IcedChartBackend<'b, Renderer>>)
            + 'static,
    ) -> Self {
        Self {
            builder: Box::new(builder),
            width: Length::Fill,
            height: Length::Fill,
            shaping: Default::default(),
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
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ChartWidget<Renderer>
where
    Renderer: geometry::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    //fn tag(&self) -> tree::Tag {
    //    struct Tag<T>(T);
    //    tree::Tag::of::<Tag<C::State>>()
    //}

    //fn state(&self) -> tree::State {
    //    tree::State::new(C::State::default())
    //}

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
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let root = IcedChartBackend::new(&mut frame, self.shaping).into_drawing_area();
        let mut chart = ChartBuilder::on(&root);
        (self.builder)(&mut chart);
        root.present().unwrap();

        let geometry = frame.into_geometry();

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_geometry(geometry);
        });
    }

    #[inline]
    fn on_event(
        &mut self,
        _tree: &mut Tree,
        _event: iced::Event,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _rectangle: &Rectangle,
    ) -> iced::event::Status {
        //let bounds = layout.bounds();
        //let canvas_event = match event {
        //    iced::Event::Mouse(mouse_event) => Some(canvas::Event::Mouse(mouse_event)),
        //    iced::Event::Keyboard(keyboard_event) => Some(canvas::Event::Keyboard(keyboard_event)),
        //    _ => None,
        //};
        //if let Some(canvas_event) = canvas_event {

        //    let (event_status, message) = self.chart.update(state, canvas_event, bounds, cursor);

        //    if let Some(message) = message {
        //        shell.publish(message);
        //    }
        //    return event_status;
        //}
        iced::event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        //let bounds = layout.bounds();
        //self.chart.mouse_interaction(state, bounds, cursor)
        mouse::Interaction::None
    }
}

impl<'a, Message, Theme, Renderer> From<ChartWidget<Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + 'a,
{
    fn from(widget: ChartWidget<Renderer>) -> Self {
        Element::new(widget)
    }
}
