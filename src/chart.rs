mod cartesian;

use crate::series::Series;
use core::f32;

use cartesian::Plane;
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::text::Renderer as _;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::Renderer as _;
use iced::advanced::{layout, mouse, renderer, Clipboard, Layout, Shell, Widget};
use iced::mouse::ScrollDelta;
use iced::widget::canvas::path::lyon_path::geom::euclid::Transform2D;
use iced::widget::canvas::{self, Path, Stroke};
use iced::widget::text::Shaping;
use iced::{alignment, event, touch, Font, Renderer, Vector};
use iced::{mouse::Cursor, Element, Length, Rectangle, Size};
use iced::{Pixels, Point};

use std::marker::PhantomData;
use std::ops::RangeInclusive;

type StateFn<'a, Message> = Box<dyn Fn(&State) -> Message + 'a>;

pub struct Chart<'a, Message, Theme = iced::Theme>
where
    Message: Clone,
{
    width: Length,
    height: Length,
    shaping: Shaping,

    margin: Margin,

    x_axis: Axis,
    y_axis: Axis,

    x_ticks: Ticks,
    y_ticks: Ticks,

    x_labels: Labels<'a>,
    y_labels: Labels<'a>,

    x_range: AxisRange<RangeInclusive<f32>>,
    y_range: AxisRange<RangeInclusive<f32>>,

    series: Vec<Series>,
    cache: canvas::Cache,

    on_move: Option<StateFn<'a, Message>>,
    on_press: Option<StateFn<'a, Message>>,
    on_release: Option<StateFn<'a, Message>>,
    on_scroll: Option<StateFn<'a, Message>>,
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
    const X_RANGE_DEFAULT: RangeInclusive<f32> = 0.0..=10.0;
    const Y_RANGE_DEFAULT: RangeInclusive<f32> = 0.0..=10.0;

    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            shaping: Shaping::default(),

            margin: Margin::default(),

            x_axis: Axis::default(),
            y_axis: Axis::default(),

            x_ticks: Ticks::default(),
            y_ticks: Ticks::default(),

            x_labels: Labels::default(),
            y_labels: Labels::default(),

            x_range: AxisRange::default(),
            y_range: AxisRange::default(),

            series: Vec::new(),
            cache: canvas::Cache::new(),
            on_move: None,
            on_press: None,
            on_release: None,
            on_scroll: None,
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

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn x_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.x_range = AxisRange::Custom(range);
        self
    }

    pub fn y_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.y_range = AxisRange::Custom(range);
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        self
    }

    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        self
    }

    pub fn x_ticks(mut self, ticks: Ticks) -> Self {
        self.x_ticks = ticks;
        self
    }

    pub fn y_ticks(mut self, ticks: Ticks) -> Self {
        self.y_ticks = ticks;
        self
    }

    pub fn x_labels(mut self, labels: Labels<'a>) -> Self {
        self.x_labels = labels;
        self
    }

    pub fn y_labels(mut self, labels: Labels<'a>) -> Self {
        self.y_labels = labels;
        self
    }

    pub fn push_series(mut self, series: impl Into<Series>) -> Self {
        let series = series.into();

        if let AxisRange::Automatic(x_range) = self.x_range {
            let x_min_cur = x_range
                .as_ref()
                .map_or(f32::INFINITY, |range| *range.start());
            let x_max_cur = x_range
                .as_ref()
                .map_or(f32::NEG_INFINITY, |range| *range.end());

            let (x_min, x_max) = {
                let iter = match &series {
                    Series::Line(line_series) => line_series.data.iter(),
                    Series::Point(point_series) => point_series.data.iter(),
                };

                iter.fold((x_min_cur, x_max_cur), |(x_min, x_max), (cur_x, _)| {
                    (x_min.min(*cur_x), x_max.max(*cur_x))
                })
            };

            self.x_range = AxisRange::Automatic(Some(x_min..=x_max));
        }

        if let AxisRange::Automatic(y_range) = self.y_range {
            let y_min_cur = y_range
                .as_ref()
                .map_or(f32::INFINITY, |range| *range.start());
            let y_max_cur = y_range
                .as_ref()
                .map_or(f32::NEG_INFINITY, |range| *range.end());

            let (y_min, y_max) = {
                let iter = match &series {
                    Series::Line(line_series) => line_series.data.iter(),
                    Series::Point(point_series) => point_series.data.iter(),
                };

                iter.fold((y_min_cur, y_max_cur), |(y_min, y_max), (_, cur_y)| {
                    (y_min.min(*cur_y), y_max.max(*cur_y))
                })
            };

            self.y_range = AxisRange::Automatic(Some(y_min..=y_max));
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

    pub fn on_scroll(mut self, msg: impl Fn(&State) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(msg));
        self
    }

    fn scale(&self, plane: &Plane, frame: &mut canvas::Frame) {
        frame.translate(Vector::new(self.margin.left, self.margin.top));
        frame.scale_nonuniform(Vector::new(plane.x.scale, plane.y.scale));
        frame.translate(Vector::new(-plane.x.min, plane.y.max));
    }

    fn draw_x_axis(&self, frame: &mut canvas::Frame, plane: &Plane) {
        frame.with_save(|frame| {
            self.scale(plane, frame);
            frame.stroke(
                &Path::line(plane.bottom_left(), plane.bottom_right()),
                Stroke::default()
                    .with_width(self.x_axis.width)
                    .with_color(self.x_axis.color),
            );
        });

        // ticks
        let tick_width = plane.x.length / self.x_ticks.amount as f32;
        let mut draw_x_tick = |x| {
            let x_scaled = plane.scale_to_cartesian_x(x);
            let y_scaled = plane.scale_to_cartesian_y(0.0);

            let half_tick_height = self.x_ticks.height / 2.0;
            let x_start = Point {
                x: x_scaled,
                y: y_scaled - half_tick_height,
            };
            let x_end = Point {
                x: x_scaled,
                y: y_scaled + half_tick_height,
            };

            frame.stroke(
                &Path::line(x_start, x_end),
                Stroke::default()
                    .with_width(self.x_ticks.width)
                    .with_color(self.x_ticks.color),
            );

            let label = self
                .x_labels
                .format
                .map_or_else(|| format!("{x}"), |fmt| fmt(&x));

            frame.fill_text(canvas::Text {
                content: label,
                size: self.x_labels.font_size.unwrap_or(12.into()),
                position: Point {
                    x: x_scaled,
                    // TODO remove magic number,
                    y: y_scaled + 8.0,
                },
                // TODO use theme
                color: self.x_labels.color.unwrap_or(iced::Color::WHITE),
                // TODO edge case center tick
                horizontal_alignment: alignment::Horizontal::Center,
                vertical_alignment: alignment::Vertical::Top,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });
        };

        let left = (plane.x.min / tick_width).ceil() as i32;
        for i in left..0 {
            draw_x_tick(i as f32 * tick_width);
        }

        let right = (plane.x.max / tick_width).floor() as i32;
        for i in 0..=right {
            draw_x_tick(i as f32 * tick_width);
        }
    }

    fn draw_y_axis(&self, frame: &mut canvas::Frame, plane: &Plane) {
        frame.with_save(|frame| {
            self.scale(plane, frame);
            frame.stroke(
                &Path::line(plane.bottom_center(), plane.top_center())
                    .transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                Stroke::default()
                    .with_width(self.y_axis.width)
                    .with_color(self.y_axis.color),
            );
        });

        let mut draw_y_tick = |y| {
            let x_scaled = plane.scale_to_cartesian_x(0.0);
            let y_scaled = plane.scale_to_cartesian_y(y);

            let half_tick_height = self.y_ticks.height / 2.0;
            let start = Point {
                x: x_scaled - half_tick_height,
                y: y_scaled,
            };
            let end = Point {
                x: x_scaled + half_tick_height,
                y: y_scaled,
            };

            frame.stroke(
                &Path::line(start, end).transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                Stroke::default()
                    .with_width(self.y_ticks.width)
                    .with_color(self.y_ticks.color),
            );

            let label = self
                .y_labels
                .format
                .map_or_else(|| format!("{y}"), |fmt| fmt(&y));

            frame.fill_text(canvas::Text {
                content: label,
                size: self.y_labels.font_size.unwrap_or(12.into()),
                position: Point {
                    // TODO remove magic number,
                    x: x_scaled - 8.0,
                    y: y_scaled,
                },
                // TODO use theme
                color: self.y_labels.color.unwrap_or(iced::Color::WHITE),
                // TODO edge case center tick
                horizontal_alignment: alignment::Horizontal::Right,
                vertical_alignment: alignment::Vertical::Center,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            })
        };

        let tick_width = plane.y.length / self.y_ticks.amount as f32;
        let down = (plane.y.min / tick_width).ceil() as i32;
        for i in down..0 {
            draw_y_tick(i as f32 * tick_width);
        }

        let up = (plane.y.max / tick_width).floor() as i32;
        for i in 1..=up {
            draw_y_tick(i as f32 * tick_width);
        }
    }

    fn draw_data(&self, frame: &mut canvas::Frame, plane: &Plane) {
        for series in &self.series {
            match series {
                Series::Line(line_series) => frame.with_save(|frame| {
                    self.scale(plane, frame);

                    let mut iter = line_series.data.iter().filter(|(x, y)| {
                        x >= &plane.x.min
                            && x <= &plane.x.max
                            && y >= &plane.y.min
                            && y <= &plane.y.max
                    });

                    let path = Path::new(|b| {
                        if let Some(p) = iter.next() {
                            b.move_to(Point { x: p.0, y: p.1 });
                            iter.fold(b, |acc, p| {
                                acc.line_to(Point { x: p.0, y: p.1 });
                                acc
                            });
                        }
                    });

                    frame.stroke(
                        &path.transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                        Stroke::default()
                            .with_width(2.0)
                            .with_color(line_series.color),
                    );
                }),

                Series::Point(point_series) => {
                    let radius = 4.0;
                    let mut iter = point_series.data.iter().filter(|(x, y)| {
                        x >= &plane.x.min
                            && x <= &plane.x.max
                            && y >= &plane.y.min
                            && y <= &plane.y.max
                    });

                    let path = Path::new(|b| {
                        if let Some(p) = iter.next() {
                            let point = Point {
                                x: plane.scale_to_cartesian_x(p.0),
                                y: plane.scale_to_cartesian_y(p.1),
                            };
                            b.circle(point, radius);
                            iter.fold(b, |acc, p| {
                                let point = Point {
                                    x: plane.scale_to_cartesian_x(p.0),
                                    y: plane.scale_to_cartesian_y(p.1),
                                };
                                acc.circle(point, radius);
                                acc
                            });
                        }
                    });
                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_width(2.0)
                            .with_color(point_series.color),
                    );
                }
            };
        }
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
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = layout::atomic(limits, self.width, self.height);
        //limits.resolve(self.width, self.height, Size::ZERO);

        let x_range = match &self.x_range {
            AxisRange::Custom(range) => range,
            AxisRange::Automatic(range) => range.as_ref().unwrap_or(&Self::X_RANGE_DEFAULT),
        };

        let y_range = match &self.y_range {
            AxisRange::Custom(range) => range,
            AxisRange::Automatic(range) => range.as_ref().unwrap_or(&Self::Y_RANGE_DEFAULT),
        };

        //let node = layout::Node::new(size);
        let bounds = node.bounds();

        let font_size: f32 = self
            .x_labels
            .font_size
            .unwrap_or_else(|| renderer.default_size())
            .into();

        let x_margin_min = self.margin.left;
        let x_margin_max = self.margin.right;
        let y_margin_min = self.margin.bottom + font_size;
        let y_margin_max = self.margin.top;

        let plane = Plane {
            x: cartesian::Axis::new(x_range, x_margin_min, x_margin_max, bounds.width),
            y: cartesian::Axis::new(y_range, y_margin_min, y_margin_max, bounds.height),
        };

        let state = tree.state.downcast_mut::<State>();
        state.plane = Some(plane);

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
        let Some(plane) = &state.plane else {
            return;
        };

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_data(frame, plane);
            self.draw_x_axis(frame, plane);
            self.draw_y_axis(frame, plane);
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_geometry(geometry)
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
        let Some(cursor_position) = cursor.position() else {
            return event::Status::Ignored;
        };

        let bounds = layout.bounds();

        let state: &mut State = tree.state.downcast_mut();
        let relative_position = cursor_position - Vector::new(bounds.x, bounds.y);
        state.prev_position = state.cursor_position;
        state.cursor_position = Some(relative_position);

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

            if let Some(message) = self.on_scroll.as_ref() {
                if let iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
                    state.scroll_delta = Some(delta);

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
    plane: Option<Plane>,
    prev_position: Option<Point>,
    cursor_position: Option<Point>,
    scroll_delta: Option<ScrollDelta>,
}

impl State {
    pub fn get_cursor_position(&self) -> Option<Point> {
        self.cursor_position
    }

    fn get_cartesian(&self, point: Point) -> Option<Point> {
        self.plane.as_ref().map(|p| p.get_cartesian(point))
    }

    pub fn get_coords(&self) -> Option<Point> {
        self.get_cartesian(self.cursor_position?)
    }

    pub fn get_offset(&self) -> Option<Point> {
        let pos = self.cursor_position?;

        self.plane.as_ref().map(|p| p.get_offset(pos))
    }

    pub fn scroll_delta(&self) -> Option<ScrollDelta> {
        self.scroll_delta
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

pub struct Ticks {
    color: iced::Color,
    height: f32,
    width: f32,
    amount: usize,
    //flip
    //noGrid
    //ints
    //times
    //limits: RangeInclusive<T>
}

impl Ticks {
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = color;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn amount(mut self, amount: usize) -> Self {
        self.amount = amount;
        self
    }
}

impl Default for Ticks {
    fn default() -> Self {
        Self {
            // use color from theme
            color: iced::Color::WHITE,
            height: 5.0,
            width: 1.0,
            amount: 10,
        }
    }
}

pub struct Axis {
    color: iced::Color,
    width: f32,
    // TODO limits
}

impl Axis {
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            // use color from theme
            color: iced::Color::WHITE,
            width: 1.0,
        }
    }
}

#[derive(Default)]
pub struct Labels<'a> {
    color: Option<iced::Color>,
    font_size: Option<iced::Pixels>,
    format: Option<&'a dyn Fn(&f32) -> String>, // TODO:
                                                // alignment
                                                // limits
                                                // uppercase    -- Make labels uppercase
                                                // rotate 90    -- Rotate labels

                                                // CA.alignRight   -- Anchor labels to the right
                                                // CA.alignLeft    -- Anchor labels to the left

                                                // CA.moveUp 5     -- Move 5 SVG units up
                                                // CA.moveDown 5   -- Move 5 SVG units down
                                                // CA.moveLeft 5   -- Move 5 SVG units left
                                                // CA.moveRight 5  -- Move 5 SVG units right

                                                // CA.amount 15   -- Change amount of ticks
                                                // , CA.flip        -- Flip to opposite direction
                                                // CA.withGrid    -- Add grid line by each label.

                                                // CA.ints            -- Add ticks at "nice" ints
                                                // CA.times Time.utc  -- Add ticks at "nice" times
}

impl<'a> Labels<'a> {
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn font_size(mut self, font_size: impl Into<Pixels>) -> Self {
        self.font_size = Some(font_size.into());
        self
    }

    pub fn format(mut self, format: &'a dyn Fn(&f32) -> String) -> Self {
        self.format = Some(format);
        self
    }
}

pub struct Margin {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Margin {
    const MARGIN_DEFAULT: f32 = 10.0;
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: Self::MARGIN_DEFAULT,
            bottom: Self::MARGIN_DEFAULT,
            left: Self::MARGIN_DEFAULT,
            right: Self::MARGIN_DEFAULT,
        }
    }
}
