//! Handle events of a canvas.
use iced::keyboard;
use iced::mouse;
use iced::touch;

pub use iced::event::Status;

/// A [`Canvas`] event.
///
/// [`Canvas`]: crate::Canvas
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A mouse event.
    Mouse(mouse::Event),

    /// A touch event.
    Touch(touch::Event),

    /// A keyboard event.
    Keyboard(keyboard::Event),
}
