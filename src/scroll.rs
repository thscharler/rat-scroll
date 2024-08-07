use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Style;
use ratatui::symbols::scrollbar::Set;
use ratatui::widgets::{
    Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, StatefulWidgetRef,
};
use std::cmp::{max, min};

/// Scrolling indicator.
///
/// This is not a widget by itself, rather it is meant to be used
/// analogous to Block. A widget that supports scrolling accepts
/// one or two of these Scroll indicators. The widget then uses this
/// struct to organize and render the scrollbars.
#[derive(Debug, Default, Clone)]
pub struct Scroll<'a> {
    policy: ScrollbarType,
    orientation: ScrollbarOrientation,
    start_margin: u16,
    end_margin: u16,
    overscroll_by: Option<usize>,
    scroll_by: Option<usize>,

    thumb_symbol: Option<&'a str>,
    thumb_style: Option<Style>,
    track_symbol: Option<&'a str>,
    track_style: Option<Style>,
    begin_symbol: Option<&'a str>,
    begin_style: Option<Style>,
    end_symbol: Option<&'a str>,
    end_style: Option<Style>,
    no_symbol: Option<&'a str>,
    no_style: Option<Style>,
}

/// Holds the Scrolled-State.
///
/// The current visible page is represented as the pair (offset, page_len).
/// The limit for scrolling is given as max_offset, which is the maximum offset
/// where a full page can still be displayed. Note that the total length of
/// the widgets data is NOT max_offset + page_len. The page_len can be different for
/// every offset selected. Only if the offset is set to max_offset and after
/// the next round of rendering len == max_offset + page_len will hold true.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollState {
    /// Area of the Scrollbar.
    pub area: Rect,
    /// Vertical/Horizontal scroll?
    pub orientation: ScrollbarOrientation,
    /// Current offset.
    pub offset: usize,
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub max_offset: usize,
    /// Page-size at the current offset.
    pub page_len: usize,

    /// Scrolling step-size for mouse-scrolling
    pub scroll_by: Option<usize>,
    /// Allow overscroll by n items.
    pub overscroll_by: Option<usize>,

    /// Mouse support.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

/// Scrollbar behaviour if no scrolling is needed.
///
/// If a widget has a scrollbar set, [layout_scroll] always reserves
/// space for the scrollbar. This makes calculating any layout much
/// easier. If the widget wants to allow switching of the scrollbar
/// completely it should use an `Option<Scroll>`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarType {
    /// Always renders the scrollbar recognizable as a scrollbar.
    Show,
    /// If the scrollbar is not needed, the area is filled with
    /// the no_symbol, but the required space is still reserved
    /// for the scrollbar.
    #[default]
    Minimal,
    /// If the scrollbar is not needed, the area is not rendered,
    /// but the area is still reserved for the scrollbar.
    /// The widget is responsible to render something in that
    /// area.
    /// This works fine if there is a regular border in place,
    /// otherwise filling it with the default style should do
    /// the trick as well.
    NoRender,
}

/// Collected styles for the Scroll.
#[derive(Debug, Clone)]
pub struct ScrollStyle {
    pub thumb_style: Option<Style>,
    pub track_symbol: Option<&'static str>,
    pub track_style: Option<Style>,
    pub begin_symbol: Option<&'static str>,
    pub begin_style: Option<Style>,
    pub end_symbol: Option<&'static str>,
    pub end_style: Option<Style>,
    /// Symbol used when the scrollbar doesn't display.
    pub no_symbol: Option<&'static str>,
    /// Style used when the scrollbar doesn't display.
    pub no_style: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Scroll<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scrollbar policy.
    pub fn scrollbar_type(mut self, policy: ScrollbarType) -> Self {
        self.policy = policy;
        self
    }

    /// Scrollbar orientation.
    pub fn orientation(mut self, pos: ScrollbarOrientation) -> Self {
        self.orientation = pos;
        self
    }

    /// Leave a margin at the start of the scrollbar.
    pub fn start_margin(mut self, start_margin: u16) -> Self {
        self.start_margin = start_margin;
        self
    }

    /// Leave a margin at the end of the scrollbar.
    pub fn end_margin(mut self, end_margin: u16) -> Self {
        self.end_margin = end_margin;
        self
    }

    /// Set overscrolling to this value.
    pub fn overscroll_by(mut self, overscroll: usize) -> Self {
        self.overscroll_by = Some(overscroll);
        self
    }

    /// Set scroll increment.
    pub fn scroll_by(mut self, scroll: usize) -> Self {
        self.scroll_by = Some(scroll);
        self
    }

    /// Ensures a vertical orientation.
    pub fn override_vertical(mut self) -> Self {
        self.orientation = match self.orientation {
            ScrollbarOrientation::VerticalRight => ScrollbarOrientation::VerticalRight,
            ScrollbarOrientation::VerticalLeft => ScrollbarOrientation::VerticalLeft,
            ScrollbarOrientation::HorizontalBottom => ScrollbarOrientation::VerticalRight,
            ScrollbarOrientation::HorizontalTop => ScrollbarOrientation::VerticalRight,
        };
        self
    }

    /// Ensures a horizontal orientation.
    pub fn override_horizontal(mut self) -> Self {
        self.orientation = match self.orientation {
            ScrollbarOrientation::VerticalRight => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::VerticalLeft => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::HorizontalBottom => ScrollbarOrientation::HorizontalBottom,
            ScrollbarOrientation::HorizontalTop => ScrollbarOrientation::HorizontalTop,
        };
        self
    }

    /// Is this a vertical scrollbar.
    pub fn is_vertical(&self) -> bool {
        match self.orientation {
            ScrollbarOrientation::VerticalRight => true,
            ScrollbarOrientation::VerticalLeft => true,
            ScrollbarOrientation::HorizontalBottom => false,
            ScrollbarOrientation::HorizontalTop => false,
        }
    }

    /// Is this a horizontal scrollbar.
    pub fn is_horizontal(&self) -> bool {
        match self.orientation {
            ScrollbarOrientation::VerticalRight => false,
            ScrollbarOrientation::VerticalLeft => false,
            ScrollbarOrientation::HorizontalBottom => true,
            ScrollbarOrientation::HorizontalTop => true,
        }
    }

    /// Set all styles.
    pub fn styles(mut self, styles: ScrollStyle) -> Self {
        self.thumb_style = styles.thumb_style;
        self.track_symbol = styles.track_symbol;
        self.track_style = styles.track_style;
        self.begin_symbol = styles.begin_symbol;
        self.begin_style = styles.begin_style;
        self.end_symbol = styles.end_symbol;
        self.end_style = styles.end_style;
        self.no_symbol = styles.no_symbol;
        self.no_style = styles.no_style;
        self
    }

    /// Symbol for the Scrollbar.
    pub fn thumb_symbol(mut self, thumb_symbol: &'a str) -> Self {
        self.thumb_symbol = Some(thumb_symbol);
        self
    }

    /// Style for the Scrollbar.
    pub fn thumb_style<S: Into<Style>>(mut self, thumb_style: S) -> Self {
        self.thumb_style = Some(thumb_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn track_symbol(mut self, track_symbol: Option<&'a str>) -> Self {
        self.track_symbol = track_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn track_style<S: Into<Style>>(mut self, track_style: S) -> Self {
        self.track_style = Some(track_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn begin_symbol(mut self, begin_symbol: Option<&'a str>) -> Self {
        self.begin_symbol = begin_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn begin_style<S: Into<Style>>(mut self, begin_style: S) -> Self {
        self.begin_style = Some(begin_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn end_symbol(mut self, end_symbol: Option<&'a str>) -> Self {
        self.end_symbol = end_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn end_style<S: Into<Style>>(mut self, end_style: S) -> Self {
        self.end_style = Some(end_style.into());
        self
    }

    /// Symbol when no Scrollbar is drawn.
    pub fn no_symbol(mut self, no_symbol: Option<&'a str>) -> Self {
        self.no_symbol = no_symbol;
        self
    }

    /// Style when no Scrollbar is drawn.
    pub fn no_style<S: Into<Style>>(mut self, no_style: S) -> Self {
        self.no_style = Some(no_style.into());
        self
    }

    /// Set all Scrollbar symbols.
    pub fn symbols(mut self, symbols: Set) -> Self {
        self.thumb_symbol = Some(symbols.thumb);
        if self.track_symbol.is_some() {
            self.track_symbol = Some(symbols.track);
        }
        if self.begin_symbol.is_some() {
            self.begin_symbol = Some(symbols.begin);
        }
        if self.end_symbol.is_some() {
            self.end_symbol = Some(symbols.end);
        }
        self
    }

    /// Set a style for all Scrollbar styles.
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        let style = style.into();
        self.track_style = Some(style);
        self.thumb_style = Some(style);
        self.begin_style = Some(style);
        self.end_style = Some(style);
        self.no_style = Some(style);
        self
    }
}

impl<'a> Scroll<'a> {
    // create the widget.
    fn scrollbar(&self) -> Scrollbar<'a> {
        let mut scrollbar = Scrollbar::new(self.orientation.clone());
        if let Some(thumb_symbol) = self.thumb_symbol {
            scrollbar = scrollbar.thumb_symbol(thumb_symbol);
        }
        if let Some(track_symbol) = self.track_symbol {
            scrollbar = scrollbar.track_symbol(Some(track_symbol));
        }
        if let Some(begin_symbol) = self.begin_symbol {
            scrollbar = scrollbar.begin_symbol(Some(begin_symbol));
        }
        if let Some(end_symbol) = self.end_symbol {
            scrollbar = scrollbar.end_symbol(Some(end_symbol));
        }
        if let Some(thumb_style) = self.thumb_style {
            scrollbar = scrollbar.thumb_style(thumb_style);
        }
        if let Some(track_style) = self.track_style {
            scrollbar = scrollbar.track_style(track_style);
        }
        if let Some(begin_style) = self.begin_style {
            scrollbar = scrollbar.begin_style(begin_style);
        }
        if let Some(end_style) = self.end_style {
            scrollbar = scrollbar.end_style(end_style);
        }
        scrollbar
    }
}

/// Calculate the layout for the given scrollbars.
/// This prevents overlaps in the corners, if both scrollbars are
/// visible, and tries to fit in the given block.
///
/// Returns (h_area, v_area, inner_area).
///
/// Panic
/// Panics if the orientation doesn't match.
/// h_scroll doesn't accept ScrollBarOrientation::Vertical* and
/// v_scroll doesn't accept ScrollBarOrientation::Horizontal*.
pub fn layout_scroll(
    area: Rect,
    block: Option<&Block<'_>>,
    h_scroll: Option<&Scroll<'_>>,
    v_scroll: Option<&Scroll<'_>>,
) -> (Rect, Rect, Rect) {
    let mut margin_left = 0;
    let mut margin_right = 0;
    let mut margin_top = 0;
    let mut margin_bottom = 0;

    if block.is_some() {
        margin_left = 1;
        margin_right = 1;
        margin_top = 1;
        margin_bottom = 1;
    }
    match v_scroll.map(|v| &v.orientation) {
        Some(ScrollbarOrientation::VerticalLeft) => margin_left = 1,
        Some(ScrollbarOrientation::VerticalRight) => margin_right = 1,
        _ => {}
    }
    match h_scroll.map(|v| &v.orientation) {
        Some(ScrollbarOrientation::HorizontalTop) => margin_top = 1,
        Some(ScrollbarOrientation::HorizontalBottom) => margin_bottom = 1,
        _ => {}
    }

    let h_area = if let Some(h_scroll) = h_scroll {
        match h_scroll.orientation {
            ScrollbarOrientation::VerticalRight => {
                unimplemented!(
                    "ScrollbarOrientation::VerticalRight not supported for horizontal scrolling."
                );
            }
            ScrollbarOrientation::VerticalLeft => {
                unimplemented!(
                    "ScrollbarOrientation::VerticalLeft not supported for horizontal scrolling."
                );
            }
            ScrollbarOrientation::HorizontalBottom => Rect::new(
                area.x + margin_left + h_scroll.start_margin,
                area.y + area.height.saturating_sub(1),
                area.width.saturating_sub(
                    margin_left + margin_right + h_scroll.start_margin + h_scroll.end_margin,
                ),
                if area.height > 0 { 1 } else { 0 },
            ),
            ScrollbarOrientation::HorizontalTop => Rect::new(
                area.x + margin_left + h_scroll.start_margin,
                area.y,
                area.width.saturating_sub(
                    margin_left + margin_right + h_scroll.start_margin + h_scroll.end_margin,
                ),
                if area.height > 0 { 1 } else { 0 },
            ),
        }
    } else {
        Rect::new(area.x, area.y, 0, 0)
    };
    let v_area = if let Some(v_scroll) = v_scroll {
        match v_scroll.orientation {
            ScrollbarOrientation::VerticalRight => Rect::new(
                area.x + area.width.saturating_sub(1),
                area.y + margin_top + v_scroll.start_margin,
                if area.width > 0 { 1 } else { 0 },
                area.height.saturating_sub(
                    margin_top + margin_bottom + v_scroll.start_margin + v_scroll.end_margin,
                ),
            ),
            ScrollbarOrientation::VerticalLeft => Rect::new(
                area.x,
                area.y + margin_top + v_scroll.start_margin,
                if area.width > 0 { 1 } else { 0 },
                area.height.saturating_sub(
                    margin_top + margin_bottom + v_scroll.start_margin + v_scroll.end_margin,
                ),
            ),
            ScrollbarOrientation::HorizontalBottom => {
                unimplemented!(
                    "ScrollbarOrientation::HorizontalBottom not supported for vertical scrolling."
                );
            }
            ScrollbarOrientation::HorizontalTop => {
                unimplemented!(
                    "ScrollbarOrientation::HorizontalTop not supported for vertical scrolling."
                );
            }
        }
    } else {
        Rect::new(area.x, area.y, 0, 0)
    };

    let inner = Rect::new(
        area.x + margin_left,
        area.y + margin_top,
        area.width.saturating_sub(margin_left + margin_right),
        area.height.saturating_sub(margin_top + margin_bottom),
    );

    (h_area, v_area, inner)
}

impl<'a> StatefulWidget for Scroll<'a> {
    type State = ScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll(&self, area, buf, state)
    }
}

impl<'a> StatefulWidgetRef for Scroll<'a> {
    type State = ScrollState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_scroll(self, area, buf, state);
    }
}

fn render_scroll(scroll: &Scroll<'_>, area: Rect, buf: &mut Buffer, state: &mut ScrollState) {
    state.set_orientation(scroll.orientation.clone());
    if scroll.overscroll_by.is_some() {
        state.set_overscroll_by(scroll.overscroll_by);
    }
    if scroll.scroll_by.is_some() {
        state.set_scroll_by(scroll.scroll_by);
    }

    state.area = area;

    if state.max_offset() == 0 {
        match scroll.policy {
            ScrollbarType::Show => {
                if !area.is_empty() {
                    scroll.scrollbar().render(
                        area,
                        buf,
                        &mut ScrollbarState::new(state.max_offset())
                            .position(state.offset())
                            .viewport_content_length(state.page_len()),
                    );
                }
            }
            ScrollbarType::Minimal => {
                let sym = if let Some(no_symbol) = scroll.no_symbol {
                    no_symbol
                } else if scroll.is_vertical() {
                    "\u{250A}"
                } else {
                    "\u{2508}"
                };
                for row in area.y..area.y + area.height {
                    for col in area.x..area.x + area.width {
                        let cell = buf.get_mut(col, row);
                        if let Some(no_style) = scroll.no_style {
                            cell.set_style(no_style);
                        }
                        cell.set_symbol(sym);
                    }
                }
            }
            ScrollbarType::NoRender => {
                // widget renders
            }
        }
    } else {
        if !area.is_empty() {
            scroll.scrollbar().render(
                area,
                buf,
                &mut ScrollbarState::new(state.max_offset())
                    .position(state.offset())
                    .viewport_content_length(state.page_len()),
            );
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            orientation: Default::default(),
            offset: 0,
            max_offset: 0,
            page_len: 0,
            scroll_by: None,
            overscroll_by: None,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn set_orientation(&mut self, orientation: ScrollbarOrientation) {
        self.orientation = orientation;
    }

    /// Vertical scroll?
    #[inline]
    pub fn is_vertical(&self) -> bool {
        self.orientation.is_vertical()
    }

    /// Horizontal scroll?
    #[inline]
    pub fn is_horizontal(&self) -> bool {
        self.orientation.is_horizontal()
    }

    /// Current vertical offset.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Change the offset. Limits the offset to max_v_offset + v_overscroll.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    #[inline]
    pub fn set_offset(&mut self, offset: usize) -> bool {
        let old = self.offset;
        self.offset = self.limit_offset(offset);
        old != self.offset
    }

    /// Scroll to make the given pos visible. Adjusts the
    /// offset just enough to make this happen. Does nothing if
    /// the position is already visible.
    #[inline]
    pub fn scroll_to_pos(&mut self, pos: usize) -> bool {
        let old = self.offset;
        if pos >= self.offset + self.page_len {
            self.offset = pos - self.page_len + 1;
        } else if pos < self.offset {
            self.offset = pos;
        }
        old != self.offset
    }

    /// Scroll up by n.
    #[inline]
    pub fn scroll_up(&mut self, n: usize) -> bool {
        let old = self.offset;
        self.offset = self.limit_offset(self.offset.saturating_sub(n));
        old != self.offset
    }

    /// Scroll down by n.
    #[inline]
    pub fn scroll_down(&mut self, n: usize) -> bool {
        let old = self.offset;
        self.offset = self.limit_offset(self.offset.saturating_add(n));
        old != self.offset
    }

    /// Scroll left by n.
    #[inline]
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.scroll_up(n)
    }

    /// Scroll right by n.
    #[inline]
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.scroll_down(n)
    }

    /// Calculate the offset limited to max_offset+overscroll_by.
    #[inline]
    pub fn limit_offset(&self, offset: usize) -> usize {
        min(offset, self.max_offset.saturating_add(self.overscroll_by()))
    }

    /// Calculate the offset limited to max_offset+overscroll_by.
    #[inline]
    pub fn clamp_offset(&self, offset: isize) -> usize {
        offset.clamp(
            0,
            self.max_offset.saturating_add(self.overscroll_by()) as isize,
        ) as usize
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    #[inline]
    pub fn max_offset(&self) -> usize {
        self.max_offset
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    #[inline]
    pub fn set_max_offset(&mut self, max: usize) {
        self.max_offset = max;
    }

    /// Page-size at the current offset.
    #[inline]
    pub fn page_len(&self) -> usize {
        self.page_len
    }

    /// Page-size at the current offset.
    #[inline]
    pub fn set_page_len(&mut self, page: usize) {
        self.page_len = page;
    }

    /// Suggested scroll per scroll-event.
    /// Defaults to 1/10 of the page
    #[inline]
    pub fn scroll_by(&self) -> usize {
        if let Some(scroll) = self.scroll_by {
            max(scroll, 1)
        } else {
            max(self.page_len / 10, 1)
        }
    }

    /// Suggested scroll per scroll-event.
    /// Defaults to 1/10 of the page
    #[inline]
    pub fn set_scroll_by(&mut self, scroll: Option<usize>) {
        self.scroll_by = scroll;
    }

    /// Allowed overscroll
    #[inline]
    pub fn overscroll_by(&self) -> usize {
        self.overscroll_by.unwrap_or_default()
    }

    /// Allowed overscroll
    #[inline]
    pub fn set_overscroll_by(&mut self, overscroll_by: Option<usize>) {
        self.overscroll_by = overscroll_by;
    }

    /// Update the state to match adding items.
    #[inline]
    pub fn items_added(&mut self, pos: usize, n: usize) {
        if self.offset >= pos {
            self.offset += n;
        }
        self.max_offset += n;
    }

    /// Update the state to match removing items.
    #[inline]
    pub fn items_removed(&mut self, pos: usize, n: usize) {
        if self.offset >= pos {
            self.offset -= n;
        }
        self.max_offset = self.max_offset.saturating_sub(n);
    }
}

impl ScrollState {
    /// Maps a screen-position to an offset.
    /// pos - row/column clicked
    /// base - x/y of the range
    /// length - width/height of the range.
    pub fn map_position_index(&self, pos: u16, base: u16, length: u16) -> usize {
        // correct for the arrows.
        let pos = pos.saturating_sub(base).saturating_sub(1) as usize;
        let span = length.saturating_sub(2) as usize;

        (self.max_offset.saturating_mul(pos)) / span
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome> for ScrollState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ScrollOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.area, m) => {
                if self.is_vertical() {
                    if m.row >= self.area.y {
                        ScrollOutcome::VPos(self.map_position_index(
                            m.row,
                            self.area.y,
                            self.area.height,
                        ))
                    } else {
                        ScrollOutcome::Unchanged
                    }
                } else {
                    if m.column >= self.area.x {
                        ScrollOutcome::HPos(self.map_position_index(
                            m.column,
                            self.area.x,
                            self.area.width,
                        ))
                    } else {
                        ScrollOutcome::Unchanged
                    }
                }
            }
            ct_event!(mouse down Left for col, row) if self.area.contains((*col, *row).into()) => {
                if self.is_vertical() {
                    ScrollOutcome::VPos(self.map_position_index(
                        *row,
                        self.area.y,
                        self.area.height,
                    ))
                } else {
                    ScrollOutcome::HPos(self.map_position_index(*col, self.area.x, self.area.width))
                }
            }
            ct_event!(scroll down for col, row)
                if self.is_vertical() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Down(self.scroll_by())
            }
            ct_event!(scroll up for col, row)
                if self.is_vertical() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Up(self.scroll_by())
            }
            // right scroll with ALT down. shift doesn't work?
            ct_event!(scroll ALT down for col, row)
                if self.is_horizontal() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Right(self.scroll_by())
            }
            // left scroll with ALT up. shift doesn't work?
            ct_event!(scroll ALT up for col, row)
                if self.is_horizontal() && self.area.contains((*col, *row).into()) =>
            {
                ScrollOutcome::Left(self.scroll_by())
            }
            _ => ScrollOutcome::Continue,
        }
    }
}

/// Handle all scroll events for the given area and the (possibly) two scrollbars.
#[derive(Debug)]
pub struct ScrollArea<'a>(
    pub Rect,
    pub Option<&'a mut ScrollState>,
    pub Option<&'a mut ScrollState>,
);

/// Handle scrolling for the whole area spanned by the two scroll-states.
impl<'a> HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome> for ScrollArea<'a> {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ScrollOutcome {
        let area = self.0;

        if let Some(hscroll) = &mut self.1 {
            flow!(match event {
                // right scroll with ALT down. shift doesn't work?
                ct_event!(scroll ALT down for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Right(hscroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                // left scroll with ALT up. shift doesn't work?
                ct_event!(scroll ALT up for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Left(hscroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                _ => ScrollOutcome::Continue,
            });
            flow!(hscroll.handle(event, MouseOnly));
        }
        if let Some(vscroll) = &mut self.2 {
            flow!(match event {
                ct_event!(scroll down for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Down(vscroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                ct_event!(scroll up for column, row) => {
                    if area.contains(Position::new(*column, *row)) {
                        ScrollOutcome::Up(vscroll.scroll_by())
                    } else {
                        ScrollOutcome::Continue
                    }
                }
                _ => ScrollOutcome::Continue,
            });
            flow!(vscroll.handle(event, MouseOnly));
        }

        ScrollOutcome::Continue
    }
}

impl Default for ScrollStyle {
    fn default() -> Self {
        Self {
            thumb_style: None,
            track_symbol: None,
            track_style: None,
            begin_symbol: None,
            begin_style: None,
            end_symbol: None,
            end_style: None,
            no_symbol: None,
            no_style: None,
            non_exhaustive: NonExhaustive,
        }
    }
}
