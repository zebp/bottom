use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use indexmap::IndexMap;
use itertools::izip;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::{BaseWidget, ClickHandler, KeyHandler, ScrollHandler};

pub struct Container<B>
where
    B: Backend,
{
    /// The children of the container and their corresponding constraints.
    children: IndexMap<u16, (Box<dyn BaseWidget<B>>, Constraint)>,

    /// The widget ID of the container.
    widget_id: u16,

    /// The bounds of the container.
    draw_bounds: Rect,

    /// Which direction to align children in.
    direction: Direction,

    /// The margins between the children of the container.
    child_margin: u16,
}

impl<B> Container<B>
where
    B: Backend,
{
    /// Creates a new container.
    pub fn new_container(
        direction: Direction,
        children: IndexMap<u16, (Box<dyn BaseWidget<B>>, Constraint)>,
        widget_id: u16,
        children_margin: u16,
        //horizontal_alignment: Alignment, vertical_alignment: Alignment,
    ) -> Self {
        Container {
            children,
            widget_id,
            draw_bounds: Rect::default(),
            direction,
            child_margin: children_margin,
            // horizontal_alignment,
            // vertical_alignment,
        }
    }

    /// Creates a new row container (children are horizontally separated).

    pub fn new_row(
        children: IndexMap<u16, (Box<dyn BaseWidget<B>>, Constraint)>,
        widget_id: u16,
        children_margin: u16,
        // horizontal_alignment: Alignment, vertical_alignment: Alignment,
    ) -> Self {
        Self::new_container(
            Direction::Horizontal,
            children,
            widget_id,
            children_margin,
            // horizontal_alignment,
            // vertical_alignment,
        )
    }

    /// Creates a new column container (children are vertically separated).
    pub fn new_column(
        children: IndexMap<u16, (Box<dyn BaseWidget<B>>, Constraint)>,
        widget_id: u16,
        children_margin: u16,
        // horizontal_alignment: Alignment, vertical_alignment: Alignment,
    ) -> Self {
        Self::new_container(
            Direction::Vertical,
            children,
            widget_id,
            children_margin,
            // horizontal_alignment,
            // vertical_alignment,
        )
    }

    /// Updates the bounds of the container with new constraints.
    pub fn update_constraints(&mut self, new_constraints: &[Constraint]) {
        let new_bounds = {
            let layout = Layout::default()
                .direction(self.direction.clone())
                .constraints(new_constraints);

            match self.direction {
                Direction::Horizontal => layout.horizontal_margin(self.child_margin),
                Direction::Vertical => layout.vertical_margin(self.child_margin),
            }
        }
        .split(self.draw_bounds);

        izip!(&mut self.children, new_constraints, new_bounds).for_each(
            |((_child_id, child), new_constraint, new_bound)| {
                child.0.set_draw_bounds(new_bound);
                child.1 = *new_constraint;
            },
        );
    }

    /// Sets the children of a `Container`.
    pub fn set_children(
        mut self, new_children: IndexMap<u16, (Box<dyn BaseWidget<B>>, Constraint)>,
    ) -> Self {
        self.children = new_children;
        self.update_child_bounds();

        self
    }

    /// Adds a child and corresponding constraint to the end of the container, and updates the new bounds.
    pub fn add_child(&mut self, new_child: Box<dyn BaseWidget<B>>, new_constraint: Constraint) {
        self.children
            .insert(new_child.get_widget_id(), (new_child, new_constraint));
        self.update_child_bounds();
    }

    /// Updates the bounds of each child in the container given its current state.
    /// This should be called after any updates to either the container's own bounds or
    /// when adding a new child + constraint to the container.
    fn update_child_bounds(&mut self) {
        let new_bounds = {
            let layout = Layout::default()
                .direction(self.direction.clone())
                .constraints(
                    self.children
                        .iter()
                        .map(|(_child_id, (_child, constraint))| *constraint)
                        .collect::<Vec<_>>(),
                );

            match self.direction {
                Direction::Horizontal => layout.horizontal_margin(self.child_margin),
                Direction::Vertical => layout.vertical_margin(self.child_margin),
            }
        }
        .split(self.draw_bounds);

        self.children.iter_mut().zip(new_bounds).for_each(
            |((_child_id, (child, _constraint)), new_bound)| {
                child.set_draw_bounds(new_bound);
            },
        );
    }
}

impl<B> BaseWidget<B> for Container<B>
where
    B: Backend,
{
    fn draw(&mut self, frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        for (_child_id, (child, _constraint)) in &mut self.children {
            child.draw(frame);
        }
    }

    fn get_widget_id(&self) -> u16 {
        self.widget_id
    }

    fn set_draw_bounds(&mut self, new_bounds: Rect) {
        self.draw_bounds = new_bounds;

        self.update_child_bounds();
    }
}

impl<B> ClickHandler for Container<B>
where
    B: Backend,
{
    type SignalType = ();

    fn is_widget_in_bounds(&self, x: u16, y: u16) -> bool {
        x >= self.draw_bounds.x
            && x < self.draw_bounds.x + self.draw_bounds.width
            && y >= self.draw_bounds.y
            && y < self.draw_bounds.y + self.draw_bounds.height
    }

    fn on_left_click(&mut self, _x: u16, _y: u16) -> Option<Self::SignalType> {
        None
    }

    fn on_middle_click(&mut self, _x: u16, _y: u16) -> Option<Self::SignalType> {
        None
    }

    fn on_right_click(&mut self, _x: u16, _y: u16) -> Option<Self::SignalType> {
        None
    }
}

impl<B> ScrollHandler for Container<B>
where
    B: Backend,
{
    type SignalType = ();

    fn on_scroll(&mut self) -> Option<Self::SignalType> {
        // TODO: This
        None
    }
}

impl<B> KeyHandler for Container<B>
where
    B: Backend,
{
    type SignalType = ();

    fn on_key(&mut self, event: crossterm::event::KeyEvent) -> Option<Self::SignalType> {
        if event.modifiers.is_empty() {
            match event.code {
                _ => None,
            }
        } else {
            match event.modifiers {
                KeyModifiers::CONTROL | KeyModifiers::SHIFT => match event.code {
                    KeyCode::Left => {
                        // Try to move to the next widget in this direction;
                        // if we fail, then propagate back up and see if a parent `Container`
                        // can handle the movement.

                        None
                    }
                    KeyCode::Right => None,
                    KeyCode::Up => None,
                    KeyCode::Down => None,
                    KeyCode::Char(c) => {
                        // This is a workaround as in some cases, if you type in, say, a capital 'G',
                        // that's recorded as a shift + 'G', and not just 'G'.
                        // So, just recurse and call the `on_key` function with no modifier!
                        if event.modifiers == KeyModifiers::SHIFT {
                            self.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },

                _ => None,
            }
        }
    }
}
