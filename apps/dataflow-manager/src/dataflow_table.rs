use makepad_widgets::*;
use serde::Deserialize;
use std::cell::RefMut;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // Colors
    HEADER_BG = #1e3a5f
    HEADER_TEXT = #ffffff
    ROW_BG = #ffffff
    ROW_ALT_BG = #f8fafc
    ROW_HOVER_BG = #e0f2fe
    ROW_SELECTED_BG = #bfdbfe
    BORDER_COLOR = #e2e8f0
    TEXT_PRIMARY = #1e293b
    TEXT_SECONDARY = #64748b

    // Status colors
    STATUS_RUNNING = #22c55e
    STATUS_FINISHED = #3b82f6
    STATUS_FAILED = #ef4444
    STATUS_STOPPED = #f59e0b
    STATUS_PENDING = #8b5cf6

    // Action button colors
    BTN_STOP_BG = #fef3c7
    BTN_STOP_TEXT = #d97706
    BTN_DESTROY_BG = #fee2e2
    BTN_DESTROY_TEXT = #dc2626
    BTN_LOGS_BG = #e0e7ff
    BTN_LOGS_TEXT = #4f46e5

    // Table header row
    TableHeader = <View> {
        width: Fill, height: 40
        flow: Right
        show_bg: true
        draw_bg: { color: #f1f5f9 }
        padding: { left: 16, right: 16 }
        align: { y: 0.5 }
        spacing: 8

        <Label> {
            width: 90, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "UUID"
        }
        <Label> {
            width: Fill, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "NAME"
        }
        <Label> {
            width: 70, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "STATUS"
        }
        <Label> {
            width: 50, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "CPU"
        }
        <Label> {
            width: 60, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "MEM"
        }
        <Label> {
            width: 110, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
            text: "ACTIONS"
        }
    }

    // Status badge component
    StatusBadge = <RoundedView> {
        width: Fit, height: 22
        padding: { left: 8, right: 8, top: 2, bottom: 2 }
        draw_bg: {
            color: #dcfce7
        }

        status_text = <Label> {
            width: Fit, height: Fit
            draw_text: {
                color: (STATUS_RUNNING),
                text_style: { font_size: 11.0 }
            }
        }
    }

    // Action button
    ActionButton = <Button> {
        width: 50, height: 24
        draw_text: { text_style: { font_size: 10.0 } }
        padding: { left: 6, right: 6 }
    }

    // Table data row
    TableRow = <View> {
        width: Fill, height: 48
        flow: Right
        show_bg: true
        draw_bg: { color: (ROW_BG) }
        padding: { left: 16, right: 16 }
        align: { y: 0.5 }
        spacing: 8

        uuid_label = <Label> {
            width: 90, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
        name_label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                color: (TEXT_PRIMARY),
                text_style: { font_size: 12.0 }
            }
        }
        status_label = <Label> {
            width: 70, height: Fit
            draw_text: {
                color: (STATUS_RUNNING),
                text_style: { font_size: 12.0 }
            }
        }
        cpu_label = <Label> {
            width: 50, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
        memory_label = <Label> {
            width: 60, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }

        // Action buttons container
        actions = <View> {
            width: 110, height: Fit
            flow: Right
            align: { x: 1.0, y: 0.5 }
            spacing: 4

            stop_button = <ActionButton> {
                text: "Stop"
            }
            destroy_button = <ActionButton> {
                text: "Kill"
            }
        }
    }

    // Alternate row with different background
    TableRowAlt = <View> {
        width: Fill, height: 48
        flow: Right
        show_bg: true
        draw_bg: { color: (ROW_ALT_BG) }
        padding: { left: 16, right: 16 }
        align: { y: 0.5 }
        spacing: 8

        uuid_label = <Label> {
            width: 90, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
        name_label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                color: (TEXT_PRIMARY),
                text_style: { font_size: 12.0 }
            }
        }
        status_label = <Label> {
            width: 70, height: Fit
            draw_text: {
                color: (STATUS_RUNNING),
                text_style: { font_size: 12.0 }
            }
        }
        cpu_label = <Label> {
            width: 50, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }
        memory_label = <Label> {
            width: 60, height: Fit
            draw_text: {
                color: (TEXT_SECONDARY),
                text_style: { font_size: 11.0 }
            }
        }

        // Action buttons container
        actions = <View> {
            width: 110, height: Fit
            flow: Right
            align: { x: 1.0, y: 0.5 }
            spacing: 4

            stop_button = <ActionButton> {
                text: "Stop"
            }
            destroy_button = <ActionButton> {
                text: "Kill"
            }
        }
    }

    // Empty state view
    EmptyState = <View> {
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
            text: "No dataflows running"
        }
        <Label> {
            width: Fit, height: Fit
            margin: { top: 8 }
            draw_text: {
                color: #94a3b8,
                text_style: { font_size: 12.0 }
            }
            text: "Start a dataflow to see it here"
        }
    }

    // Loading state view
    LoadingState = <View> {
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
            text: "Loading dataflows..."
        }
    }

    pub DataflowTable = {{DataflowTable}} {
        width: Fill, height: Fit
        flow: Down

        // Header
        <TableHeader> {}

        // Data rows via PortalList
        table_list = <PortalList> {
            width: Fill, height: 300
            flow: Down

            TableRow = <TableRow> {}
            TableRowAlt = <TableRowAlt> {}
            EmptyState = <EmptyState> {}
            LoadingState = <LoadingState> {}
        }
    }
}

/// Dataflow information from dora list command
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DataflowInfo {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub nodes: u32,
    #[serde(default)]
    pub cpu: f64,
    #[serde(default)]
    pub memory: f64,
}

impl DataflowInfo {
    /// Parse NDJSON (newline-delimited JSON) into a vector of DataflowInfo
    pub fn parse_ndjson(input: &str) -> Vec<Self> {
        input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    /// Parse JSON array into a vector of DataflowInfo
    pub fn parse_json_array(input: &str) -> Vec<Self> {
        serde_json::from_str(input).unwrap_or_default()
    }

    /// Format memory in human-readable format
    pub fn memory_formatted(&self) -> String {
        if self.memory < 0.001 {
            "0 B".to_string()
        } else if self.memory < 1.0 {
            format!("{:.0} MB", self.memory * 1024.0)
        } else {
            format!("{:.2} GB", self.memory)
        }
    }

    /// Format CPU percentage
    pub fn cpu_formatted(&self) -> String {
        format!("{:.1}%", self.cpu)
    }

    /// Get short UUID (first 8 characters)
    pub fn uuid_short(&self) -> String {
        if self.uuid.len() > 8 {
            format!("{}...", &self.uuid[..8])
        } else {
            self.uuid.clone()
        }
    }

    /// Check if dataflow is running
    pub fn is_running(&self) -> bool {
        self.status.to_lowercase() == "running"
    }
}

/// Actions emitted by the DataflowTable
#[derive(Clone, Debug, DefaultNone)]
pub enum DataflowTableAction {
    None,
    Refresh,
    Stop(String),     // uuid
    Destroy(String),  // uuid
    ViewLogs(String), // uuid
    SelectRow(usize), // row index
}

/// Loading state for the table
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TableLoadingState {
    #[default]
    Idle,
    Loading,
    Error,
}

#[derive(Live, LiveHook, Widget)]
pub struct DataflowTable {
    #[deref]
    view: View,
    #[rust]
    dataflows: Vec<DataflowInfo>,
    #[rust]
    loading_state: TableLoadingState,
    #[rust]
    selected_row: Option<usize>,
    #[rust]
    error_message: String,
}

impl Widget for DataflowTable {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
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

impl WidgetMatchEvent for DataflowTable {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        log!(
            "[DataflowTable] handle_actions called, actions count: {}",
            actions.len()
        );

        // Handle row action buttons via PortalList
        let table_list = self.view.portal_list(ids!(table_list));
        for (item_id, item) in table_list.items_with_actions(actions) {
            if item_id < self.dataflows.len() {
                let uuid = self.dataflows[item_id].uuid.clone();

                if item.button(ids!(stop_button)).clicked(actions) {
                    cx.widget_action(
                        self.widget_uid(),
                        &scope.path,
                        DataflowTableAction::Stop(uuid.clone()),
                    );
                }

                if item.button(ids!(destroy_button)).clicked(actions) {
                    cx.widget_action(
                        self.widget_uid(),
                        &scope.path,
                        DataflowTableAction::Destroy(uuid.clone()),
                    );
                }

                if item.button(ids!(logs_button)).clicked(actions) {
                    cx.widget_action(
                        self.widget_uid(),
                        &scope.path,
                        DataflowTableAction::ViewLogs(uuid.clone()),
                    );
                }
            }
        }
    }
}

impl DataflowTable {
    /// Set the dataflows to display
    pub fn set_dataflows(&mut self, cx: &mut Cx, dataflows: Vec<DataflowInfo>) {
        log!("[DataflowTable] set_dataflows: {} items", dataflows.len());
        self.dataflows = dataflows;
        self.loading_state = TableLoadingState::Idle;
        log!("[DataflowTable] calling redraw");
        // Redraw the PortalList specifically to ensure it updates
        self.view.portal_list(ids!(table_list)).redraw(cx);
        self.redraw(cx);
    }

    /// Parse and set dataflows from NDJSON string
    pub fn set_from_ndjson(&mut self, cx: &mut Cx, ndjson: &str) {
        self.dataflows = DataflowInfo::parse_ndjson(ndjson);
        self.loading_state = TableLoadingState::Idle;
        self.view.portal_list(ids!(table_list)).redraw(cx);
        self.redraw(cx);
    }

    /// Parse and set dataflows from JSON array string
    pub fn set_from_json(&mut self, cx: &mut Cx, json: &str) {
        self.dataflows = DataflowInfo::parse_json_array(json);
        self.loading_state = TableLoadingState::Idle;
        self.view.portal_list(ids!(table_list)).redraw(cx);
        self.redraw(cx);
    }

    /// Set loading state
    pub fn set_loading(&mut self, cx: &mut Cx) {
        self.loading_state = TableLoadingState::Loading;
        self.view.portal_list(ids!(table_list)).redraw(cx);
        self.redraw(cx);
    }

    /// Set error state with message
    pub fn set_error(&mut self, cx: &mut Cx, message: &str) {
        self.loading_state = TableLoadingState::Error;
        self.error_message = message.to_string();
        self.view.portal_list(ids!(table_list)).redraw(cx);
        self.redraw(cx);
    }

    /// Get current dataflows
    pub fn get_dataflows(&self) -> &[DataflowInfo] {
        &self.dataflows
    }

    /// Get dataflow by index
    pub fn get_dataflow(&self, index: usize) -> Option<&DataflowInfo> {
        self.dataflows.get(index)
    }

    /// Get dataflow by UUID
    pub fn get_dataflow_by_uuid(&self, uuid: &str) -> Option<&DataflowInfo> {
        self.dataflows.iter().find(|df| df.uuid == uuid)
    }

    /// Clear all dataflows
    pub fn clear(&mut self, cx: &mut Cx) {
        self.dataflows.clear();
        self.selected_row = None;
        self.loading_state = TableLoadingState::Idle;
        self.view.portal_list(ids!(table_list)).redraw(cx);
        self.redraw(cx);
    }

    fn draw_rows(&mut self, cx: &mut Cx2d, list: &mut RefMut<PortalList>) {
        log!(
            "[DataflowTable] draw_rows called, loading_state={:?}, dataflows.len()={}",
            self.loading_state,
            self.dataflows.len()
        );

        // Show loading state
        if self.loading_state == TableLoadingState::Loading {
            log!("[DataflowTable] showing loading state");
            list.set_item_range(cx, 0, 1);
            while let Some(item_id) = list.next_visible_item(cx) {
                if item_id == 0 {
                    let item = list.item(cx, item_id, live_id!(LoadingState));
                    item.draw_all(cx, &mut Scope::empty());
                }
            }
            return;
        }

        // Show empty state if no dataflows
        if self.dataflows.is_empty() {
            log!("[DataflowTable] showing empty state");
            list.set_item_range(cx, 0, 1);
            while let Some(item_id) = list.next_visible_item(cx) {
                if item_id == 0 {
                    let item = list.item(cx, item_id, live_id!(EmptyState));
                    item.draw_all(cx, &mut Scope::empty());
                }
            }
            return;
        }

        // Draw data rows
        log!("[DataflowTable] drawing {} data rows", self.dataflows.len());
        list.set_item_range(cx, 0, self.dataflows.len());

        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < self.dataflows.len() {
                let df = &self.dataflows[item_id];

                // Alternate row colors
                let template = if item_id % 2 == 0 {
                    live_id!(TableRow)
                } else {
                    live_id!(TableRowAlt)
                };

                let item = list.item(cx, item_id, template);

                // Set row data
                item.label(ids!(uuid_label)).set_text(cx, &df.uuid_short());
                item.label(ids!(name_label)).set_text(cx, &df.name);
                item.label(ids!(status_label)).set_text(cx, &df.status);
                item.label(ids!(cpu_label))
                    .set_text(cx, &df.cpu_formatted());
                item.label(ids!(memory_label))
                    .set_text(cx, &df.memory_formatted());

                log!(
                    "[DataflowTable] Drawing row {}: uuid={}, name={}, status={}, cpu={}, mem={}",
                    item_id,
                    df.uuid_short(),
                    df.name,
                    df.status,
                    df.cpu_formatted(),
                    df.memory_formatted()
                );

                item.draw_all(cx, &mut Scope::empty());
            }
        }
    }
}

impl DataflowTableRef {
    /// Set the dataflows to display
    pub fn set_dataflows(&self, cx: &mut Cx, dataflows: Vec<DataflowInfo>) {
        log!(
            "[DataflowTableRef] set_dataflows called with {} items",
            dataflows.len()
        );
        if let Some(mut inner) = self.borrow_mut() {
            log!("[DataflowTableRef] borrow_mut succeeded, setting dataflows");
            inner.set_dataflows(cx, dataflows);
        } else {
            log!("[DataflowTableRef] borrow_mut returned None!");
        }
    }

    /// Parse and set dataflows from NDJSON string
    pub fn set_from_ndjson(&self, cx: &mut Cx, ndjson: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_from_ndjson(cx, ndjson);
        }
    }

    /// Parse and set dataflows from JSON string
    pub fn set_from_json(&self, cx: &mut Cx, json: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_from_json(cx, json);
        }
    }

    /// Set loading state
    pub fn set_loading(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_loading(cx);
        }
    }

    /// Set error state
    pub fn set_error(&self, cx: &mut Cx, message: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_error(cx, message);
        }
    }

    /// Clear all dataflows
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }

    /// Check if a DataflowTableAction was triggered
    pub fn action(&self, actions: &Actions) -> Option<DataflowTableAction> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            item.cast()
        } else {
            None
        }
    }

    /// Check if a stop button was clicked, returns the UUID if so
    pub fn stop_clicked(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            let table_list = inner.view.portal_list(ids!(table_list));
            for (item_id, item) in table_list.items_with_actions(actions) {
                if item_id < inner.dataflows.len()
                    && item.button(ids!(stop_button)).clicked(actions)
                {
                    return Some(inner.dataflows[item_id].uuid.clone());
                }
            }
        }
        None
    }

    /// Check if a destroy button was clicked, returns the UUID if so
    pub fn destroy_clicked(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            let table_list = inner.view.portal_list(ids!(table_list));
            for (item_id, item) in table_list.items_with_actions(actions) {
                if item_id < inner.dataflows.len()
                    && item.button(ids!(destroy_button)).clicked(actions)
                {
                    return Some(inner.dataflows[item_id].uuid.clone());
                }
            }
        }
        None
    }

    /// Check if a logs button was clicked, returns the UUID if so
    pub fn logs_clicked(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            let table_list = inner.view.portal_list(ids!(table_list));
            for (item_id, item) in table_list.items_with_actions(actions) {
                if item_id < inner.dataflows.len()
                    && item.button(ids!(logs_button)).clicked(actions)
                {
                    return Some(inner.dataflows[item_id].uuid.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ndjson() {
        let input = r#"{"uuid":"abc","name":"test","status":"Running","nodes":3,"cpu":0.5,"memory":0.036}
{"uuid":"def","name":"test2","status":"Failed","nodes":0,"cpu":0.0,"memory":0.0}"#;

        let dataflows = DataflowInfo::parse_ndjson(input);
        assert_eq!(dataflows.len(), 2);
        assert_eq!(dataflows[0].name, "test");
        assert_eq!(dataflows[0].status, "Running");
        assert_eq!(dataflows[1].status, "Failed");
    }

    #[test]
    fn test_parse_json_array() {
        let input = r#"[
            {"uuid":"abc123","name":"dataflow1","status":"Running","nodes":3,"cpu":25.5,"memory":0.5},
            {"uuid":"def456","name":"dataflow2","status":"Stopped","nodes":0,"cpu":0.0,"memory":0.0}
        ]"#;

        let dataflows = DataflowInfo::parse_json_array(input);
        assert_eq!(dataflows.len(), 2);
        assert_eq!(dataflows[0].uuid, "abc123");
        assert_eq!(dataflows[0].name, "dataflow1");
        assert!(dataflows[0].is_running());
        assert!(!dataflows[1].is_running());
    }

    #[test]
    fn test_parse_json_array_empty() {
        let input = "[]";
        let dataflows = DataflowInfo::parse_json_array(input);
        assert!(dataflows.is_empty());
    }

    #[test]
    fn test_parse_json_array_invalid() {
        let input = "invalid json";
        let dataflows = DataflowInfo::parse_json_array(input);
        assert!(dataflows.is_empty());
    }

    #[test]
    fn test_memory_formatted() {
        let df = DataflowInfo {
            uuid: "test".to_string(),
            name: "test".to_string(),
            status: "Running".to_string(),
            nodes: 1,
            cpu: 0.0,
            memory: 0.036,
        };
        assert_eq!(df.memory_formatted(), "37 MB");

        let df2 = DataflowInfo {
            memory: 1.5,
            ..df.clone()
        };
        assert_eq!(df2.memory_formatted(), "1.50 GB");

        let df3 = DataflowInfo {
            memory: 0.0,
            ..df.clone()
        };
        assert_eq!(df3.memory_formatted(), "0 B");
    }

    #[test]
    fn test_cpu_formatted() {
        let df = DataflowInfo {
            uuid: "test".to_string(),
            name: "test".to_string(),
            status: "Running".to_string(),
            nodes: 1,
            cpu: 45.678,
            memory: 0.0,
        };
        assert_eq!(df.cpu_formatted(), "45.7%");
    }

    #[test]
    fn test_uuid_short() {
        let df = DataflowInfo {
            uuid: "abc123def456789".to_string(),
            name: "test".to_string(),
            status: "Running".to_string(),
            nodes: 1,
            cpu: 0.0,
            memory: 0.0,
        };
        assert_eq!(df.uuid_short(), "abc123de...");

        let df2 = DataflowInfo {
            uuid: "short".to_string(),
            ..df.clone()
        };
        assert_eq!(df2.uuid_short(), "short");
    }

    #[test]
    fn test_is_running() {
        let running = DataflowInfo {
            uuid: "1".to_string(),
            name: "test".to_string(),
            status: "Running".to_string(),
            nodes: 1,
            cpu: 0.0,
            memory: 0.0,
        };
        assert!(running.is_running());

        let stopped = DataflowInfo {
            status: "Stopped".to_string(),
            ..running.clone()
        };
        assert!(!stopped.is_running());

        // Case insensitive
        let running_lower = DataflowInfo {
            status: "running".to_string(),
            ..running.clone()
        };
        assert!(running_lower.is_running());
    }

    #[test]
    fn test_dataflow_info_default() {
        let df = DataflowInfo::default();
        assert!(df.uuid.is_empty());
        assert!(df.name.is_empty());
        assert!(df.status.is_empty());
        assert_eq!(df.nodes, 0);
        assert_eq!(df.cpu, 0.0);
        assert_eq!(df.memory, 0.0);
    }

    #[test]
    fn test_loading_state_default() {
        let state = TableLoadingState::default();
        assert_eq!(state, TableLoadingState::Idle);
    }
}
