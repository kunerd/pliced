use std::marker::PhantomData;

use crate::backend::IcedChartBackend;
use crate::event::{self, Event};
use crate::program::Program;

use iced::advanced::graphics::geometry;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout, mouse, renderer, Clipboard, Layout, Shell, Widget};
use iced::widget::canvas;
use iced::widget::text::Shaping;
use iced::Vector;
use iced::{mouse::Cursor, Element, Length, Rectangle, Size};
use plotters::prelude::*;

pub type ChartBuilderFn<Renderer> =
    Box<dyn for<'a, 'b> Fn(&mut ChartBuilder<'a, 'b, IcedChartBackend<'b, Renderer>>)>;

pub struct ChartWidget<P, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    program: P,
    width: Length,
    height: Length,
    shaping: Shaping,
    message_: PhantomData<Message>,
    theme_: PhantomData<Theme>,
    renderer_: PhantomData<Renderer>,
}

impl<P, Message, Theme, Renderer> ChartWidget<P, Message, Theme, Renderer>
where
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    /// create a new [`ChartWidget`]
    pub fn new(program: P) -> Self {
        Self {
            program,
            width: Length::Fill,
            height: Length::Fill,
            shaping: Default::default(),
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
}

impl<P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ChartWidget<P, Message, Theme, Renderer>
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

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let root = IcedChartBackend::new(&mut frame, self.shaping).into_drawing_area();
        let mut chart_builder = ChartBuilder::on(&root);

        self.program
            .draw(state, &mut chart_builder, theme, bounds, cursor);
        root.present().unwrap();

        let geometry = frame.into_geometry();

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

impl<'a, P, Message, Theme, Renderer> From<ChartWidget<P, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + geometry::Renderer,
    P: 'a + Program<Message, Theme, Renderer>,
{
    fn from(
        canvas: ChartWidget<P, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(canvas)
    }
}
