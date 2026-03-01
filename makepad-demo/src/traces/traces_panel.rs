use makepad_widgets::*;
use std::cell::RefMut;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::otlp::types::Span;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // Colors (reused from dataflow_table)
    HEADER_BG = #1e3a5f
    HEADER_TEXT = #ffffff
    ROW_BG = #ffffff
    ROW_ALT_BG = #f8fafc
    BORDER_COLOR = #e2e8f0
    TEXT_PRIMARY = #1e293b
    TEXT_SECONDARY = #64748b
    STATUS_OK = #22c55e
    STATUS_ERROR = #ef4444
    STATUS_UNSET = #94a3b8

    // Trace table header
    TraceTableHeader = <View> {
        width: Fill, height: 40
        flow: Right
        show_bg: true
        draw_bg: { color: #f1f5f9 }
        padding: { left: 16, right: 16 }
        align: { y: 0.5 }
        spacing: 8

        <Label> {
            width: 120, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "SERVICE"
        }
        <Label> {
            width: Fill, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "OPERATION"
        }
        <Label> {
            width: 80, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "DURATION"
        }
        <Label> {
            width: 60, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "STATUS"
        }
        <Label> {
            width: 140, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "TIME"
        }
    }

    // Trace row
    TraceRow = <View> {
        width: Fill, height: 40
        flow: Right
        show_bg: true
        draw_bg: { color: (ROW_BG) }
        padding: { left: 16, right: 16 }
        align: { y: 0.5 }
        spacing: 8

        service_label = <Label> {
            width: 120, height: Fit
            draw_text: {
                color: (TEXT_PRIMARY),
                text_style: { font_size: 11.0 }
            }
        }
        operation_label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                color: (TEXT_PRIMARY),
                text_style: { font_size: 12.0 }
            }
        }
        duration_label = <Label> {
            width: 80, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
        status_label = <Label> {
            width: 60, height: Fit
            draw_text: {
                color: (STATUS_OK),
                text_style: { font_size: 11.0 }
            }
        }
        time_label = <Label> {
            width: 140, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
    }

    // Alternate trace row
    TraceRowAlt = <View> {
        width: Fill, height: 40
        flow: Right
        show_bg: true
        draw_bg: { color: (ROW_ALT_BG) }
        padding: { left: 16, right: 16 }
        align: { y: 0.5 }
        spacing: 8

        service_label = <Label> {
            width: 120, height: Fit
            draw_text: {
                color: (TEXT_PRIMARY),
                text_style: { font_size: 11.0 }
            }
        }
        operation_label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                color: (TEXT_PRIMARY),
                text_style: { font_size: 12.0 }
            }
        }
        duration_label = <Label> {
            width: 80, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
        status_label = <Label> {
            width: 60, height: Fit
            draw_text: {
                color: (STATUS_OK),
                text_style: { font_size: 11.0 }
            }
        }
        time_label = <Label> {
            width: 140, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
    }

    // Empty state
    TracesEmptyState = <View> {
        width: Fill, height: 120
        flow: Down
        align: { x: 0.5, y: 0.5 }
        show_bg: true
        draw_bg: { color: (ROW_BG) }

        <Label> {
            width: Fit, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 14.0 }
            }
            text: "No traces found"
        }
        <Label> {
            width: Fit, height: Fit
            margin: { top: 8 }
            draw_text: {
                color: #94a3b8,
                text_style: { font_size: 12.0 }
            }
            text: "No trace data available yet"
        }
    }

    // Loading state
    TracesLoadingState = <View> {
        width: Fill, height: 80
        flow: Down
        align: { x: 0.5, y: 0.5 }
        show_bg: true
        draw_bg: { color: (ROW_BG) }

        <Label> {
            width: Fit, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 14.0 }
            }
            text: "Loading traces..."
        }
    }

    // Error state
    TracesErrorState = <View> {
        width: Fill, height: 120
        flow: Down
        align: { x: 0.5, y: 0.5 }
        show_bg: true
        draw_bg: { color: (ROW_BG) }

        error_title = <Label> {
            width: Fit, height: Fit
            draw_text: {
                color: (STATUS_ERROR),
                text_style: { font_size: 14.0 }
            }
            text: "Error loading traces"
        }
        error_detail = <Label> {
            width: Fit, height: Fit
            margin: { top: 8 }
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 12.0 }
            }
            text: ""
        }
    }

    pub TracesPanel = {{TracesPanel}} {
        width: Fill, height: Fit
        flow: Down

        // Header
        <TraceTableHeader> {}

        // Data rows via PortalList
        trace_list = <PortalList> {
            width: Fill, height: 300
            flow: Down

            TraceRow = <TraceRow> {}
            TraceRowAlt = <TraceRowAlt> {}
            TracesEmptyState = <TracesEmptyState> {}
            TracesLoadingState = <TracesLoadingState> {}
            TracesErrorState = <TracesErrorState> {}
        }
    }
}

/// Loading state for the traces panel
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TracesLoadingState {
    #[default]
    Idle,
    Loading,
    Error,
}

#[derive(Live, LiveHook, Widget)]
pub struct TracesPanel {
    #[deref]
    view: View,
    #[rust]
    spans: Vec<Span>,
    #[rust]
    loading_state: TracesLoadingState,
    #[rust]
    error_message: String,
}

impl Widget for TracesPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                self.draw_rows(cx, &mut list);
            }
        }
        DrawStep::done()
    }
}

impl TracesPanel {
    pub fn set_spans(&mut self, cx: &mut Cx, spans: Vec<Span>) {
        log!("[TracesPanel] set_spans: {} items", spans.len());
        self.spans = spans;
        self.loading_state = TracesLoadingState::Idle;
        self.view.portal_list(ids!(trace_list)).redraw(cx);
        self.redraw(cx);
    }

    pub fn set_loading(&mut self, cx: &mut Cx) {
        self.loading_state = TracesLoadingState::Loading;
        self.view.portal_list(ids!(trace_list)).redraw(cx);
        self.redraw(cx);
    }

    pub fn set_error(&mut self, cx: &mut Cx, message: &str) {
        self.loading_state = TracesLoadingState::Error;
        self.error_message = message.to_string();
        self.view.portal_list(ids!(trace_list)).redraw(cx);
        self.redraw(cx);
    }

    fn draw_rows(&mut self, cx: &mut Cx2d, list: &mut RefMut<PortalList>) {
        // Loading state
        if self.loading_state == TracesLoadingState::Loading {
            list.set_item_range(cx, 0, 1);
            while let Some(item_id) = list.next_visible_item(cx) {
                if item_id == 0 {
                    let item = list.item(cx, item_id, live_id!(TracesLoadingState));
                    item.draw_all(cx, &mut Scope::empty());
                }
            }
            return;
        }

        // Error state
        if self.loading_state == TracesLoadingState::Error {
            list.set_item_range(cx, 0, 1);
            while let Some(item_id) = list.next_visible_item(cx) {
                if item_id == 0 {
                    let item = list.item(cx, item_id, live_id!(TracesErrorState));
                    item.label(ids!(error_detail))
                        .set_text(cx, &self.error_message);
                    item.draw_all(cx, &mut Scope::empty());
                }
            }
            return;
        }

        // Empty state
        if self.spans.is_empty() {
            list.set_item_range(cx, 0, 1);
            while let Some(item_id) = list.next_visible_item(cx) {
                if item_id == 0 {
                    let item = list.item(cx, item_id, live_id!(TracesEmptyState));
                    item.draw_all(cx, &mut Scope::empty());
                }
            }
            return;
        }

        // Data rows
        list.set_item_range(cx, 0, self.spans.len());

        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < self.spans.len() {
                let span = &self.spans[item_id];

                let template = if item_id % 2 == 0 {
                    live_id!(TraceRow)
                } else {
                    live_id!(TraceRowAlt)
                };

                let item = list.item(cx, item_id, template);

                item.label(ids!(service_label))
                    .set_text(cx, &span.service_name);
                item.label(ids!(operation_label))
                    .set_text(cx, &span.operation_name);
                item.label(ids!(duration_label))
                    .set_text(cx, &format_duration(span.duration_ms));
                item.label(ids!(status_label))
                    .set_text(cx, &format_status(span.has_error, span.status_code));
                item.label(ids!(time_label))
                    .set_text(cx, &format_time(span.start_time_ms));

                item.draw_all(cx, &mut Scope::empty());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Ref wrapper (same pattern as DataflowTableRef)
// ---------------------------------------------------------------------------

impl TracesPanelRef {
    pub fn set_spans(&self, cx: &mut Cx, spans: Vec<Span>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_spans(cx, spans);
        }
    }

    pub fn set_loading(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_loading(cx);
        }
    }

    pub fn set_error(&self, cx: &mut Cx, message: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_error(cx, message);
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", ms as f64 / 60_000.0)
    }
}

fn format_status(has_error: bool, status_code: i32) -> String {
    if has_error {
        "Error".to_string()
    } else if status_code == 0 {
        "Unset".to_string()
    } else {
        "OK".to_string()
    }
}

fn format_time(timestamp_ms: u64) -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    if timestamp_ms > now_ms {
        return "just now".to_string();
    }

    let diff_secs = (now_ms - timestamp_ms) / 1000;

    if diff_secs < 60 {
        format!("{}s ago", diff_secs)
    } else if diff_secs < 3600 {
        format!("{}m ago", diff_secs / 60)
    } else if diff_secs < 86400 {
        format!("{}h ago", diff_secs / 3600)
    } else {
        format!("{}d ago", diff_secs / 86400)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration(0), "0ms");
        assert_eq!(format_duration(150), "150ms");
        assert_eq!(format_duration(999), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(1000), "1.0s");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(59999), "60.0s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60_000), "1.0m");
        assert_eq!(format_duration(90_000), "1.5m");
    }

    #[test]
    fn test_format_status() {
        assert_eq!(format_status(true, 2), "Error");
        assert_eq!(format_status(false, 0), "Unset");
        assert_eq!(format_status(false, 1), "OK");
    }

    #[test]
    fn test_format_time_recent() {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let result = format_time(now_ms - 5_000);
        assert!(result.contains("5s ago"));

        let result = format_time(now_ms - 120_000);
        assert!(result.contains("2m ago"));

        let result = format_time(now_ms - 7200_000);
        assert!(result.contains("2h ago"));
    }

    #[test]
    fn test_format_time_future() {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        assert_eq!(format_time(now_ms + 10_000), "just now");
    }

    #[test]
    fn test_loading_state_default() {
        let state = TracesLoadingState::default();
        assert_eq!(state, TracesLoadingState::Idle);
    }
}
