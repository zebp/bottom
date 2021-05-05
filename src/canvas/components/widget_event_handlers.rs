use crossterm::event::KeyEvent;

pub trait KeyHandler {
    type SignalType;

    /// The handler for a key input in a widget.
    fn on_key(&mut self, event: KeyEvent) -> Option<Self::SignalType>;
}

pub trait ScrollHandler {
    type SignalType;

    /// The handler for scrolling in a widget.
    fn on_scroll(&mut self) -> Option<Self::SignalType>;
}

/// Handlers for clicking.
/// The handler functions are pre-defined to be functions that simply return `None`,
/// with functions for left, middle, and right click.  For the left, middle, and right click handlers,
/// they all accept x and y coordinates that are absolute.
///
/// Meanwhile, the `is_widget_in_bounds` function takes in an absolute coordinate and returns a boolean.
pub trait ClickHandler {
    type SignalType;

    /// Returns whether the widget is in the bounds of the given coordinate.
    /// Assumes absolute coordinates to the widget.
    fn is_widget_in_bounds(&self, x: u16, y: u16) -> bool;

    /// The handler for a left mouse click in a widget.  Assumes absolute coordinates to the widget.
    fn on_left_click(&mut self, _x: u16, _y: u16) -> Option<Self::SignalType> {
        None
    }

    /// The handler for a middle mouse click in a widget.  Assumes absolute coordinates to the widget.
    fn on_middle_click(&mut self, _x: u16, _y: u16) -> Option<Self::SignalType> {
        None
    }

    /// The handler for a right mouse click in a widget.  Assumes absolute coordinates to the widget.
    fn on_right_click(&mut self, _x: u16, _y: u16) -> Option<Self::SignalType> {
        None
    }
}
