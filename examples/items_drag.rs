extern crate pliced;

use pliced::chart::{line_series, point_series, Chart, PointStyle};

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
    handles: Vec<Handle>,
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

#[derive(Debug, Clone)]
struct Handle {
    coords: (f32, f32),
    style: PointStyle,
}

impl Handle {
    fn new(coords: (f32, f32)) -> Self {
        Self {
            coords,
            style: PointStyle::default(),
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data: Vec<_> = [(0.0, 0.0), (1.0, 1.0), (2.0, 1.0), (3.0, 0.0)]
            .into_iter()
            .map(Handle::new)
            .collect();

        (
            Self {
                handles: data,
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
                if id.is_none() {
                    if let Some(handle) = self.hovered_item.and_then(|id| self.handles.get_mut(id))
                    {
                        handle.style = PointStyle::default()
                    }
                }

                self.hovered_item = id;

                let Some(pos) = pos else {
                    return Task::none();
                };

                match self.dragging {
                    Dragging::CouldStillBeClick(id, prev_pos) => {
                        if prev_pos == pos {
                            return Task::none();
                        } else {
                            if let Some(handle) = self.handles.get_mut(id) {
                                handle.coords.0 -= prev_pos.x - pos.x;
                            }
                            self.dragging = Dragging::ForSure(id, pos);
                        }
                    }
                    Dragging::ForSure(id, prev_pos) => {
                        if let Some(handle) = self.handles.get_mut(id) {
                            handle.coords.0 -= prev_pos.x - pos.x;
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
                    Dragging::CouldStillBeClick(id, _point) => {
                        if let Some(handle) = self.handles.get_mut(id) {
                            handle.style = PointStyle::default();
                        }
                        self.hovered_item = None;
                        self.dragging = Dragging::None;
                    }
                    Dragging::ForSure(id, prev_pos) => {
                        if let Some(handle) = self.handles.get_mut(id) {
                            handle.coords.0 -= prev_pos.x - pos.x;
                            handle.style = PointStyle::default();
                        }
                        self.dragging = Dragging::None;
                    }
                    Dragging::None => {}
                }
            }
        }

        let yellow: iced::Color = iced::Color::from_rgb8(238, 230, 0);
        let green: iced::Color = iced::Color::from_rgb8(50, 205, 50);

        match self.dragging {
            Dragging::CouldStillBeClick(id, _point) | Dragging::ForSure(id, _point) => {
                if let Some(handle) = self.handles.get_mut(id) {
                    handle.style = PointStyle {
                        color: Some(green),
                        radius: 10.0,
                        ..Default::default()
                    }
                }
            }
            Dragging::None => {
                if let Some(handle) = self.hovered_item.and_then(|id| self.handles.get_mut(id)) {
                    handle.style = PointStyle {
                        color: Some(yellow),
                        radius: 8.0,
                        ..Default::default()
                    }
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
                .x_range(-0.5..=3.5)
                .y_range(-0.5..=1.5)
                .push_series(line_series(self.handles.iter()).color(palette.primary))
                .push_series(
                    point_series(self.handles.iter())
                        .color(palette.danger)
                        .style(|handle| handle.style.clone())
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

impl From<&Handle> for (f32, f32) {
    fn from(handle: &Handle) -> Self {
        handle.coords
    }
}
