pub mod traces_panel;

pub use traces_panel::{TracesPanel, TracesPanelRef, TracesPanelWidgetRefExt};

use makepad_widgets::*;

/// Register the traces module's widgets with the Makepad framework.
///
/// This must be called during application initialization to ensure `TracesPanel`
/// and its associated UI components are available.
///
/// # Examples
///
/// ```rust
/// use makepad_widgets::Cx;
///
/// pub fn app_register(cx: &mut Cx) {
///     dora_studio::traces::live_design(cx);
/// }
/// ```
pub fn live_design(cx: &mut Cx) {
    traces_panel::live_design(cx);
}
