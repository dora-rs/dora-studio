pub mod traces_panel;

pub use traces_panel::{TracesPanel, TracesPanelRef, TracesPanelWidgetRefExt};

use makepad_widgets::*;

pub fn live_design(cx: &mut Cx) {
    traces_panel::live_design(cx);
}
