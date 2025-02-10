use std::fmt::{Debug, Formatter};

use plotters::{
    coord::{types::RangedCoordf32, ReverseCoordTranslate},
    prelude::Cartesian2d,
};

#[derive(Clone)]
pub struct Cartesian(Cartesian2d<RangedCoordf32, RangedCoordf32>);

impl Cartesian {
    pub fn new(cartesian: Cartesian2d<RangedCoordf32, RangedCoordf32>) -> Self {
        Self(cartesian)
    }

    pub fn get_coords(&self, position: iced::Point) -> Option<iced::Point> {
        let plotters_position = (position.x as i32, position.y as i32);

        let cartesian_position = self.0.reverse_translate(plotters_position);

        cartesian_position.map(iced::Point::from)
    }
}

impl Debug for Cartesian {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x_range = self.0.get_x_range();
        let y_range = self.0.get_x_range();

        f.debug_tuple("Cartesian")
            .field(&x_range)
            .field(&y_range)
            .finish()
    }
}
