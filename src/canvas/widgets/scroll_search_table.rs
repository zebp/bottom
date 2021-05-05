use std::borrow::Cow;

use indexmap::IndexMap;

use tui::{backend::Backend, layout::Constraint};

use crate::{
    app::AppState,
    canvas::components::{BaseWidget, Container, TableColumn, TextTable},
};

/// A scrollable and searchable `Container` that wraps a table, text input, and sort window.
/// Manages the table, column state, and sort state.
pub struct ScrollSearchTable<B>
where
    B: Backend,
{
    /// Whether to allow searching.
    is_searchable: bool,

    /// Whether to allow opening the sort menu.
    has_sort_menu: bool,

    /// Whether the search widget is open.
    is_search_open: bool,

    /// Whether the sort widget is open.
    is_sort_open: bool,

    /// The main wrapper `Container`.
    child: Container<B>,

    /// The stored data
    data: Vec<Vec<Cow<'static, str>>>,

    /// The columns
    columns: Vec<TableColumn>,

    /// Application state
    app_state: &'static AppState,
}

impl<B> ScrollSearchTable<B>
where
    B: Backend,
{
    /// Creates a new `ScrollSearchTable`.
    pub fn new(
        widget_id: u16, columns: Vec<TableColumn>, data: Vec<Vec<Cow<'static, str>>>,
        app_state: &'static AppState,
    ) -> Self {
        let row_container_children: IndexMap<u16, (Box<dyn BaseWidget<B>>, Constraint)> =
            IndexMap::new();

        let mut child = Container::new_row(row_container_children, widget_id, 1);

        let mut ss_table = ScrollSearchTable {
            is_searchable: true,
            has_sort_menu: true,
            is_search_open: false,
            is_sort_open: false,
            child,
            data,
            columns,
            app_state,
        };

        ss_table.child.add_child(
            Box::from(TextTable::new(widget_id + 3, &vec![], &vec![], app_state)),
            Constraint::Length(1),
        );

        ss_table
    }

    /// Sets whether the `ScrollSearchTable` is searchable.
    pub fn is_searchable(mut self, is_searchable: bool) -> Self {
        self.is_searchable = is_searchable;
        self
    }
}

impl<B> BaseWidget<B> for ScrollSearchTable<B>
where
    B: Backend,
{
    fn draw(&mut self, frame: &mut tui::Frame<'_, B>) {
        self.child.draw(frame);
    }

    fn get_widget_id(&self) -> u16 {
        self.child.get_widget_id()
    }

    fn set_draw_bounds(&mut self, new_bounds: tui::layout::Rect) {}

    fn get_name(&self) -> Option<Cow<'static, str>> {
        None
    }
}
