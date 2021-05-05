use std::convert::TryFrom;
use std::{borrow::Cow, cell::RefCell};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{app::AppState, constants::TABLE_GAP_HEIGHT_LIMIT};

use super::{
    widget_event_handlers::{ClickHandler, KeyHandler, ScrollHandler},
    BaseWidget,
};

/// Flexible constraints.
pub enum FlexConstraint {
    Length(u16),
    Percentage(f64),
}

pub struct Coordinate {
    pub x: u16,
    pub y: u16,
}

/// Signals to propagate back up from a table.
pub enum TableKeySignal {
    OpenSort,
    OpenSearch,
}

pub struct TableColumn {
    /// The desired width of the column.
    pub desired_width: u16,

    /// The desired upper flex bound.  If it is not present (`None`), then
    /// the column is assumed to be inflexible.
    pub upper_bound: Option<FlexConstraint>,

    /// The column header
    pub column_header: Cow<'static, str>,

    /// Whether this column is the column we are sorting by.
    pub is_sorting_column: bool,

    /// Whether this column is currently hidden.
    pub is_hidden: bool,

    /// The relative mouse x bounds of a column.  We don't store the y, since that's implicitly
    /// known.  Since the column may be hidden, the bounds are optional.
    pub x_bounds: Option<(u16, u16)>,
}

#[derive(Default)]
struct HorizontalScrollState {
    offset_multiplier: usize,
}

#[derive(Debug)]
pub enum ScrollDirection {
    /// Up means scrolling up, which decrements an index.
    Up,
    /// Down means scrolling down, which increments an index.
    Down,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Down
    }
}

#[derive(Default)]
struct VerticalScrollState {
    pub current_position: usize,
    pub previous_position: usize,
    pub scroll_direction: ScrollDirection,
}

enum TableWidthStrategy {
    MaxNumColumns,
    MaxColumnInfo,
}

pub struct TextTable<'d> {
    /// Representing the columns and headers of the table.  Each column contains its data.
    columns: &'d Vec<TableColumn>,

    /// Represents our processed and sorted data as per the table's state.
    data: &'d Vec<Vec<Cow<'static, str>>>,

    /// Represents the application's state.
    app_state: &'static AppState,

    /// Represents the drawing bounds of the table.
    draw_bounds: Rect,

    /// Represents the horizontal scrolling state.
    horizontal_state: HorizontalScrollState,

    /// Represents the vertical scrolling state.
    vertical_state: VerticalScrollState,

    /// The vertical start index from where to slice our data from.
    /// Determined by the vertical scroll state.
    vertical_start_index: usize,

    /// The vertical end index from where to stop the slice of our data from.
    /// Determined by the vertical scroll state.
    vertical_end_index: usize,

    /// Represents how column widths are calculated.
    width_strategy: TableWidthStrategy,

    /// Calculated column widths.
    column_widths: Vec<Constraint>,

    /// A constant offset to the table's actual height to account for the border and table gaps.
    table_height_offset: u16,

    /// The underlying tui-rs table state
    table_state: RefCell<TableState>,

    /// The widget's ID.
    widget_id: u16,

    /// The gap size between the table headers and data.  Overrides `table_gap`.
    table_offset: u16,

    /// The border type of the table.
    border_type: Borders,
}

impl<'d> TextTable<'d> {
    /// Creates a new `TextTable`.
    pub fn new(
        widget_id: u16, columns: &'d Vec<TableColumn>, data: &'d Vec<Vec<Cow<'static, str>>>,
        app_state: &'static AppState,
    ) -> Self {
        // TextTable {
        //     columns,
        //     data,
        //     app_state,
        //     draw_bounds: Rect::default(),
        //     horizontal_state: HorizontalScrollState::default(),
        //     vertical_state: VerticalScrollState::default(),
        //     vertical_start_index: 0,
        //     vertical_end_index: 0,
        //     width_strategy: TableWidthStrategy::MaxColumnInfo,
        //     column_widths,
        //     table_height_offset: 0,
        //     table_state: (),
        //     widget_id,
        //     table_offset: 0,
        //     given_table_gap: (),
        //     border_type: Borders::ALL,
        // }

        todo!()
    }

    /// This column width strategy takes into account either a given width, or a set of width bounds + desired width.
    /// It then determines how to best maximize the number of columns while still respecting the bounds.
    ///
    /// This is the old behaviour used before the widget system rewrite.
    fn get_column_widths_maximize_num_columns(&self) -> Vec<Constraint> {
        let mut total_width = self.draw_bounds.width;
        let mut bailed_early = false;
        let mut calculated_widths: Vec<u16> = vec![];

        vec![]
    }

    /// This column width strategy uses the maximal size of the column to calculate
    /// the column widths.  It's basically just a greedy algorithm.
    fn get_column_widths_maximize_column_info(&self) -> Vec<Constraint> {
        let mut total_width = self.draw_bounds.width;
        let mut bailed_early = false;
        let mut calculated_widths: Vec<u16> = vec![];

        if self.horizontal_state.offset_multiplier > 0 {
            // If there is any horizontal scrolling to the right,
            // enforce a one unit loss to the total available width and
            // set aside one column for the horizontal scroll marker.

            total_width -= 1;
            calculated_widths.push(1);
        }

        for column in self.columns {
            if !column.is_hidden {
                if total_width < column.desired_width {
                    // Darn, we can't add it.
                    bailed_early = true;
                    break;
                } else {
                    total_width -= column.desired_width;
                    calculated_widths.push(column.desired_width);
                }
            }
        }

        if bailed_early {
            // Basically redo the entire thing with a 1 pixel removal.  We can work with
            // a smaller set of column widths though.
            let mut new_total_width = self.draw_bounds.width - 1;
            let mut new_calculated_widths: Vec<u16> = vec![];

            if self.horizontal_state.offset_multiplier > 0 {
                new_total_width -= 1;
                new_calculated_widths.push(1);
            }

            for column_width in calculated_widths {
                if new_total_width < column_width {
                    // Stop adding.  Halt.
                    break;
                } else {
                    new_total_width -= column_width;
                    new_calculated_widths.push(column_width);
                }
            }

            new_calculated_widths.push(1);
            calculated_widths = new_calculated_widths;
            total_width = new_total_width;
        }

        // Now distribute any remaining space.
        let per_col_space =
            u16::try_from(usize::from(total_width) / calculated_widths.len()).unwrap_or(0);
        let mut remaining_col_space =
            u16::try_from(usize::from(total_width) % calculated_widths.len()).unwrap_or(0);

        for itx in 0..calculated_widths.len() {
            let remaining = if remaining_col_space > 0 {
                remaining_col_space -= 1;
                1
            } else {
                0
            };
            calculated_widths[itx] += per_col_space + remaining;
        }

        calculated_widths
            .into_iter()
            .map(|width| Constraint::Length(width))
            .collect()
    }

    /// Gets the starting index position of a vertically scrolled table.
    fn get_vertical_start_position(&mut self, num_rows: usize) {
        self.vertical_start_index = match self.vertical_state.scroll_direction {
            ScrollDirection::Down => {
                if self.vertical_state.current_position
                    < self.vertical_state.previous_position + num_rows
                {
                    // If, using previous_scrolled_position, we can see the element
                    // (so within that and + num_rows) just reuse the current previously scrolled position
                    self.vertical_state.previous_position
                } else if self.vertical_state.current_position >= num_rows {
                    // Else if the current position past the last element visible in the list, omit
                    // until we can see that element
                    self.vertical_state.previous_position =
                        self.vertical_state.current_position - num_rows;
                    self.vertical_state.previous_position
                } else {
                    // Else, if it is not past the last element visible, do not omit anything
                    0
                }
            }
            ScrollDirection::Up => {
                if self.vertical_state.current_position <= self.vertical_state.previous_position {
                    // If it's past the first element, then show from that element downwards
                    self.vertical_state.previous_position = self.vertical_state.current_position;
                } else if self.vertical_state.current_position
                    >= self.vertical_state.previous_position + num_rows
                {
                    self.vertical_state.previous_position =
                        self.vertical_state.current_position - num_rows;
                }
                // Else, don't change what our start position is from whatever it is set to!
                self.vertical_state.previous_position
            }
        };

        // TODO: Not sure if the upper bound is right...
        self.vertical_end_index = self.vertical_start_index + usize::from(self.draw_bounds.height)
            - usize::from(self.table_offset);
    }

    /// Update the stored data within the widget with newer data.
    ///
    /// Main thing here is re-sorting any data and updating any desired column widths,
    /// calculated column widths, etc.
    fn update_data(&mut self, new_data: &'d Vec<Vec<Cow<'static, str>>>) {
        self.data = new_data;

        // Update desired column widths
        self.column_widths = match self.width_strategy {
            TableWidthStrategy::MaxNumColumns => self.get_column_widths_maximize_num_columns(),
            TableWidthStrategy::MaxColumnInfo => self.get_column_widths_maximize_column_info(),
        };

        // Calculate column widths if needed and store for later use
    }
}

impl<'d, B> BaseWidget<B> for TextTable<'d>
where
    B: Backend,
{
    fn draw(&mut self, frame: &mut Frame<'_, B>) {
        // Note that self is mutable, but this is really not needed outside of managing
        // the state of tui's TableState.

        // Gather data as required, and put it into Rows.  We assume that this data is sorted as required.
        let gathered_data = {
            let sliced_rows = &self.data[self.vertical_start_index..self.vertical_end_index];

            sliced_rows
                .iter()
                .zip(&self.column_widths)
                .map(|(data_row, constraint)| {
                    Row::new(
                        data_row
                            .iter()
                            .zip(self.columns)
                            .filter_map(|(data, column)| {
                                if column.is_hidden {
                                    None
                                } else {
                                    if let Constraint::Length(length) = constraint {
                                        let graphemes =
                                            UnicodeSegmentation::graphemes(data.as_ref(), true)
                                                .collect::<Vec<&str>>();
                                        let mut truncated_data = String::default();
                                        let length_usize = usize::from(*length);

                                        for (itx, s) in graphemes.iter().enumerate() {
                                            if itx >= length_usize {
                                                break;
                                            }
                                            truncated_data.push_str(s);
                                        }

                                        Some(Cell::from(truncated_data))
                                    } else {
                                        Some(Cell::from(data.as_ref()))
                                    }
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        };

        // Get headers.
        let headers = Row::new(
            self.columns
                .iter()
                .map(|column| column.column_header.as_ref()),
        )
        .style(self.app_state.colours.table_header_style)
        .bottom_margin(self.table_offset);

        // Is this widget selected?  If so, use a selected border colour and highlight current entry.
        let (border_style, highlighted_entry_style) =
            if self.widget_id == self.app_state.selected_widget_id {
                (
                    self.app_state.colours.highlighted_border_style,
                    self.app_state.colours.currently_selected_text_style,
                )
            } else {
                (
                    self.app_state.colours.border_style,
                    self.app_state.colours.text_style,
                )
            };

        // The block
        let block = Block::default()
            .borders(self.border_type)
            .border_style(border_style);

        // And finally, draw.
        frame.render_stateful_widget(
            Table::new(gathered_data)
                .block(block)
                .highlight_style(highlighted_entry_style)
                .style(self.app_state.colours.text_style)
                .header(headers)
                .widths(&self.column_widths),
            self.draw_bounds,
            &mut self.table_state.borrow_mut(),
        );
    }

    fn get_widget_id(&self) -> u16 {
        self.widget_id
    }

    fn set_draw_bounds(&mut self, new_bounds: Rect) {
        if new_bounds != self.draw_bounds {
            self.draw_bounds = new_bounds;

            // Update table offset...
            self.table_offset = if self.draw_bounds.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                self.app_state.settings.table_gap
            };

            // Update click bounds of the table and the columns
        }
    }
}

impl<'d> KeyHandler for TextTable<'d> {
    type SignalType = TableKeySignal;

    fn on_key(&mut self, event: KeyEvent) -> Option<TableKeySignal> {
        if event.modifiers.is_empty() {
            match event.code {
                KeyCode::Char('/') => Some(TableKeySignal::OpenSearch),
                KeyCode::Char('g') => {
                    // TODO: Detect second 'g', if so, skip to the start of the list.
                    None
                }
                KeyCode::Char('G') => {
                    // Skip to end of the list.
                    self.vertical_state.current_position = self.data.len() - 1;
                    self.vertical_state.scroll_direction = ScrollDirection::Down;

                    None
                }
                KeyCode::F(6) => Some(TableKeySignal::OpenSort),
                KeyCode::Up | KeyCode::Char('k') => {
                    // Increment list
                    self.vertical_state.current_position =
                        self.vertical_state.current_position.saturating_sub(1);
                    self.vertical_state.scroll_direction = ScrollDirection::Up;

                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Decrement list
                    if self.vertical_state.current_position + 1 < self.data.len() {
                        self.vertical_state.current_position += 1;
                    }
                    self.vertical_state.scroll_direction = ScrollDirection::Down;

                    None
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    // Scroll left

                    self.horizontal_state.offset_multiplier =
                        self.horizontal_state.offset_multiplier.saturating_sub(1);

                    None
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    // Scroll right

                    if self.horizontal_state.offset_multiplier + 1 < self.columns.len() {
                        self.horizontal_state.offset_multiplier += 1;
                    }

                    None
                }
                _ => None,
            }
        } else {
            match event.modifiers {
                KeyModifiers::CONTROL => {
                    if let KeyCode::Char('f') = event.code {
                        Some(TableKeySignal::OpenSearch)
                    } else {
                        None
                    }
                }
                KeyModifiers::SHIFT => {
                    // This is a workaround as in some cases, if you type in, say, a capital 'G',
                    // that's recorded as a shift + 'G', and not just 'G'.
                    // So, just recurse and call the `on_key` function with no modifier!
                    if let KeyCode::Char(c) = event.code {
                        self.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
}

impl<'d> ScrollHandler for TextTable<'d> {
    type SignalType = ();

    fn on_scroll(&mut self) -> Option<()> {
        self.get_vertical_start_position(usize::from(
            (self.draw_bounds.height + (1 - self.table_offset))
                .saturating_sub(self.table_height_offset),
        ));

        None
    }
}

impl<'d> ClickHandler for TextTable<'d> {
    type SignalType = usize;

    fn on_left_click(&mut self, x: u16, y: u16) -> Option<usize> {
        // Click logic for a table.  This function assumes *absolute* x and y
        // coordinates to the displayed table!

        // Let's convert those absolute coordinates to *relative* coordinates.
        let relative_x = x - self.draw_bounds.x;
        let relative_y = y - self.draw_bounds.y;

        if relative_y == 0 {
            for (index, column) in self.columns.iter().enumerate() {
                if !column.is_hidden {
                    if let Some((left_x, right_x)) = column.x_bounds {
                        if relative_x >= left_x && relative_x < right_x {
                            // We assume this means we've clicked on the header.

                            Some(index);
                        }
                    }
                }
            }
        }

        None
    }

    fn is_widget_in_bounds(&self, x: u16, y: u16) -> bool {
        x >= self.draw_bounds.x
            && x < self.draw_bounds.x + self.draw_bounds.width
            && y >= self.draw_bounds.y
            && y < self.draw_bounds.y + self.draw_bounds.height
    }
}
