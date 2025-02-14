extern crate pliced;

use std::default;
use std::ops::Range;

use iced::{Theme, Vector};
use pliced::custom_chart::Chart;
use pliced::widget::{line_series, point_series};

use iced::{widget::container, Element, Length, Task};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    OnMove(Option<iced::Point>),
    MouseDown(Option<iced::Point>),
    MouseUp(Option<iced::Point>),
}

#[derive(Debug, Default)]
struct App {
    x_range: Range<f32>,
    data: Vec<(f32, f32)>,
    dragging: Dragging,
}

#[derive(Debug, Default)]
enum Dragging {
    CouldStillBeClick(iced::Point),
    ForSure(iced::Point),
    #[default]
    None,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = (-50..=50)
            .map(|x| x as f32 / 50.0)
            .map(|x| (x, x * x))
            .collect();

        (
            Self {
                data,
                x_range: -1.0..1.0,
                dragging: Dragging::None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        let mut update_center = |prev_pos: iced::Point, pos: iced::Point| {
            let shift_x = prev_pos.x - pos.x;
            self.x_range.start += shift_x;
            self.x_range.end += shift_x;
        };
        match msg {
            Message::MouseDown(pos) => {
                let Dragging::None = self.dragging else {
                    return Task::none();
                };

                if let Some(pos) = pos {
                    self.dragging = Dragging::CouldStillBeClick(pos);
                }
            }
            Message::OnMove(pos) => {
                let Some(pos) = pos else {
                    dbg!("no pos: {:?}", &msg);
                    return Task::none();
                };

                match self.dragging {
                    Dragging::CouldStillBeClick(prev_pos) => {
                        if prev_pos == pos {
                            return Task::none();
                        } else {
                            update_center(prev_pos, pos);
                            self.dragging = Dragging::ForSure(pos);
                        }
                    }
                    Dragging::ForSure(prev_pos) => {
                        update_center(prev_pos, pos);
                        self.dragging = Dragging::ForSure(pos);
                    }
                    Dragging::None => {}
                }
            }
            Message::MouseUp(pos) => {
                let Some(pos) = pos else {
                    dbg!("no pos: {:?}", &msg);
                    return Task::none();
                };
                match self.dragging {
                    Dragging::CouldStillBeClick(_point) => {
                        self.dragging = Dragging::None;
                    }
                    Dragging::ForSure(prev_pos) => {
                        update_center(prev_pos, pos);
                        self.dragging = Dragging::None;
                    }
                    Dragging::None => {}
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let palette = self.theme().palette();
        container(
            Chart::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .x_range(self.x_range.clone())
                //.y_range(-1.0..1.0)
                .push_series(line_series(self.data.iter().copied()).color(palette.primary))
                .push_series(
                    line_series(self.data.iter().copied().map(|(x, y)| (x, y * 0.5)))
                        .color(palette.success),
                )
                //.push_series(
                //    point_series(self.data.iter().copied().map(|(x, y)| (x + 0.5, y * 2.0)))
                //        .color(palette.danger),
                //),
                .on_press(|state| Message::MouseDown(state.get_offset()))
                .on_release(|state| Message::MouseUp(state.get_offset()))
                .on_move(|state| Message::OnMove(state.get_offset())),
        )
        .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
