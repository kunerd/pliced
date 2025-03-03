extern crate pliced;

use pliced::chart::Chart;
use pliced::series::{line_series, point_series};

use iced::{widget::container, Element, Length, Task, Theme};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    OnMove(Option<usize>, Option<iced::Point>),
    MouseDown(Option<usize>, Option<iced::Point>),
    MouseUp(Option<iced::Point>),
}

#[derive(Debug)]
struct App {
    data: Vec<(f32, f32)>,
    hovered_item: Option<usize>,
    dragging: Dragging,
}

#[derive(Debug, Clone)]
enum ItemId {
    PointList,
}

#[derive(Debug, Default)]
enum Dragging {
    CouldStillBeClick(usize, iced::Point),
    ForSure(usize, iced::Point),
    #[default]
    None,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 1.0), (3.0, 0.0)];

        (
            Self {
                data,
                hovered_item: None,
                dragging: Dragging::None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::MouseDown(id, pos) => {
                let Dragging::None = self.dragging else {
                    return Task::none();
                };

                if let (Some(id), Some(pos)) = (id, pos) {
                    self.dragging = Dragging::CouldStillBeClick(id, pos);
                }
            }
            Message::OnMove(id, pos) => {
                self.hovered_item = id;

                let Some(pos) = pos else {
                    return Task::none();
                };

                match self.dragging {
                    Dragging::CouldStillBeClick(id, prev_pos) => {
                        if prev_pos == pos {
                            return Task::none();
                        } else {
                            if let Some(dp) = self.data.get_mut(id) {
                                dp.0 -= prev_pos.x - pos.x;
                            }
                            self.dragging = Dragging::ForSure(id, pos);
                        }
                    }
                    Dragging::ForSure(id, prev_pos) => {
                        if let Some(dp) = self.data.get_mut(id) {
                            dp.0 -= prev_pos.x - pos.x;
                        }
                        self.dragging = Dragging::ForSure(id, pos);
                    }
                    Dragging::None => {}
                }
            }
            Message::MouseUp(pos) => {
                let Some(pos) = pos else {
                    return Task::none();
                };
                match self.dragging {
                    Dragging::CouldStillBeClick(_id, _point) => {
                        self.hovered_item = None;
                        self.dragging = Dragging::None;
                    }
                    Dragging::ForSure(id, prev_pos) => {
                        if let Some(dp) = self.data.get_mut(id) {
                            dp.0 -= prev_pos.x - pos.x;
                        }
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
        let selected_item = match self.dragging {
            Dragging::CouldStillBeClick(id, _point) | Dragging::ForSure(id, _point) => Some(id),
            Dragging::None => self.hovered_item,
        };
        container(
            Chart::new()
                .width(Length::Fill)
                .height(Length::Fill)
                //.x_labels(Labels::default().format(&|v| format!("{v:.0}")))
                //.y_labels(Labels::default().format(&|v| format!("{v:.0}")))
                .x_range(-0.5..=3.5)
                .y_range(-0.5..=1.5)
                .push_series(line_series(self.data.iter().copied()).color(palette.primary))
                .push_series(
                    point_series(self.data.iter().copied())
                        .color(palette.danger)
                        .style(move |index| {
                            if Some(index) == selected_item {
                                10.0
                            } else {
                                4.0
                            }
                        })
                        .with_id(ItemId::PointList),
                )
                .on_press(|state: &pliced::chart::State<ItemId>| {
                    let id = state.items().and_then(|l| l.first().map(|i| i.1));
                    Message::MouseDown(id, state.get_offset())
                })
                .on_move(|state: &pliced::chart::State<ItemId>| {
                    let id = state.items().and_then(|l| l.first().map(|i| i.1));
                    Message::OnMove(id, state.get_offset())
                })
                .on_release(|state: &pliced::chart::State<ItemId>| {
                    Message::MouseUp(state.get_offset())
                }),
        )
        .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
