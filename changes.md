# 0.27.0

** Start of BETA **

* feature: add collaboration with Split
* refactor: rename everything from Scrolled* to Scroll*
* refactor: Remove scroll.core, too much.
* refactor: Move View and Viewport to rat-widget. They have no
  special casing anymore.
* refactor: change ScrollbarPolicy to ScrollbarType and add/rename to Show/Minimal/NoRender
* fix: underflow
* fix: use overscroll_by, scroll_by set on the widget only if it was set.
* fix: layout with scrollbars + block

# 0.12.0

Throw away the whole concept. Using Scrolled as a container widget is
very unwieldy. Replace with an in internal `Scroll<'a>` utility a la Block.

# 0.11.3

* Better event-forwarding for Scrolled and ViewPort.

# 0.11.2

* feature: impl StatefulWidgetRef and WidgetRef for this.
* feature: Add keymap Inner(KeyMap) for forwarding events to the inner widget.

# 0.11.1

* add ScrolledStyle for setting all styles at once.

* fix potential `- 1` panic

# 0.11.0

* reorg of rat-event
* rename Outcome to ScrollOutcome to avoid 4d-chess.
* removed StatefulWidgetRef for now

# 0.10.1

* fixed versions.

# 0.10.0

* Doubling viewport to View/Viewport.

* Recursion works now. Scrolled can contain a widget that has a
  scrolled of its own. Can scroll both the inner scrolled and the
  outer one.

# 0.9.0

Copied from test area. 