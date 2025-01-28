

// plotters-iced
//
// Iced backend for Plotters
// Copyright: 2022, Joylei <leingliu@gmail.com>
// License: MIT

use iced::advanced::{graphics::geometry, renderer, text, Layout};
use iced::widget::{
    canvas::{Cache, Frame, Geometry},
    text::Shaping,
};
use iced::{Size, Vector};
use plotters::prelude::DrawingArea;

use crate::backend::IcedChartBackend;

