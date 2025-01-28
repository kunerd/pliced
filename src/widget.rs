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

/// Chart container, turns [`Chart`]s to [`Widget`]s
pub struct ChartWidget {
    series: Vec<f32>,
    width: Length,
    height: Length,
    shaping: Shaping,
}

impl ChartWidget {
    /// create a new [`ChartWidget`]
    pub fn new(series: Vec<f32>) -> Self {
        Self {
            series,
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

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ChartWidget
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
        let geometry = {
            {
                let x_range = 0..self.series.len();
                let y_min = *self.series.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
                let y_max = *self.series.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
                let y_range = y_min..y_max;

                let root = IcedChartBackend::new(&mut frame, self.shaping).into_drawing_area();
                let mut chart = ChartBuilder::on(&root)
                    .caption("y=x^2", ("sans-serif", 50).into_font())
                    .margin(5)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_cartesian_2d(x_range, y_range)
                    .unwrap();

                chart.configure_mesh().draw().unwrap();

                chart
                    .draw_series(LineSeries::new(
                        self.series.iter().cloned().enumerate(),
                        RED,
                    ))
                    .unwrap();

                chart
                    .configure_series_labels()
                    .background_style(WHITE.mix(0.8))
                    .border_style(BLACK)
                    .draw()
                    .unwrap();

                root.present().unwrap();
            }

            frame.into_geometry()
        };

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

impl<'a, Message, Theme, Renderer> From<ChartWidget> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer,
{
    fn from(widget: ChartWidget) -> Self {
        Element::new(widget)
    }
}
