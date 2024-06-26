///
/// Add scrolling behaviour to a widget.
///
/// [Scrolled] acts as a wrapper around a widget that implements [ScrollingWidget].
/// There is a second trait [ScrollingState] necessary for the state.
///
use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use crate::event::{FocusKeys, HandleEvent, MouseOnly};
use crate::inner::{InnerStatefulOwned, InnerStatefulRef, InnerWidget};
use crate::view::View;
use crate::viewport::Viewport;
use crate::{ScrollingState, ScrollingWidget};
#[allow(unused_imports)]
use log::debug;
use rat_event::{ct_event, ConsumedEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::prelude::{BlockExt, Style};
use ratatui::symbols::scrollbar::Set;
use ratatui::widgets::{
    Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, StatefulWidgetRef,
    Widget, WidgetRef,
};
use std::cmp::min;

/// A wrapper widget that scrolls it's content.
#[derive(Debug, Default, Clone)]
pub struct Scrolled<'a, T> {
    /// widget
    widget: T,
    scrolled: ScrolledImpl<'a>,
}

#[derive(Debug, Default, Clone)]
struct ScrolledImpl<'a> {
    h_overscroll: usize,
    v_overscroll: usize,
    h_scroll_policy: ScrollbarPolicy,
    v_scroll_policy: ScrollbarPolicy,
    h_scroll_position: HScrollPosition,
    v_scroll_position: VScrollPosition,

    block: Option<Block<'a>>,

    thumb_symbol: Option<&'a str>,
    thumb_style: Option<Style>,
    track_symbol: Option<&'a str>,
    track_style: Option<Style>,
    begin_symbol: Option<&'a str>,
    begin_style: Option<Style>,
    end_symbol: Option<&'a str>,
    end_style: Option<Style>,
}

#[derive(Debug, Clone)]
pub struct ScrolledStyle {
    pub thumb_style: Option<Style>,
    pub track_symbol: Option<&'static str>,
    pub track_style: Option<Style>,
    pub begin_symbol: Option<&'static str>,
    pub begin_style: Option<Style>,
    pub end_symbol: Option<&'static str>,
    pub end_style: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

/// Scrolled state.
#[derive(Debug, Clone)]
pub struct ScrolledState<WidgetState> {
    /// State of the scrolled widget.
    pub widget: WidgetState,

    /// Total screen area.
    pub area: Rect,
    /// View area.
    pub view_area: Rect,
    /// Scrollbar area.
    pub h_scrollbar_area: Option<Rect>,
    /// Scrollbar area.
    pub v_scrollbar_area: Option<Rect>,

    /// Allow overscroll by n items.
    pub v_overscroll: usize,
    /// Allow overscroll by n items.
    pub h_overscroll: usize,

    /// mouse action in progress
    pub v_drag: bool,
    pub h_drag: bool,

    pub non_exhaustive: NonExhaustive,
}

/// This policy plus the result of [ScrollingWidget::need_scroll]
/// allow to decide what to show.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarPolicy {
    Always,
    #[default]
    AsNeeded,
    Never,
}

/// Position of the vertical scrollbar.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VScrollPosition {
    Left,
    #[default]
    Right,
}

/// Position of the horizontal scrollbar.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HScrollPosition {
    Top,
    #[default]
    Bottom,
}

impl<'a, T> Scrolled<'a, T> {
    /// New scrolled widget.
    pub fn new(inner: T) -> Self {
        Self {
            widget: inner,
            scrolled: ScrolledImpl::default(),
        }
    }

    /// Allow overscrolling the max_offset by n.
    pub fn vertical_overscroll(mut self, n: usize) -> Self {
        self.scrolled.v_overscroll = n;
        self
    }

    /// Allow overscrolling the max_offset by n.
    pub fn horizontal_overscroll(mut self, n: usize) -> Self {
        self.scrolled.h_overscroll = n;
        self
    }

    /// Horizontal scrollbar policy.
    pub fn horizontal_scrollbar_policy(mut self, policy: ScrollbarPolicy) -> Self {
        self.scrolled.h_scroll_policy = policy;
        self
    }

    /// Vertical scrollbar policy.
    pub fn vertical_scrollbar_policy(mut self, policy: ScrollbarPolicy) -> Self {
        self.scrolled.v_scroll_policy = policy;
        self
    }

    /// Position
    pub fn horizontal_scroll_position(mut self, pos: HScrollPosition) -> Self {
        self.scrolled.h_scroll_position = pos;
        self
    }

    /// Position
    pub fn vertical_scroll_position(mut self, pos: VScrollPosition) -> Self {
        self.scrolled.v_scroll_position = pos;
        self
    }

    /// Block around the scrolled widget. The scrollbars are drawn
    /// as part of the block.
    ///
    /// Attention: There must be a border at the sides where you want
    /// the scrollbars. Otherwise, the calculations for the scrollbar placement
    /// will be off somewhat.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.scrolled.block = Some(block);
        self
    }

    pub fn styles(mut self, styles: ScrolledStyle) -> Self {
        self.scrolled.thumb_style = styles.thumb_style;
        self.scrolled.track_symbol = styles.track_symbol;
        self.scrolled.track_style = styles.track_style;
        self.scrolled.begin_symbol = styles.begin_symbol;
        self.scrolled.begin_style = styles.begin_style;
        self.scrolled.end_symbol = styles.end_symbol;
        self.scrolled.end_style = styles.end_style;
        self
    }

    /// Symbol for the Scrollbar.
    pub fn thumb_symbol(mut self, thumb_symbol: &'a str) -> Self {
        self.scrolled.thumb_symbol = Some(thumb_symbol);
        self
    }

    /// Style for the Scrollbar.
    pub fn thumb_style<S: Into<Style>>(mut self, thumb_style: S) -> Self {
        self.scrolled.thumb_style = Some(thumb_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn track_symbol(mut self, track_symbol: Option<&'a str>) -> Self {
        self.scrolled.track_symbol = track_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn track_style<S: Into<Style>>(mut self, track_style: S) -> Self {
        self.scrolled.track_style = Some(track_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn begin_symbol(mut self, begin_symbol: Option<&'a str>) -> Self {
        self.scrolled.begin_symbol = begin_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn begin_style<S: Into<Style>>(mut self, begin_style: S) -> Self {
        self.scrolled.begin_style = Some(begin_style.into());
        self
    }

    /// Symbol for the Scrollbar.
    pub fn end_symbol(mut self, end_symbol: Option<&'a str>) -> Self {
        self.scrolled.end_symbol = end_symbol;
        self
    }

    /// Style for the Scrollbar.
    pub fn end_style<S: Into<Style>>(mut self, end_style: S) -> Self {
        self.scrolled.end_style = Some(end_style.into());
        self
    }

    /// Set all Scrollbar symbols.
    pub fn symbols(mut self, symbols: Set) -> Self {
        self.scrolled.thumb_symbol = Some(symbols.thumb);
        if self.scrolled.track_symbol.is_some() {
            self.scrolled.track_symbol = Some(symbols.track);
        }
        if self.scrolled.begin_symbol.is_some() {
            self.scrolled.begin_symbol = Some(symbols.begin);
        }
        if self.scrolled.end_symbol.is_some() {
            self.scrolled.end_symbol = Some(symbols.end);
        }
        self
    }

    /// Set a style for all Scrollbar styles.
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        let style = style.into();
        self.scrolled.track_style = Some(style);
        self.scrolled.thumb_style = Some(style);
        self.scrolled.begin_style = Some(style);
        self.scrolled.end_style = Some(style);
        self
    }
}

impl<'a, W> Scrolled<'a, View<W>>
where
    W: Widget,
{
    /// Create a `Scrolled<View<W>>` widget for widgets without builtin
    /// scrolling behaviour.
    ///
    /// You need to set a [view_size](Scrolled::view_size()) for the
    /// area the inner widget shall receive.
    ///
    /// See [Viewport] too.
    pub fn new_view(inner: W) -> Scrolled<'a, View<W>> {
        Self {
            widget: View::new(inner),
            scrolled: Default::default(),
        }
    }

    /// Size for the inner widget.
    pub fn view_size(mut self, size: Size) -> Self {
        self.widget = self.widget.view_size(size);
        self
    }

    /// Style for the empty space outside the rendered buffer.
    pub fn view_style(mut self, style: Style) -> Self {
        self.widget = self.widget.style(style);
        self
    }
}

impl<'a, W> Scrolled<'a, Viewport<W>>
where
    W: StatefulWidget,
{
    /// Create a `Scrolled<Viewport<W>>` widget for widgets without builtin
    /// scrolling behaviour.
    ///
    /// You need to set a [view_size](Scrolled::view_size()) for the
    /// area the inner widget shall receive.
    ///
    /// See [Viewport] too.
    pub fn new_viewport(inner: W) -> Scrolled<'a, Viewport<W>> {
        Self {
            widget: Viewport::new(inner),
            scrolled: Default::default(),
        }
    }

    /// Size for the inner widget.
    pub fn view_size(mut self, size: Size) -> Self {
        self.widget = self.widget.view_size(size);
        self
    }

    /// Style for the empty space outside the rendered buffer.
    pub fn view_style(mut self, style: Style) -> Self {
        self.widget = self.widget.style(style);
        self
    }
}

impl<'a, W> StatefulWidgetRef for Scrolled<'a, W>
where
    W: StatefulWidgetRef + ScrollingWidget<W::State>,
    W::State: ScrollingState,
{
    type State = ScrolledState<W::State>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = InnerStatefulRef {
            inner: &self.widget,
        };
        render_ref(&self.scrolled, inner, area, buf, state);
    }
}

impl<'a, W> StatefulWidget for Scrolled<'a, W>
where
    W: StatefulWidget + ScrollingWidget<W::State>,
    W::State: ScrollingState,
{
    type State = ScrolledState<W::State>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = InnerStatefulOwned { inner: self.widget };
        render_ref(&self.scrolled, inner, area, buf, state);
    }
}

fn render_ref<W, S>(
    scrolled: &ScrolledImpl<'_>,
    inner: impl InnerWidget<W, S> + ScrollingWidget<S>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ScrolledState<S>,
) where
    S: ScrollingState,
{
    // reduced area for the widget to account for possible scrollbars.
    let view_area = if scrolled.block.is_some() {
        // block should already account for the scrollbars.
        scrolled.block.inner_if_some(area)
    } else {
        let w = if scrolled.h_scroll_policy != ScrollbarPolicy::Never {
            area.width.saturating_sub(1)
        } else {
            area.width
        };
        let h = if scrolled.v_scroll_policy != ScrollbarPolicy::Never {
            area.height.saturating_sub(1)
        } else {
            area.height
        };
        Rect::new(area.x, area.y, w, h)
    };

    let scroll_param = inner.need_scroll(view_area, &mut state.widget);

    state.area = area;
    state.v_overscroll = scrolled.v_overscroll;
    state.h_overscroll = scrolled.h_overscroll;

    let has_hscroll = scrolled.h_scroll_policy.apply(scroll_param.0);
    let has_vscroll = scrolled.v_scroll_policy.apply(scroll_param.1);

    debug!("scroll {:?} {:?}", has_hscroll, has_vscroll);

    // Calculate the areas for the scrollbars and the view-area.
    // If there is a block set, assume there is a right and a bottom border too.
    // Currently, there is no way to know it. Overwriting part of the content is
    // ok in this case.
    if has_vscroll {
        let mut vscrollbar_area = area.columns().last().expect("scroll");
        if scrolled.block.is_some() {
            vscrollbar_area.y += 1;
            vscrollbar_area.height = vscrollbar_area.height.saturating_sub(1);
        }
        if has_hscroll {
            debug!("double scroll");
            vscrollbar_area.height = vscrollbar_area.height.saturating_sub(1);
        }
        state.v_scrollbar_area = Some(vscrollbar_area);
    }

    if has_hscroll {
        let mut hscrollbar_area = area.rows().last().expect("scroll");
        if scrolled.block.is_some() {
            hscrollbar_area.x += 1;
            hscrollbar_area.width = hscrollbar_area.width.saturating_sub(1);
        }
        if has_vscroll {
            hscrollbar_area.width = hscrollbar_area.width.saturating_sub(1);
        }
        state.h_scrollbar_area = Some(hscrollbar_area);
    }

    // calculate actual view area
    if let Some(block) = scrolled.block.as_ref() {
        state.view_area = block.inner(area);
    } else {
        state.view_area = area;
        if has_vscroll {
            state.view_area.width = state.view_area.width.saturating_sub(1);
        }
        if has_hscroll {
            state.view_area.height = state.view_area.height.saturating_sub(1);
        }
    }

    inner.render_inner(state.view_area, buf, &mut state.widget);

    scrolled.block.render_ref(area, buf);

    if let Some(vscrollbar_area) = state.v_scrollbar_area {
        let mut vscroll = Scrollbar::new(scrolled.v_scroll_position.orientation());
        if let Some(thumb_symbol) = scrolled.thumb_symbol {
            vscroll = vscroll.thumb_symbol(thumb_symbol);
        }
        if let Some(track_symbol) = scrolled.track_symbol {
            vscroll = vscroll.track_symbol(Some(track_symbol));
        }
        if let Some(begin_symbol) = scrolled.begin_symbol {
            vscroll = vscroll.begin_symbol(Some(begin_symbol));
        }
        if let Some(end_symbol) = scrolled.end_symbol {
            vscroll = vscroll.end_symbol(Some(end_symbol));
        }
        if let Some(thumb_style) = scrolled.thumb_style {
            vscroll = vscroll.thumb_style(thumb_style);
        }
        if let Some(track_style) = scrolled.track_style {
            vscroll = vscroll.track_style(track_style);
        }
        if let Some(begin_style) = scrolled.begin_style {
            vscroll = vscroll.begin_style(begin_style);
        }
        if let Some(end_style) = scrolled.end_style {
            vscroll = vscroll.end_style(end_style);
        }

        let max_offset = state.widget.vertical_max_offset();
        let offset = state.widget.vertical_offset();
        let view_len = state.widget.vertical_page();

        if max_offset == 0 {
            // when max_offset is 0, Scrollbar doesn't do anything.
            if let Some(track_style) = scrolled.track_style {
                buf.set_style(vscrollbar_area, track_style);
            }
        } else {
            let mut vscroll_state = ScrollbarState::new(max_offset)
                .position(offset)
                .viewport_content_length(view_len);
            vscroll.render(vscrollbar_area, buf, &mut vscroll_state);
        }
    }

    if let Some(hscrollbar_area) = state.h_scrollbar_area {
        let mut hscroll = Scrollbar::new(scrolled.h_scroll_position.orientation());
        if let Some(thumb_symbol) = scrolled.thumb_symbol {
            hscroll = hscroll.thumb_symbol(thumb_symbol);
        }
        if let Some(track_symbol) = scrolled.track_symbol {
            hscroll = hscroll.track_symbol(Some(track_symbol));
        }
        if let Some(begin_symbol) = scrolled.begin_symbol {
            hscroll = hscroll.begin_symbol(Some(begin_symbol));
        }
        if let Some(end_symbol) = scrolled.end_symbol {
            hscroll = hscroll.end_symbol(Some(end_symbol));
        }
        if let Some(thumb_style) = scrolled.thumb_style {
            hscroll = hscroll.thumb_style(thumb_style);
        }
        if let Some(track_style) = scrolled.track_style {
            hscroll = hscroll.track_style(track_style);
        }
        if let Some(begin_style) = scrolled.begin_style {
            hscroll = hscroll.begin_style(begin_style);
        }
        if let Some(end_style) = scrolled.end_style {
            hscroll = hscroll.end_style(end_style);
        }

        let max_offset = state.widget.horizontal_max_offset();
        let offset = state.widget.horizontal_offset();
        let view_len = state.widget.horizontal_page();

        if max_offset == 0 {
            // when max_offset is 0, Scrollbar doesn't do anything.
            if let Some(track_style) = scrolled.track_style {
                buf.set_style(hscrollbar_area, track_style);
            }
        } else {
            let mut hscroll_state = ScrollbarState::new(max_offset)
                .position(offset)
                .viewport_content_length(view_len);

            hscroll.render(hscrollbar_area, buf, &mut hscroll_state);
        }
    }
}

impl Default for ScrolledStyle {
    fn default() -> Self {
        Self {
            thumb_style: None,
            track_symbol: None,
            track_style: None,
            begin_symbol: None,
            begin_style: None,
            end_symbol: None,
            end_style: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollbarPolicy {
    /// Apply the policy to the scroll-flag received from the inner widget.
    pub fn apply(&self, scroll: bool) -> bool {
        match self {
            ScrollbarPolicy::Always => true,
            ScrollbarPolicy::AsNeeded => scroll,
            ScrollbarPolicy::Never => false,
        }
    }
}

impl HScrollPosition {
    /// Convert to ScrollbarOrientation.
    pub fn orientation(&self) -> ScrollbarOrientation {
        match self {
            HScrollPosition::Top => ScrollbarOrientation::HorizontalTop,
            HScrollPosition::Bottom => ScrollbarOrientation::HorizontalBottom,
        }
    }
}

impl VScrollPosition {
    /// Convert to ScrollbarOrientation.
    pub fn orientation(&self) -> ScrollbarOrientation {
        match self {
            VScrollPosition::Left => ScrollbarOrientation::VerticalLeft,
            VScrollPosition::Right => ScrollbarOrientation::VerticalRight,
        }
    }
}

impl<WState: Default> Default for ScrolledState<WState> {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            area: Default::default(),
            view_area: Default::default(),
            h_scrollbar_area: None,
            v_scrollbar_area: None,
            v_overscroll: 0,
            h_overscroll: 0,
            v_drag: false,
            h_drag: false,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<WState: ScrollingState> ScrolledState<WState> {
    /// Current vertical offset.
    pub fn vertical_offset(&self) -> usize {
        self.widget.vertical_offset()
    }

    /// Current horizontal offset.
    pub fn horizontal_offset(&self) -> usize {
        self.widget.horizontal_offset()
    }

    /// Change the offset. Limits the offset to max_v_offset + v_overscroll.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    pub fn set_vertical_offset(&mut self, offset: usize) -> bool {
        let voffset = min(
            offset,
            self.widget.vertical_max_offset() + self.v_overscroll,
        );
        self.widget.set_vertical_offset(voffset)
    }

    /// Change the offset. Limits the offset to max_h_offset + h_overscroll.
    ///
    /// Due to overscroll it's possible that this is an invalid
    /// offset for the widget. The widget must deal with this
    /// situation.
    pub fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        let hoffset = min(
            offset,
            self.widget.horizontal_max_offset() + self.h_overscroll,
        );
        self.widget.set_horizontal_offset(hoffset)
    }

    /// Scroll up by n.
    pub fn scroll_up(&mut self, n: usize) -> bool {
        self.set_vertical_offset(self.vertical_offset().saturating_sub(n))
    }

    /// Scroll down by n, but limited by the max_offset + overscroll
    pub fn scroll_down(&mut self, n: usize) -> bool {
        let v_offset = min(
            self.widget.vertical_offset() + n,
            self.widget.vertical_max_offset() + self.v_overscroll,
        );
        self.set_vertical_offset(v_offset)
    }

    /// Scroll up by n.
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.set_horizontal_offset(self.horizontal_offset().saturating_sub(n))
    }

    /// Scroll right by n, but limited by the max_offset + overscroll
    pub fn scroll_right(&mut self, n: usize) -> bool {
        let hoffset = min(
            self.widget.horizontal_offset() + n,
            self.widget.horizontal_max_offset() + self.h_overscroll,
        );
        self.set_horizontal_offset(hoffset)
    }

    pub fn widget_mut(&mut self) -> &mut WState {
        &mut self.widget
    }
}

/// A way to call event-handlers for the inner widget.
///
/// call the event-handler for DoubleClick on the inner widget.
/// ```rust ignore
/// scroll_state.handle(event, Inner(DoubleClick))
/// ```
///
/// or call it on inner directly
///
/// ```rust ignore
/// scroll_state.inner.handle(event, DoubleClick)
/// ```
#[derive(Debug)]
pub struct Inner<Qualifier>(pub Qualifier);

/// Forward event-handling to the inner widget.
impl<WState, Q, R> HandleEvent<crossterm::event::Event, Inner<Q>, ScrollOutcome<R>>
    for ScrolledState<WState>
where
    WState: ScrollingState + HandleEvent<crossterm::event::Event, Q, R>,
    R: ConsumedEvent,
{
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: Inner<Q>) -> ScrollOutcome<R> {
        forward_filter(self, event, qualifier.0) // ...
            .or_else(|| mouse_handling(self, event, MouseOnly))
    }
}

/// Handle events or the scrolled widget and forward to the inner widget.
impl<R, WState> HandleEvent<crossterm::event::Event, FocusKeys, ScrollOutcome<R>>
    for ScrolledState<WState>
where
    WState: ScrollingState
        + HandleEvent<crossterm::event::Event, FocusKeys, R>
        + HandleEvent<crossterm::event::Event, MouseOnly, R>,
    R: ConsumedEvent,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> ScrollOutcome<R> {
        forward_filter(self, event, FocusKeys) // ...
            .or_else(|| mouse_handling(self, event, MouseOnly))
    }
}

/// Handle events for the Scrolled widget and the scrollbars.
impl<R, WState> HandleEvent<crossterm::event::Event, MouseOnly, ScrollOutcome<R>>
    for ScrolledState<WState>
where
    WState: ScrollingState + HandleEvent<crossterm::event::Event, MouseOnly, R>,
    R: ConsumedEvent,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> ScrollOutcome<R> {
        forward_filter(self, event, MouseOnly) // ...
            .or_else(|| mouse_handling(self, event, MouseOnly))
    }
}

// only mouse handling for the scrolled widget itself.
fn mouse_handling<W, R>(
    widget: &mut ScrolledState<W>,
    event: &crossterm::event::Event,
    _qualifier: MouseOnly,
) -> ScrollOutcome<R>
where
    W: ScrollingState,
    R: ConsumedEvent,
{
    match event {
        // Click on one of the scrollbar sets the offset to
        // the scaled up position.
        ct_event!(mouse down Left for column,row) => {
            if let Some(vscroll_area) = widget.v_scrollbar_area {
                if vscroll_area.contains(Position::new(*column, *row)) {
                    // correct for the top `^` and bottom `v` arrows.
                    let row = row.saturating_sub(vscroll_area.y).saturating_sub(1) as usize;
                    let height = vscroll_area.height.saturating_sub(2) as usize;

                    let pos = (widget.widget.vertical_max_offset() * row) / height;

                    widget.v_drag = true;
                    if widget.widget.set_vertical_offset(pos) {
                        return ScrollOutcome::Changed;
                    } else {
                        return ScrollOutcome::NotUsed;
                    }
                }
            }
            if let Some(hscroll_area) = widget.h_scrollbar_area {
                if hscroll_area.contains(Position::new(*column, *row)) {
                    // correct for the left `<` and right `>` arrows.
                    let col = column.saturating_sub(hscroll_area.x).saturating_sub(1) as usize;
                    let width = hscroll_area.width.saturating_sub(2) as usize;

                    let pos = (widget.widget.horizontal_max_offset() * col) / width;

                    widget.h_drag = true;
                    if widget.widget.set_horizontal_offset(pos) {
                        return ScrollOutcome::Changed;
                    } else {
                        return ScrollOutcome::NotUsed;
                    }
                }
            }
        }
        // the same as before with drag events.
        ct_event!(mouse drag Left for column, row) => {
            if widget.v_drag {
                if let Some(vscroll_area) = widget.v_scrollbar_area {
                    // correct for the top `^` and bottom `v` arrows.
                    let row = row.saturating_sub(vscroll_area.y).saturating_sub(1) as usize;
                    let height = vscroll_area.height.saturating_sub(2) as usize;

                    let pos = (widget.widget.vertical_max_offset() * row) / height;

                    if widget.set_vertical_offset(pos) {
                        return ScrollOutcome::Changed;
                    } else {
                        return ScrollOutcome::NotUsed;
                    }
                }
            }
            if widget.h_drag {
                if let Some(hscroll_area) = widget.h_scrollbar_area {
                    // correct for the left `<` and right `>` arrows.
                    let col = column.saturating_sub(hscroll_area.x).saturating_sub(1) as usize;
                    let width = hscroll_area.width.saturating_sub(2) as usize;

                    let pos = (col * widget.widget.horizontal_max_offset()) / width;
                    if widget.set_horizontal_offset(pos) {
                        return ScrollOutcome::Changed;
                    } else {
                        return ScrollOutcome::NotUsed;
                    }
                }
            }
        }

        ct_event!(mouse moved) => {
            // reset drag
            widget.v_drag = false;
            widget.h_drag = false;
        }

        ct_event!(scroll down for column, row) => {
            if widget.area.contains(Position::new(*column, *row)) {
                if widget.scroll_down(widget.widget.vertical_scroll()) {
                    return ScrollOutcome::Changed;
                } else {
                    return ScrollOutcome::NotUsed;
                }
            }
        }
        ct_event!(scroll up for column, row) => {
            if widget.area.contains(Position::new(*column, *row)) {
                if widget.widget.scroll_up(widget.widget.vertical_scroll()) {
                    return ScrollOutcome::Changed;
                } else {
                    return ScrollOutcome::NotUsed;
                }
            }
        }
        // right scroll with ALT down. shift doesn't work?
        ct_event!(scroll ALT down for column, row) => {
            if widget.area.contains(Position::new(*column, *row)) {
                if widget.scroll_right(widget.widget.horizontal_scroll()) {
                    return ScrollOutcome::Changed;
                } else {
                    return ScrollOutcome::NotUsed;
                }
            }
        }
        // left scroll with ALT up. shift doesn't work?
        ct_event!(scroll ALT up for column, row) => {
            if widget.area.contains(Position::new(*column, *row)) {
                if widget.widget.scroll_left(widget.widget.horizontal_scroll()) {
                    return ScrollOutcome::Changed;
                } else {
                    return ScrollOutcome::NotUsed;
                }
            }
        }
        _ => {}
    }
    ScrollOutcome::NotUsed
}

fn forward_filter<W, Q, R>(
    widget: &mut ScrolledState<W>,
    event: &crossterm::event::Event,
    qualifier: Q,
) -> ScrollOutcome<R>
where
    W: ScrollingState + HandleEvent<crossterm::event::Event, Q, R>,
    R: ConsumedEvent,
{
    let r = match event {
        // these are the events where the scrolled widget might
        // compete with the widget. these are only forwarded if
        // inside the view area.
        ct_event!(mouse down Left for column, row)
        | ct_event!(scroll down for column, row)
        | ct_event!(scroll up for column, row)
        | ct_event!(scroll ALT down for column, row)
        | ct_event!(scroll ALT up for column, row) => {
            if widget.view_area.contains(Position::new(*column, *row)) {
                ScrollOutcome::Inner(widget.widget.handle(event, qualifier))
            } else {
                ScrollOutcome::NotUsed
            }
        }
        // the rest is simply forwarded
        _ => ScrollOutcome::Inner(widget.widget.handle(event, qualifier)),
    };
    r
}
