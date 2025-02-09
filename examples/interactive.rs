extern crate pliced;

use std::{fmt::Debug, ops::Range};

use pliced::widget::{line_series, point_series, Chart};

use iced::{
    mouse::ScrollDelta,
    widget::{canvas, container},
    Element, Length, Point, Task,
};
use plotters::{
    coord::{types::RangedCoordf32, ReverseCoordTranslate},
    prelude::Cartesian2d,
};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view).run_with(App::new)
}

#[derive(Clone)]
enum Message {
    MousePressed,
    MouseWheelScrolled(
        Point,
        ScrollDelta,
        Cartesian2d<RangedCoordf32, RangedCoordf32>,
    ),
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MousePressed => write!(f, "MousePressed"),
            Self::MouseWheelScrolled(pos, delta, _cartesian) => f
                .debug_tuple("MouseWheelScrolled")
                .field(pos)
                .field(delta)
                .finish(),
        }
    }
}

#[derive(Debug, Default)]
struct App {
    data: Vec<(f32, f32)>,
    x_range: Range<f32>,
    cache: canvas::Cache,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = (-5..=5).map(|x| x as f32).map(|x| (x, x * x)).collect();

        let x_range = -5.0..5.0;

        (
            Self {
                data,
                x_range,
                ..Default::default()
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::MousePressed => {
                dbg!("Chart pressed");
            }
            Message::MouseWheelScrolled(pos, delta, coord) => {
                let ScrollDelta::Lines { x: _, y } = delta else {
                    return Task::none();
                };

                match y.is_sign_positive() {
                    true => self.zoom_in(pos, coord),
                    false => self.zoom_out(pos, coord),
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            Chart::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .x_range(self.x_range.clone())
                .with_cache(&self.cache)
                .push_series(
                    line_series(self.data.iter().copied()).color(iced::Color::from_rgb8(255, 0, 0)),
                )
                .push_series(
                    line_series(self.data.iter().copied().map(|(x, y)| (x, y * 0.5)))
                        .color(iced::Color::from_rgb8(0, 255, 0)),
                )
                .push_series(point_series(
                    self.data.iter().copied().map(|(x, y)| (x + 0.5, y * 2.0)),
                ))
                .on_press(Message::MousePressed)
                .on_scroll(Message::MouseWheelScrolled),
        )
        .into()
    }

    fn zoom_in(
        &mut self,
        mouse_pos: iced::Point,
        spec: Cartesian2d<RangedCoordf32, RangedCoordf32>,
    ) {
        let cur_pos = spec.reverse_translate((mouse_pos.x as i32, mouse_pos.y as i32));

        let Some((x, ..)) = cur_pos else {
            return;
        };

        let old_viewport = self.x_range.clone();
        let old_len = old_viewport.end - old_viewport.start;

        let center_scale: f32 = (x - old_viewport.start) / old_len;

        const ZOOM_FACTOR: f32 = 0.8;
        const LOWER_BOUND: f32 = 0.5;
        let mut new_len = old_len * ZOOM_FACTOR;
        if new_len < LOWER_BOUND {
            new_len = LOWER_BOUND;
        }

        let new_start = x - (new_len * center_scale);
        let new_end = new_start + new_len;
        self.x_range = new_start..new_end;
        self.cache.clear();
    }

    fn zoom_out(&mut self, p: iced::Point, spec: Cartesian2d<RangedCoordf32, RangedCoordf32>) {
        let cur_pos = spec.reverse_translate((p.x as i32, p.y as i32));

        let Some((x, ..)) = cur_pos else {
            return;
        };

        let old_viewport = self.x_range.clone();
        let old_len = old_viewport.end - old_viewport.start;

        let center_scale = (x - old_viewport.start) / old_len;

        const ZOOM_FACTOR: f32 = 1.2;
        let mut new_len = old_len * ZOOM_FACTOR;
        if new_len >= self.data.len() as f32 * 2.0 {
            new_len = self.data.len() as f32 * 2.0;
        }

        let new_start = x - (new_len * center_scale);
        let new_end = new_start + new_len;
        self.x_range = new_start..new_end;
        self.cache.clear();
    }
}
