/// A view allows scrolling of a `Widget` without builtin
/// support for scrolling.
///
/// View and Viewport are the same in functionality.
///
/// The difference is that View works for [Widget]s and
/// Viewport for [StatefulWidget]s.
///
use crate::_private::NonExhaustive;
use crate::event::ScrollOutcome;
use crate::inner::{InnerOwned, InnerRef, InnerWidget};
use crate::util::copy_buffer;
use crate::{ScrollingState, ScrollingWidget};
use rat_event::{ConsumedEvent, HandleEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};

/// View has its own size, and can contain a stateless widget
/// that will be rendered to a view sized buffer.
/// This buffer is then offset and written to the actual
/// frame buffer.
#[derive(Debug, Default, Clone)]
pub struct View<T> {
    /// The widget.
    widget: T,
    view: ViewImpl,
}

#[derive(Debug, Default, Clone)]
struct ViewImpl {
    /// Size of the view. The widget is drawn to a separate buffer
    /// with this size. x and y are set to the rendering area.
    view_size: Size,
    /// Style for any area outside the contained widget.
    style: Style,
}

/// State of the view.
#[derive(Debug, Clone)]
pub struct ViewState {
    /// The drawing area for the view.
    pub area: Rect,
    /// The view area that the inner widget sees.
    pub view_area: Rect,
    /// Horizontal offset
    pub h_offset: usize,
    /// Vertical offset
    pub v_offset: usize,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<T> View<T> {
    /// New view.
    pub fn new(inner: T) -> Self {
        Self {
            widget: inner,
            view: Default::default(),
        }
    }

    /// Size for the inner widget.
    pub fn view_size(mut self, size: Size) -> Self {
        self.view.view_size = size;
        self
    }

    /// Style for the empty space outside the rendered buffer.
    pub fn style(mut self, style: Style) -> Self {
        self.view.style = style;
        self
    }
}

impl<T> StatefulWidgetRef for View<T>
where
    T: WidgetRef,
{
    type State = ViewState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = InnerRef {
            inner: &self.widget,
        };
        render_ref(&self.view, inner, area, buf, state);
    }
}

impl<T> StatefulWidget for View<T>
where
    T: Widget,
{
    type State = ViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = InnerOwned { inner: self.widget };
        render_ref(&self.view, inner, area, buf, state)
    }
}

fn render_ref<W>(
    view: &ViewImpl,
    inner: impl InnerWidget<W, ()>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ViewState,
) {
    state.area = area;
    state.view_area = Rect::new(area.x, area.y, view.view_size.width, view.view_size.height);

    let mut tmp = Buffer::empty(state.view_area);

    inner.render_inner(state.view_area, &mut tmp, &mut ());

    copy_buffer(
        state.view_area,
        tmp,
        state.v_offset,
        state.h_offset,
        view.style,
        area,
        buf,
    );
}

impl<State, T> ScrollingWidget<State> for View<T>
where
    T: Widget,
{
    fn need_scroll(&self, area: Rect, _state: &mut State) -> (bool, bool) {
        (
            area.width < self.view.view_size.width,
            area.height < self.view.view_size.height,
        )
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            view_area: Default::default(),
            h_offset: 0,
            v_offset: 0,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollingState for ViewState {
    fn vertical_max_offset(&self) -> usize {
        self.view_area.height.saturating_sub(self.area.height) as usize
    }

    fn vertical_offset(&self) -> usize {
        self.v_offset
    }

    fn vertical_page(&self) -> usize {
        self.area.height as usize
    }

    fn horizontal_max_offset(&self) -> usize {
        self.view_area.width.saturating_sub(self.area.width) as usize
    }

    fn horizontal_offset(&self) -> usize {
        self.h_offset
    }

    fn horizontal_page(&self) -> usize {
        self.area.width as usize
    }

    fn set_vertical_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.v_offset;

        if self.v_offset < self.view_area.height as usize {
            self.v_offset = offset;
        } else if self.v_offset >= self.view_area.height as usize {
            self.v_offset = self.view_area.height.saturating_sub(1) as usize;
        }

        old_offset != self.v_offset
    }

    fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.h_offset;

        if self.h_offset < self.view_area.width as usize {
            self.h_offset = offset;
        } else if self.h_offset >= self.view_area.width as usize {
            self.h_offset = self.view_area.width.saturating_sub(1) as usize;
        }

        old_offset != self.h_offset
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
impl<R, Q> HandleEvent<crossterm::event::Event, Q, ScrollOutcome<R>> for ViewState
where
    R: ConsumedEvent,
{
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: Q) -> ScrollOutcome<R> {
        ScrollOutcome::NotUsed
    }
}
