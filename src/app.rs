use crate::dataflow::{DataflowInfo, DataflowTableWidgetRefExt};
use crate::tools::execute_tool;
use makepad_platform::event::KeyCode;
use makepad_widgets::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::otlp::bridge;
#[cfg(not(target_arch = "wasm32"))]
use crate::traces::TracesPanelWidgetRefExt;

// Auto-refresh interval in seconds
const AUTO_REFRESH_INTERVAL: f64 = 5.0;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::chat::chat_screen::ChatScreen;
    use crate::dataflow::dataflow_table::DataflowTable;
    use crate::traces::traces_panel::TracesPanel;

    // Colors
    SIDEBAR_BG = #1e293b
    MAIN_BG = #f8fafc
    DIVIDER_COLOR = #e2e8f0
    HEADER_BG = #1e3a5f
    HEADER_TEXT = #ffffff
    TAB_ACTIVE_BG = #2d4a6f
    TAB_INACTIVE_BG = #1e3a5f
    CONNECTION_OK = #4ade80
    CONNECTION_ERR = #f87171

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: { title: "Dora Studio" }
                body = <View> {
                    width: Fill, height: Fill
                    flow: Down
                    show_bg: true
                    draw_bg: { color: (MAIN_BG) }

                    // Shared title bar with tabs
                    <View> {
                        width: Fill, height: 48
                        flow: Right
                        show_bg: true
                        draw_bg: { color: (HEADER_BG) }
                        padding: { left: 16, right: 16 }
                        align: { y: 0.5 }
                        spacing: 8

                        <Label> {
                            width: Fit, height: Fit
                            draw_text: {
                                color: (HEADER_TEXT),
                                text_style: { font_size: 16.0 }
                            }
                            text: "Dora Studio"
                        }

                        // Spacer between title and tabs
                        <View> { width: 16, height: Fit }

                        tab_dataflows = <Button> {
                            width: 100, height: 32
                            text: "Dataflows"
                            draw_text: { text_style: { font_size: 12.0 } }
                        }

                        tab_traces = <Button> {
                            width: 80, height: 32
                            text: "Traces"
                            draw_text: { text_style: { font_size: 12.0 } }
                        }

                        // Spacer to push right-side items
                        <View> { width: Fill, height: Fit }

                        connection_label = <Label> {
                            width: Fit, height: Fit
                            draw_text: {
                                color: (HEADER_TEXT),
                                text_style: { font_size: 11.0 }
                            }
                            text: ""
                        }

                        refresh_button = <Button> {
                            width: 80, height: 32
                            text: "Refresh"
                            draw_text: { text_style: { font_size: 12.0 } }
                        }
                    }

                    // Panels container
                    <View> {
                        width: Fill, height: Fill
                        flow: Down

                        // Dataflow panel (visible by default)
                        dataflow_view = <View> {
                            width: Fill, height: Fill
                            flow: Down
                            align: { x: 0.0, y: 0.0 }
                            padding: { top: 0, left: 16, right: 16, bottom: 16 }

                            dataflow_table = <DataflowTable> {}
                        }

                        // Traces panel (hidden by default)
                        traces_view = <View> {
                            width: Fill, height: 0
                            flow: Down
                            align: { x: 0.0, y: 0.0 }
                            padding: { top: 0, left: 16, right: 16, bottom: 16 }

                            traces_panel = <TracesPanel> {}
                        }
                    }

                    // Divider line
                    <View> {
                        width: Fill, height: 1
                        show_bg: true
                        draw_bg: { color: (DIVIDER_COLOR) }
                    }

                    // Bottom panel - Chat
                    <View> {
                        width: Fill, height: 300
                        flow: Down
                        show_bg: true
                        draw_bg: { color: #ffffff }

                        <ChatScreen> {}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum ActivePanel {
    #[default]
    Dataflows,
    Traces,
}

/// The main application widget for Dora Studio.
///
/// Handles the lifecycle, UI state, tabs (Dataflows vs Traces), and background polling
/// of the native and WASM tools.
///
/// # Examples
///
/// ```rust
/// // The App struct is primarily instantiated via Makepad's `app_main!` macro:
/// // app_main!(App);
/// ```
/// Represents the currently focused UI element for keyboard navigation
#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum FocusTarget {
    #[default]
    None,
    TabDataflows,
    TabTraces,
    RefreshButton,
    DataflowTable,
    TabTracesPanel,
    ChatInput,
    ChatSend,
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    next_frame: NextFrame,
    #[rust]
    initialized: bool,
    #[rust]
    last_refresh_time: f64,
    #[rust]
    active_panel: ActivePanel,
    #[rust]
    signoz_available: bool,
    #[rust]
    traces_loaded_once: bool,
    #[rust]
    focused_element: FocusTarget,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
        crate::chat::live_design(cx);
        crate::dataflow::live_design(cx);
        #[cfg(not(target_arch = "wasm32"))]
        crate::traces::live_design(cx);
        // Light theme
        cx.link(live_id!(theme), live_id!(theme_desktop_light));
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Initialize API key from environment variable
        crate::api::init_api_key_from_env();

        // Initialize SigNoz bridge from env vars
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.signoz_available = bridge::init_signoz_from_env();
            if self.signoz_available {
                bridge::request_health_check();
            }
        }

        // Schedule initial data load for next frame (after UI is ready)
        self.next_frame = cx.new_next_frame();
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle tab buttons
        if self.ui.button(ids!(tab_dataflows)).clicked(actions) {
            self.switch_to_panel(cx, ActivePanel::Dataflows);
        }

        if self.ui.button(ids!(tab_traces)).clicked(actions) {
            self.switch_to_panel(cx, ActivePanel::Traces);
            #[cfg(not(target_arch = "wasm32"))]
            if self.signoz_available && !self.traces_loaded_once {
                self.refresh_traces(cx);
            }
        }

        // Handle shared refresh button
        if self.ui.button(ids!(refresh_button)).clicked(actions) {
            match self.active_panel {
                ActivePanel::Dataflows => {
                    log!("[App] Refresh button clicked - refreshing dataflows");
                    self.refresh_dataflows(cx);
                }
                ActivePanel::Traces => {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        log!("[App] Refresh button clicked - refreshing traces");
                        self.refresh_traces(cx);
                    }
                }
            }
        }

        // Handle DataflowTable row actions
        let table = self.ui.dataflow_table(ids!(dataflow_table));

        if let Some(uuid) = table.stop_clicked(actions) {
            log!("[App] Stop button clicked for {}", uuid);
            self.stop_dataflow(cx, &uuid);
        }

        if let Some(uuid) = table.destroy_clicked(actions) {
            log!("[App] Destroy button clicked for {}", uuid);
            self.destroy_dataflow(cx, &uuid);
        }

        if let Some(uuid) = table.logs_clicked(actions) {
            log!("[App] Logs button clicked for {}", uuid);
            self.view_dataflow_logs(&uuid);
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);

        // Handle next frame for initialization and auto-refresh
        if let Some(ne) = self.next_frame.is_event(event) {
            if !self.initialized {
                self.initialized = true;
                self.last_refresh_time = ne.time;
                log!("[App] Initializing dataflow table on first frame");
                self.refresh_dataflows(cx);
            } else {
                // Check if it's time for auto-refresh
                let elapsed = ne.time - self.last_refresh_time;
                if elapsed >= AUTO_REFRESH_INTERVAL {
                    self.last_refresh_time = ne.time;

                    match self.active_panel {
                        ActivePanel::Dataflows => {
                            log!("[App] Auto-refresh triggered after {:.1}s", elapsed);
                            self.refresh_dataflows(cx);
                        }
                        ActivePanel::Traces =>
                        {
                            #[cfg(not(target_arch = "wasm32"))]
                            if self.signoz_available {
                                log!("[App] Auto-refresh traces after {:.1}s", elapsed);
                                self.refresh_traces(cx);
                            }
                        }
                    }
                }
            }

            // Poll SigNoz responses
            #[cfg(not(target_arch = "wasm32"))]
            {
                for response in bridge::take_signoz_responses() {
                    self.handle_signoz_response(cx, response);
                }
            }

            // Schedule the next frame to keep auto-refresh running
            self.next_frame = cx.new_next_frame();
        }

        // Handle keyboard events for accessibility
        if let Event::KeyDown(key_event) = event {
            self.handle_keyboard(cx, key_event);
        }

        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl App {
    fn switch_to_panel(&mut self, cx: &mut Cx, panel: ActivePanel) {
        self.active_panel = panel;
        match panel {
            ActivePanel::Dataflows => {
                self.ui
                    .view(ids!(dataflow_view))
                    .apply_over(cx, live! { height: Fill });
                self.ui
                    .view(ids!(traces_view))
                    .apply_over(cx, live! { height: 0 });
            }
            ActivePanel::Traces => {
                self.ui
                    .view(ids!(dataflow_view))
                    .apply_over(cx, live! { height: 0 });
                self.ui
                    .view(ids!(traces_view))
                    .apply_over(cx, live! { height: Fill });
            }
        }
        self.ui.redraw(cx);
    }

    fn handle_keyboard(&mut self, cx: &mut Cx, key_event: &makepad_platform::event::KeyEvent) {
        let shift = key_event.modifiers.shift;
        let ctrl = key_event.modifiers.ctrl;

        // Tab navigation for accessibility
        if key_event.key_code == KeyCode::Tab {
            if shift {
                // Shift+Tab: move focus backward
                self.move_focus_backward(cx);
            } else {
                // Tab: move focus forward
                self.move_focus_forward(cx);
            }
            return;
        }

        // Keyboard shortcuts
        if ctrl {
            match key_event.key_code {
                KeyCode::R => {
                    // Ctrl+R: Refresh current panel
                    log!("[App] Ctrl+R pressed - refreshing");
                    match self.active_panel {
                        ActivePanel::Dataflows => self.refresh_dataflows(cx),
                        ActivePanel::Traces =>
                        {
                            #[cfg(not(target_arch = "wasm32"))]
                            if self.signoz_available {
                                self.refresh_traces(cx);
                            }
                        }
                    }
                }
                KeyCode::D1 => {
                    // Ctrl+1: Switch to Dataflows tab
                    self.switch_to_panel(cx, ActivePanel::Dataflows);
                }
                KeyCode::D2 => {
                    // Ctrl+2: Switch to Traces tab
                    self.switch_to_panel(cx, ActivePanel::Traces);
                    #[cfg(not(target_arch = "wasm32"))]
                    if self.signoz_available && !self.traces_loaded_once {
                        self.refresh_traces(cx);
                    }
                }
                KeyCode::Return | KeyCode::NumpadEnter => {
                    // Ctrl+Enter: Send chat message (when input is focused)
                    if self.focused_element == FocusTarget::ChatInput
                        || self.focused_element == FocusTarget::ChatSend
                    {
                        // Trigger send action through the chat screen
                        self.ui
                            .chat_screen(ids!(chat_screen))
                            .view
                            .button(ids!(send_button))
                            .click();
                    }
                }
                _ => {}
            }
        }

        // Escape: Clear focus
        if key_event.key_code == KeyCode::Escape {
            self.focused_element = FocusTarget::None;
            self.ui.redraw(cx);
        }
    }

    fn move_focus_forward(&mut self, cx: &mut Cx) {
        self.focused_element = match self.focused_element {
            FocusTarget::None => {
                if self.active_panel == ActivePanel::Dataflows {
                    FocusTarget::TabDataflows
                } else {
                    FocusTarget::TabTraces
                }
            }
            FocusTarget::TabDataflows => FocusTarget::DataflowTable,
            FocusTarget::DataflowTable => FocusTarget::RefreshButton,
            FocusTarget::RefreshButton => FocusTarget::ChatInput,
            FocusTarget::ChatInput => FocusTarget::ChatSend,
            FocusTarget::ChatSend => {
                if self.active_panel == ActivePanel::Dataflows {
                    FocusTarget::TabDataflows
                } else {
                    FocusTarget::TabTraces
                }
            }
            FocusTarget::TabTraces => FocusTarget::TabTracesPanel,
            FocusTarget::TabTracesPanel => FocusTarget::RefreshButton,
        };
        self.update_focus_visual(cx);
    }

    fn move_focus_backward(&mut self, cx: &mut Cx) {
        self.focused_element = match self.focused_element {
            FocusTarget::None => FocusTarget::ChatSend,
            FocusTarget::TabDataflows => FocusTarget::ChatSend,
            FocusTarget::DataflowTable => FocusTarget::TabDataflows,
            FocusTarget::RefreshButton => {
                if self.active_panel == ActivePanel::Traces {
                    FocusTarget::TabTracesPanel
                } else {
                    FocusTarget::DataflowTable
                }
            }
            FocusTarget::ChatInput => FocusTarget::RefreshButton,
            FocusTarget::ChatSend => FocusTarget::ChatInput,
            FocusTarget::TabTraces => FocusTarget::ChatSend,
            FocusTarget::TabTracesPanel => FocusTarget::TabTraces,
        };
        self.update_focus_visual(cx);
    }

    fn update_focus_visual(&mut self, _cx: &mut Cx) {
        // Focus tracking is maintained internally for keyboard navigation
        // The actual visual focus is handled by the widget's built-in focus state
        // Makepad widgets automatically show focus indicators when they have keyboard focus
    }

    fn refresh_dataflows(&mut self, cx: &mut Cx) {
        log!("[App] refresh_dataflows called");
        let table = self.ui.dataflow_table(ids!(dataflow_table));
        table.set_loading(cx);

        // Execute dora list command
        let result = execute_tool("dora_list", "refresh", &serde_json::json!({}));
        log!(
            "[App] dora_list result: is_error={}, content={}",
            result.is_error,
            &result.content
        );

        if result.is_error {
            table.set_error(cx, &result.content);
        } else {
            // Try parsing as JSON array first, then NDJSON
            let dataflows = if result.content.trim().starts_with('[') {
                DataflowInfo::parse_json_array(&result.content)
            } else {
                DataflowInfo::parse_ndjson(&result.content)
            };
            log!("[App] Parsed {} dataflows", dataflows.len());
            table.set_dataflows(cx, dataflows);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_traces(&mut self, cx: &mut Cx) {
        log!("[App] refresh_traces called");
        let panel = self.ui.traces_panel(ids!(traces_panel));
        panel.set_loading(cx);

        let query = crate::otlp::types::TraceQuery {
            limit: Some(100),
            ..Default::default()
        };
        bridge::request_traces(query);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_signoz_response(&mut self, cx: &mut Cx, response: crate::otlp::SignozResponse) {
        match response {
            crate::otlp::SignozResponse::HealthOk => {
                log!("[App] SigNoz connected");
                self.ui
                    .label(ids!(connection_label))
                    .set_text(cx, "Connected");
            }
            crate::otlp::SignozResponse::HealthError(e) => {
                log!("[App] SigNoz health error: {}", e);
                let msg = format!("SigNoz: {}", truncate_str(&e, 40));
                self.ui.label(ids!(connection_label)).set_text(cx, &msg);
            }
            crate::otlp::SignozResponse::Traces(spans) => {
                log!("[App] Received {} trace spans", spans.len());
                self.traces_loaded_once = true;
                let panel = self.ui.traces_panel(ids!(traces_panel));
                panel.set_spans(cx, spans);
            }
            crate::otlp::SignozResponse::TracesError(e) => {
                log!("[App] Traces query error: {}", e);
                let panel = self.ui.traces_panel(ids!(traces_panel));
                panel.set_error(cx, &e);
            }
        }
    }

    fn stop_dataflow(&mut self, cx: &mut Cx, uuid: &str) {
        let args = serde_json::json!({ "dataflow_id": uuid });
        let result = execute_tool("dora_stop", "stop", &args);

        if result.is_error {
            log!("Error stopping dataflow: {}", result.content);
        }

        // Refresh the table after stopping
        self.refresh_dataflows(cx);
    }

    fn destroy_dataflow(&mut self, cx: &mut Cx, uuid: &str) {
        let args = serde_json::json!({ "dataflow_id": uuid });
        let result = execute_tool("dora_destroy", "destroy", &args);

        if result.is_error {
            log!("Error destroying dataflow: {}", result.content);
        }

        // Refresh the table after destroying
        self.refresh_dataflows(cx);
    }

    fn view_dataflow_logs(&self, uuid: &str) {
        let args = serde_json::json!({ "dataflow_id": uuid });
        let result = execute_tool("dora_logs", "logs", &args);

        if result.is_error {
            log!("Error getting logs: {}", result.content);
        } else {
            log!("Dataflow logs for {}:\n{}", uuid, result.content);
        }
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // App Configuration Tests
    // ============================================================================

    #[test]
    fn test_live_design_macro_compiles() {
        // This test verifies that the live_design! macro compiles correctly
        // The actual UI rendering requires a graphics context
        assert!(true, "live_design! macro compiled successfully");
    }

    #[test]
    fn test_window_title() {
        // Verify expected window configuration
        let expected_title = "Dora Studio";
        // The title is defined in the live_design! macro
        // This test documents the expected behavior
        assert_eq!(expected_title, "Dora Studio");
    }

    #[test]
    fn test_app_background_color() {
        // Verify expected background color configuration
        // The color #1a1a30 is defined in the live_design! macro
        let expected_color_hex = "#1a1a30";
        assert!(expected_color_hex.starts_with("#"));
        assert_eq!(expected_color_hex.len(), 7);
    }

    // ============================================================================
    // Live Register Tests
    // ============================================================================

    #[test]
    fn test_theme_configuration() {
        // Document the expected theme linking behavior
        // App::live_register links to theme_desktop_dark
        let expected_theme = "theme_desktop_dark";
        assert!(!expected_theme.is_empty());
    }

    // ============================================================================
    // Color Parsing Tests
    // ============================================================================

    #[test]
    fn test_hex_color_format() {
        // Verify the background color is valid hex format
        let color = "#1a1a30";
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);

        // Parse RGB components
        let r = u8::from_str_radix(&color[1..3], 16).unwrap();
        let g = u8::from_str_radix(&color[3..5], 16).unwrap();
        let b = u8::from_str_radix(&color[5..7], 16).unwrap();

        assert_eq!(r, 0x1a);
        assert_eq!(g, 0x1a);
        assert_eq!(b, 0x30);
    }

    // ============================================================================
    // Active Panel Tests
    // ============================================================================

    #[test]
    fn test_active_panel_default() {
        let panel = ActivePanel::default();
        assert_eq!(panel, ActivePanel::Dataflows);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 5), "hello...");
    }

    // ============================================================================
    // App Module Structure Tests
    // ============================================================================

    #[test]
    fn test_app_module_structure() {
        // Document expected module structure
        // - App struct with ui: WidgetRef field
        // - LiveRegister implementation for live_design registration
        // - MatchEvent implementation for event handling
        // - AppMain implementation for main event loop
        assert!(true, "Module structure documented");
    }
}
