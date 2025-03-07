extern crate pliced;

use pliced::chart::{line_series, point_series, Chart};

use iced::{widget::container, Element, Length, Task, Theme};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    OnMove(Option<Vec<(ItemId, usize)>>),
}

#[derive(Debug)]
struct App {
    data: Vec<(f32, f32)>,
    selected_item: Option<usize>,
}

#[derive(Debug, Clone)]
enum ItemId {
    PointList,
}

//#[derive(Debug, Default)]
//enum Dragging {
//    CouldStillBeClick(iced::Point),
//    ForSure(iced::Point),
//    #[default]
//    None,
//}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 1.0), (3.0, 0.0)];

        (
            Self {
                data,
                selected_item: None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match &msg {
            Message::OnMove(Some(items)) => {
                self.selected_item = items.first().map(|(_, index)| *index)
            }
            _ => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let palette = self.theme().palette();
        let selected_item = self.selected_item;
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
                .on_move(|state: &pliced::chart::State<ItemId>| {
                    Message::OnMove(state.items().cloned())
                }),
        )
        .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
