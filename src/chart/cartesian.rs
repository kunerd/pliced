use std::{f32, ops::RangeInclusive};

use iced::widget::canvas::path::lyon_path::geom::euclid::Transform2D;

pub struct Plane {
    pub x: Axis,
    pub y: Axis,
}

impl Plane {
    pub fn bottom_center(&self) -> iced::Point {
        iced::Point {
            x: 0.0,
            y: self.y.min,
        }
    }

    pub fn top_center(&self) -> iced::Point {
        iced::Point {
            x: 0.0,
            y: self.y.max,
        }
    }

    pub fn bottom_left(&self) -> iced::Point {
        iced::Point {
            x: self.x.min,
            y: 0.0,
        }
    }

    pub fn bottom_right(&self) -> iced::Point {
        iced::Point {
            x: self.x.max,
            y: 0.0,
        }
    }

    pub fn scale_to_cartesian_x(&self, value: f32) -> f32 {
        let mut result = value - self.x.min;
        result *= self.x.scale;
        result += self.x.margin_min;

        result
    }

    pub fn scale_to_cartesian_y(&self, value: f32) -> f32 {
        let mut result = -value + self.y.max;
        result *= self.y.scale;
        result += self.y.margin_max;

        result
    }

    pub fn get_cartesian(&self, pos: iced::Point) -> iced::Point {
        let x_margin = self.x.margin_min + self.x.margin_max;
        let y_margin = self.y.margin_min + self.y.margin_max;

        let mut point = pos * iced::Transformation::translate(-x_margin, -y_margin);
        point.x /= self.x.scale;
        point.y /= self.y.scale;
        let mut point = point * iced::Transformation::translate(self.x.min, -self.y.max);
        point.y = -point.y;

        point
    }

    pub fn get_offset(&self, pos: iced::Point) -> iced::Point {
        let pos = self.get_cartesian(pos);

        iced::Point::new(
            pos.x - (self.x.min + self.x.length / 2.0),
            pos.y - (self.y.min + self.y.length / 2.0),
        )
    }
}

pub struct Axis {
    pub length: f32,
    pub scale: f32,
    pub margin_min: f32,
    pub margin_max: f32,
    pub min: f32,
    pub max: f32,
}

impl Axis {
    pub fn new(range: &RangeInclusive<f32>, margin_min: f32, margin_max: f32, width: f32) -> Self {
        let length = -range.start() + range.end();
        let margin = margin(margin_min, margin_max);
        let scale = (width - margin) / length;

        let min = *range.start();
        let max = *range.end();

        Self {
            length,
            scale,
            margin_min,
            margin_max,
            min,
            max,
        }
    }
}

fn margin(min: f32, max: f32) -> f32 {
    min + max
}

//#[cfg(test)]
//mod test {
//    use super::*;
//
//    #[test]
//    fn length_all_positive() {
//        let axis = Axis::default().range(1.0..=5.0);
//        assert!((axis.length() - 4.0) <= f32::EPSILON)
//    }
//
//    #[test]
//    fn length_all_negative() {
//        let axis = Axis::default().range(-1.0..=-5.0);
//        assert!((axis.length() - 4.0) <= f32::EPSILON)
//    }
//
//    #[test]
//    fn length_negative_and_positive() {
//        let axis = Axis::default().range(-1.0..=5.0);
//        assert!((axis.length() - 6.0) <= f32::EPSILON)
//    }
//}

//            // x axis
//            {
//                let x_start = Point {
//                    x: x_range.start,
//                    y: 0.0,
//                };
//                let x_end = Point {
//                    x: x_range.end,
//                    y: 0.0,
//                };
//
//                frame.stroke(
//                    &Path::line(x_start, x_end),
//                    Stroke::default().with_width(1.0).with_color(Color::WHITE),
//                );
//
//                // ticks
//                let ticks = 10usize;
//                let tick_width = x_axis_length / ticks as f32;
//
//                let mut draw_x_tick = |x| {
//                    let tick_length = 5.0;
//                    let tick_stroke_width = 2.0;
//
//                    let half_tick_length = tick_length / 2.0;
//                    let x_start = Point {
//                        x,
//                        //todo tick length
//                        y: -half_tick_length / y_scale,
//                    };
//                    let x_end = Point {
//                        x,
//                        y: half_tick_length / y_scale,
//                    };
//                    frame.stroke(
//                        &Path::line(x_start, x_end),
//                        Stroke::default()
//                            .with_width(tick_stroke_width)
//                            .with_color(Color::WHITE),
//                    );
//                    frame.with_save(|frame| {
//                        frame.scale_nonuniform(Vector::new(1.0 / x_scale, 1.0 / y_scale));
//                        frame.fill_text(canvas::Text {
//                            content: format!("{x}"),
//                            size: 12.0.into(),
//                            // TODO remove magic number,
//                            position: Point {
//                                x: x * x_scale,
//                                y: 8.0,
//                            },
//                            color: Color::WHITE,
//                            // TODO edge case center tick
//                            horizontal_alignment: alignment::Horizontal::Center,
//                            vertical_alignment: alignment::Vertical::Top,
//                            font: Font::MONOSPACE,
//                            ..canvas::Text::default()
//                        });
//                    })
//                };
//
//                let left = (x_range.start / tick_width).floor() as i32;
//                for i in left..0 {
//                    draw_x_tick(i as f32 * tick_width);
//                }
//
//                let right = (x_range.end / tick_width).ceil() as i32;
//                for i in 0..=right {
//                    draw_x_tick(i as f32 * tick_width);
//                }
//            }
