use std::borrow::Cow;

use tui::{backend::Backend, layout::Rect, Frame};

pub trait BaseWidget<B>
where
    B: Backend,
{
    /// How a widget is to be drawn.
    fn draw(&mut self, frame: &mut Frame<'_, B>);

    /// Return the widget's ID.
    fn get_widget_id(&self) -> u16;

    /// Set new drawing bounds for a widget.
    fn set_draw_bounds(&mut self, new_bounds: Rect);

    /// Returns the name of the widget if it exists.  The default implementation returns `None`.
    fn get_name(&self) -> Option<Cow<'static, str>> {
        None
    }
}
