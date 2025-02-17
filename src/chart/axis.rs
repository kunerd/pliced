use std::{f32, ops::RangeInclusive};

#[derive(Debug, Clone)]
pub struct Axis {
    range: RangeInclusive<f32>,
}

#[derive(Debug, Clone)]
pub struct Tick {
    length: f32,
    width: f32,
    color: iced::Color,
}

impl Axis {
    pub const DEFAULT_RANGE: RangeInclusive<f32> = 0.0..=10.0;

    pub fn length(&self) -> f32 {
        -self.range.start() + self.range.end()
    }

    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;

        self
    }
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            range: Self::DEFAULT_RANGE,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn length_all_positive() {
        let axis = Axis::default().range(1.0..=5.0);
        assert!((axis.length() - 4.0) <= f32::EPSILON)
    }

    #[test]
    fn length_all_negative() {
        let axis = Axis::default().range(-1.0..=-5.0);
        assert!((axis.length() - 4.0) <= f32::EPSILON)
    }

    #[test]
    fn length_negative_and_positive() {
        let axis = Axis::default().range(-1.0..=5.0);
        assert!((axis.length() - 6.0) <= f32::EPSILON)
    }
}

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
