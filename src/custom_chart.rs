use core::f32;
use std::marker::PhantomData;
use std::ops::Range;

use crate::cartesian::Cartesian;
use crate::event::{self};
use crate::widget::Series;

use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout, mouse, renderer, Clipboard, Layout, Shell, Widget};
use iced::widget::canvas::path::lyon_path::geom::euclid::Transform2D;
use iced::widget::canvas::{self, Path, Stroke};
use iced::widget::text::Shaping;
use iced::{alignment, touch, Color, Font, Point, Renderer, Vector};
use iced::{mouse::Cursor, Element, Length, Rectangle, Size};

type StateFn<'a, Message> = Box<dyn Fn(&State) -> Message + 'a>;

pub struct Chart<'a, Message, Theme = iced::Theme>
where
    Message: Clone,
{
    width: Length,
    height: Length,
    shaping: Shaping,

    x_range: AxisRange<Range<f32>>,
    y_range: AxisRange<Range<f32>>,
    series: Vec<Series>,
    cache: canvas::Cache,

    on_move: Option<StateFn<'a, Message>>,
    on_scroll: Option<Box<dyn Fn(iced::Point, mouse::ScrollDelta, Cartesian) -> Message + 'a>>,
    on_press: Option<StateFn<'a, Message>>,
    on_release: Option<StateFn<'a, Message>>,
    //on_right_press: Option<Message>,
    //on_right_release: Option<Message>,
    //on_middle_press: Option<Message>,
    //on_middle_release: Option<Message>,
    //on_enter: Option<Message>,
    //on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    //on_exit: Option<Message>,
    //interaction: Option<mouse::Interaction>,
    theme_: PhantomData<Theme>,
}

impl<'a, Message, Theme> Chart<'a, Message, Theme>
where
    Message: Clone,
{
    const X_RANGE_DEFAULT: Range<f32> = 0.0..10.0;
    const Y_RANGE_DEFAULT: Range<f32> = 0.0..10.0;

    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            shaping: Shaping::default(),
            x_range: AxisRange::default(),
            y_range: AxisRange::default(),
            series: Vec::new(),
            cache: canvas::Cache::new(),
            on_move: None,
            on_scroll: None,
            on_press: None,
            on_release: None,
            theme_: PhantomData,
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

    //pub fn with_cache(mut self, cache: &'a canvas::Cache) -> Self {
    //    self.cache = cache;
    //    self
    //}

    pub fn x_range(mut self, range: Range<f32>) -> Self {
        self.x_range = AxisRange::Custom(range);

        self
    }

    pub fn y_range(mut self, range: Range<f32>) -> Self {
        self.y_range = AxisRange::Custom(range);

        self
    }

    pub fn push_series(mut self, series: impl Into<Series>) -> Self {
        let series = series.into();

        if let AxisRange::Automatic(x_range) = self.x_range {
            let x_min_cur = x_range.as_ref().map_or(f32::INFINITY, |range| range.start);
            let x_max_cur = x_range
                .as_ref()
                .map_or(f32::NEG_INFINITY, |range| range.end);

            let (x_min, x_max) = {
                let iter = match &series {
                    Series::Line(line_series) => line_series.data.iter(),
                    Series::Point(point_series) => point_series.data.iter(),
                };

                iter.fold((x_min_cur, x_max_cur), |(x_min, x_max), (cur_x, _)| {
                    (x_min.min(*cur_x), x_max.max(*cur_x))
                })
            };

            self.x_range = AxisRange::Automatic(Some(x_min..x_max));
        }

        if let AxisRange::Automatic(y_range) = self.y_range {
            let y_min_cur = y_range.as_ref().map_or(f32::INFINITY, |range| range.start);
            let y_max_cur = y_range
                .as_ref()
                .map_or(f32::NEG_INFINITY, |range| range.end);

            let (y_min, y_max) = {
                let iter = match &series {
                    Series::Line(line_series) => line_series.data.iter(),
                    Series::Point(point_series) => point_series.data.iter(),
                };

                iter.fold((y_min_cur, y_max_cur), |(y_min, y_max), (_, cur_y)| {
                    (y_min.min(*cur_y), y_max.max(*cur_y))
                })
            };

            self.y_range = AxisRange::Automatic(Some(y_min..y_max));
        }

        self.series.push(series);

        self
    }

    pub fn extend_series(
        self,
        series_list: impl IntoIterator<Item = impl Into<Series>> + Clone,
    ) -> Self {
        series_list.into_iter().fold(self, Self::push_series)
    }

    pub fn on_press(mut self, msg: impl Fn(&State) -> Message + 'a) -> Self {
        self.on_press = Some(Box::new(msg));
        self
    }

    pub fn on_release(mut self, msg: impl Fn(&State) -> Message + 'a) -> Self {
        self.on_release = Some(Box::new(msg));
        self
    }

    pub fn on_move(mut self, msg: impl Fn(&State) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(msg));
        self
    }

    pub fn on_scroll(
        mut self,
        msg: impl Fn(iced::Point, mouse::ScrollDelta, Cartesian) -> Message + 'a,
    ) -> Self {
        self.on_scroll = Some(Box::new(msg));
        self
    }
}

impl<Message, Theme> Widget<Message, Theme, Renderer> for Chart<'_, Message, Theme>
where
    Message: Clone,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![]
    }

    #[inline]
    fn layout(
        &self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        let state = tree.state.downcast_mut::<State>();

        let x_range = match &self.x_range {
            AxisRange::Custom(range) => range,
            AxisRange::Automatic(range) => range.as_ref().unwrap_or(&Self::X_RANGE_DEFAULT),
        };

        let y_range = match &self.y_range {
            AxisRange::Custom(range) => range,
            AxisRange::Automatic(range) => range.as_ref().unwrap_or(&Self::Y_RANGE_DEFAULT),
        };

        let node = layout::Node::new(size);

        state.x_range = Some(x_range.clone());
        state.y_range = Some(y_range.clone());
        state.bounds = node.bounds();

        node
    }

    #[inline]
    fn draw(
        &self,
        tree: &Tree,
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

        let state: &State = tree.state.downcast_ref();
        let x_range = state.x_range.as_ref().unwrap();
        let y_range = state.y_range.as_ref().unwrap();

        //let x_range = match &self.x_range {
        //    AxisRange::Custom(range) => range,
        //    AxisRange::Automatic(range) => range.as_ref().unwrap_or(&Self::X_RANGE_DEFAULT),
        //};

        //let y_range = match &self.y_range {
        //    AxisRange::Custom(range) => range,
        //    AxisRange::Automatic(range) => range.as_ref().unwrap_or(&Self::Y_RANGE_DEFAULT),
        //};

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let x_margin = 10f32;
            let y_margin = 20f32;

            let x_axis_length = -x_range.start + x_range.end;
            let y_axis_length = -y_range.start + y_range.end;
            let x_scale = (bounds.width - 2.0 * x_margin) / x_axis_length;
            let y_scale = (bounds.height - 2.0 * y_margin) / y_axis_length;

            frame.translate(Vector::new(x_margin, y_margin));
            frame.scale_nonuniform(Vector::new(x_scale, y_scale));
            frame.translate(Vector::new(-x_range.start, y_range.end));

            // x axis
            {
                let x_start = Point {
                    x: x_range.start,
                    y: 0.0,
                };
                let x_end = Point {
                    x: x_range.end,
                    y: 0.0,
                };

                frame.stroke(
                    &Path::line(x_start, x_end),
                    Stroke::default().with_width(1.0).with_color(Color::WHITE),
                );

                // ticks
                let ticks = 10usize;
                let tick_width = x_axis_length / ticks as f32;

                let mut draw_x_tick = |x| {
                    let tick_length = 5.0;
                    let tick_stroke_width = 2.0;

                    let half_tick_length = tick_length / 2.0;
                    let x_start = Point {
                        x,
                        //todo tick length
                        y: -half_tick_length / y_scale,
                    };
                    let x_end = Point {
                        x,
                        y: half_tick_length / y_scale,
                    };
                    frame.stroke(
                        &Path::line(x_start, x_end),
                        Stroke::default()
                            .with_width(tick_stroke_width)
                            .with_color(Color::WHITE),
                    );
                    frame.with_save(|frame| {
                        frame.scale_nonuniform(Vector::new(1.0 / x_scale, 1.0 / y_scale));
                        frame.fill_text(canvas::Text {
                            content: format!("{x}"),
                            size: 12.0.into(),
                            // TODO remove magic number,
                            position: Point {
                                x: x * x_scale,
                                y: 8.0,
                            },
                            color: Color::WHITE,
                            // TODO edge case center tick
                            horizontal_alignment: alignment::Horizontal::Center,
                            vertical_alignment: alignment::Vertical::Top,
                            font: Font::MONOSPACE,
                            ..canvas::Text::default()
                        });
                    })
                };

                let left = (x_range.start / tick_width).floor() as i32;
                for i in left..0 {
                    draw_x_tick(i as f32 * tick_width);
                }

                let right = (x_range.end / tick_width).ceil() as i32;
                for i in 0..=right {
                    draw_x_tick(i as f32 * tick_width);
                }
            }

            // y axis
            {
                let y_start = Point {
                    x: 0.0,
                    y: y_range.start,
                };
                let y_end = Point {
                    x: 0.0,
                    y: y_range.end,
                };
                frame.stroke(
                    &Path::line(y_start, y_end)
                        .transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                    Stroke::default().with_width(1.0).with_color(Color::WHITE),
                );

                // ticks
                let ticks = 10usize;
                let tick_width = y_axis_length / ticks as f32;

                let mut draw_y_tick = |y| {
                    let tick_length = 5.0;
                    let tick_stroke_width = 2.0;

                    let half_tick_length = tick_length / 2.0;
                    let start = Point {
                        x: -half_tick_length / x_scale,
                        //todo tick length
                        y,
                    };
                    let end = Point {
                        x: half_tick_length / x_scale,
                        y,
                    };
                    frame.stroke(
                        &Path::line(start, end)
                            .transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                        Stroke::default()
                            .with_width(tick_stroke_width)
                            .with_color(Color::WHITE),
                    );
                    frame.with_save(|frame| {
                        frame.scale_nonuniform(Vector::new(1.0 / x_scale, 1.0 / y_scale));
                        frame.fill_text(canvas::Text {
                            content: format!("{y}"),
                            size: 12.0.into(),
                            // TODO remove magic number,
                            position: Point {
                                x: -5.0,
                                y: -y * y_scale + 2.5,
                            },
                            color: Color::WHITE,
                            // TODO edge case center tick
                            horizontal_alignment: alignment::Horizontal::Right,
                            vertical_alignment: alignment::Vertical::Center,
                            font: Font::MONOSPACE,
                            ..canvas::Text::default()
                        })
                    })
                };

                let down = (y_range.start / tick_width).floor() as i32;
                for i in down..0 {
                    draw_y_tick(i as f32 * tick_width);
                }

                let up = (y_range.end / tick_width).ceil() as i32;
                for i in 1..=up {
                    draw_y_tick(i as f32 * tick_width);
                }
            }

            // series
            for series in &self.series {
                let path = match series {
                    Series::Line(line_series) => {
                        let mut iter = line_series.data.iter();
                        let path = Path::new(|b| {
                            if let Some(p) = iter.next() {
                                b.move_to(Point { x: p.0, y: p.1 });
                                iter.fold(b, |acc, p| {
                                    acc.line_to(Point { x: p.0, y: p.1 });
                                    acc
                                });
                            }
                        });
                        Some((path, line_series.color))
                    }
                    Series::Point(point_series) => {
                        let radius = 2.0;
                        let mut iter = point_series.data.iter();

                        let path = Path::new(|b| {
                            if let Some(p) = iter.next() {
                                let point = Point { x: p.0, y: p.1 };
                                b.circle(point, radius);
                                iter.fold(b, |acc, p| {
                                    let point = Point { x: p.0, y: p.1 };
                                    acc.circle(point, radius);
                                    acc
                                });
                            }
                        })
                        .transform(&Transform2D::new(
                            radius / x_scale,
                            0.0,
                            0.0,
                            radius / y_scale,
                            0.0,
                            0.0,
                        ));
                        Some((path, point_series.color))
                    }
                };

                if let Some((path, color)) = path {
                    frame.stroke(
                        &path.transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                        Stroke::default().with_width(2.0).with_color(color.0),
                    );
                }
            }
        });

        renderer.draw_geometry(geometry);
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
        let Some(cursor_position) = cursor.position() else {
            return event::Status::Ignored;
        };

        let state: &mut State = tree.state.downcast_mut();
        state.prev_position = state.cursor_position;
        state.cursor_position = Some(cursor_position);

        let bounds = layout.bounds();

        //if state.cursor_position != cursor_position || state.bounds != bounds {
        if bounds.contains(cursor_position) {
            if let Some(message) = self.on_press.as_ref() {
                if let iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | iced::Event::Touch(touch::Event::FingerPressed { .. }) = event
                {
                    shell.publish(message(state));

                    return event::Status::Captured;
                }
            }

            if let Some(message) = self.on_release.as_ref() {
                if let iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | iced::Event::Touch(touch::Event::FingerLifted { .. }) = event
                {
                    shell.publish(message(state));

                    return event::Status::Captured;
                }
            }

            if let Some(message) = self.on_move.as_ref() {
                if let iced::Event::Mouse(mouse::Event::CursorMoved { .. })
                | iced::Event::Touch(touch::Event::FingerMoved { .. }) = event
                {
                    shell.publish(message(state));

                    return event::Status::Captured;
                }
            }
        }
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }
}

/// Local state of the [`Chart`].
#[derive(Default)]
pub struct State {
    //is_hovered: bool,
    bounds: Rectangle,
    prev_position: Option<Point>,
    cursor_position: Option<Point>,
    x_range: Option<Range<f32>>,
    y_range: Option<Range<f32>>,
}

impl State {
    pub fn get_cursor_position(&self) -> Option<Point> {
        self.cursor_position
    }

    fn get_cartesian(&self, point: Point) -> Option<Point> {
        let x_margin = 10.0;
        let y_margin = 20.0;

        let x_range = self.x_range.clone()?;
        let y_range = self.y_range.clone()?;
        let bounds = self.bounds;

        let x_axis_length = -x_range.start + x_range.end;
        let y_axis_length = -y_range.start + y_range.end;
        let x_scale = (bounds.width - 2.0 * x_margin) / x_axis_length;
        let y_scale = (bounds.height - 2.0 * y_margin) / y_axis_length;

        let mut point = point * iced::Transformation::translate(-x_margin, -y_margin);
        point.x *= 1.0 / x_scale;
        point.y *= 1.0 / y_scale;
        let mut point = point * iced::Transformation::translate(x_range.start, -y_range.end);
        point.y = -point.y;
        Some(point)
    }

    pub fn get_coords(&self) -> Option<Point> {
        self.get_cartesian(self.cursor_position?)
    }

    pub fn get_offset(&self) -> Option<Point> {
        let cur = self.get_cartesian(self.cursor_position?)?;
        let x_range = self.x_range.clone()?;
        let y_range = self.y_range.clone()?;
        let x_axis_length = -x_range.start + x_range.end;
        let y_axis_length = -y_range.start + y_range.end;

        Some(Point::new(
            cur.x - (x_range.start + x_axis_length / 2.0),
            cur.y - (y_range.start + y_axis_length / 2.0),
        ))
    }
}

impl<'a, Message, Theme> From<Chart<'a, Message, Theme>> for Element<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a,
{
    fn from(chart: Chart<'a, Message, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(chart)
    }
}

#[derive(Clone)]
pub enum AxisRange<T> {
    Custom(T),
    Automatic(Option<T>),
}

impl<T> Default for AxisRange<T> {
    fn default() -> Self {
        Self::Automatic(None)
    }
}
