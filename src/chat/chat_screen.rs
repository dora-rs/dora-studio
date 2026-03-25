use crate::api::{
    submit_chat_request, take_pending_response, ChatMessage, ChatResponse, MessageRole,
};
use makepad_widgets::*;
use std::cell::RefMut;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // Color palette
    USER_BUBBLE_COLOR = #3b82f6
    ASSISTANT_BUBBLE_COLOR = #e5e7eb
    BG_COLOR = #f9fafb
    HEADER_COLOR = #1e40af

    // User message bubble (right-aligned, blue)
    UserBubble = <View> {
        width: Fill, height: Fit
        flow: Right
        align: { x: 1.0 }
        padding: { left: 60, right: 16, top: 4, bottom: 4 }

        bubble = <RoundedView> {
            width: Fit, height: Fit
            draw_bg: { color: (USER_BUBBLE_COLOR) }
            padding: { left: 16, right: 16, top: 10, bottom: 10 }

            label = <Label> {
                width: Fit, height: Fit
                draw_text: {
                    text_style: { font_size: 14.0 }
                    color: #ffffff
                    wrap: Word
                }
            }
        }
    }

    // Assistant message bubble (left-aligned, gray)
    AssistantBubble = <View> {
        width: Fill, height: Fit
        flow: Right
        align: { x: 0.0 }
        padding: { left: 16, right: 60, top: 4, bottom: 4 }

        bubble = <RoundedView> {
            width: Fit, height: Fit
            draw_bg: { color: (ASSISTANT_BUBBLE_COLOR) }
            padding: { left: 16, right: 16, top: 10, bottom: 10 }

            label = <Label> {
                width: Fit, height: Fit
                draw_text: {
                    text_style: { font_size: 14.0 }
                    color: #1f2937
                    wrap: Word
                }
            }
        }
    }

    pub ChatScreen = {{ChatScreen}} {
        width: Fill, height: Fill
        flow: Down
        show_bg: true
        draw_bg: { color: (BG_COLOR) }

        // Toolbar: status + history / reply toggles + clear
        toolbar_row = <View> {
            width: Fill, height: Fit
            flow: Right
            padding: { left: 16, right: 16, top: 8, bottom: 4 }
            spacing: 8
            align: { y: 0.5 }

            status_label = <Label> {
                width: Fill, height: Fit
                draw_text: { color: #6b7280, text_style: { font_size: 12.0 } }
                text: "Ready"
            }

            history_toggle = <Button> {
                width: Fit, height: Fit
                text: "History ▾"
            }

            response_toggle = <Button> {
                width: Fit, height: Fit
                text: "Reply ▾"
            }

            clear_button = <Button> {
                width: Fit, height: Fit
                text: "Clear"
            }
        }

        // Outer view stays Fill so collapsing the list does not lift the bottom bar.
        history_panel = <View> {
            width: Fill, height: Fill
            flow: Down

            message_list = <PortalList> {
                width: Fill, height: Fill
                flow: Down
                auto_tail: true

                UserBubble = <UserBubble> {}
                AssistantBubble = <AssistantBubble> {}
            }
        }

        // Expandable response strip: typing + latest reply preview
        response_panel = <View> {
            width: Fill, height: Fit
            flow: Down
            padding: { left: 16, right: 16, top: 8, bottom: 8 }
            spacing: 6
            show_bg: true
            draw_bg: { color: #ffffff }

            typing_indicator = <Label> {
                width: Fill, height: Fit
                draw_text: { color: #6b7280, text_style: { font_size: 13.0 } }
                text: ""
            }

            response_preview = <Label> {
                width: Fill, height: Fit
                draw_text: { color: #1f2937, text_style: { font_size: 14.0 }, wrap: Word }
                text: "Latest reply will appear here."
            }
        }

        // Bottom bar: 💬, input (placeholder when empty), ↵
        bottom_bar = <View> {
            width: Fill, height: 72
            show_bg: true
            draw_bg: { color: #ffffff }
            padding: { left: 16, right: 16, top: 12, bottom: 12 }
            flow: Right
            spacing: 10
            align: { y: 0.5 }

            emoji_label = <Label> {
                width: Fit, height: Fit
                draw_text: { text_style: { font_size: 18.0 } }
                text: "💬"
            }

            message_input = <TextInput> {
                width: Fill, height: 48
                empty_text: "Ask AI..."
                padding: { left: 12, right: 12, top: 13, bottom: 13 }
                draw_text: {
                    text_style: { font_size: 14.0 }
                    color: #000000
                    uniform color_hover: #000000
                    uniform color_focus: #000000
                    uniform color_down: #000000
                    uniform color_empty: #888888
                }
            }

            send_button = <Button> {
                width: 52, height: 48
                text: "↵"
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreen {
    #[deref]
    view: View,
    #[rust]
    messages: Vec<ChatMessage>,
    #[rust]
    is_loading: bool,
    #[rust]
    next_frame: NextFrame,
    /// When true, message list is collapsed (height 0).
    #[rust]
    history_collapsed: bool,
    /// When true, response panel is collapsed (height 0).
    #[rust]
    response_collapsed: bool,
}

impl Widget for ChatScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Poll for API responses
        if self.next_frame.is_event(event).is_some() {
            if let Some(resp) = take_pending_response() {
                self.is_loading = false;
                let content = match resp {
                    ChatResponse::Message(s) => s,
                    ChatResponse::ToolExecution(s) => s,
                    ChatResponse::Error(e) => format!("Error: {}", e),
                };
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content,
                });
                self.update_display(cx);
            }
            if self.is_loading {
                self.next_frame = cx.new_next_frame();
            }
        }

        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let mut drew_portal = false;
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                self.draw_messages(cx, &mut list);
                drew_portal = true;
            }
        }
        if !drew_portal {
            if let Some(mut list) = self.view.portal_list(ids!(message_list)).borrow_mut() {
                self.draw_messages(cx, &mut list);
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.view.button(ids!(send_button)).clicked(actions) {
            self.send_message(cx);
        }

        if self
            .view
            .text_input(ids!(message_input))
            .returned(actions)
            .is_some()
        {
            self.send_message(cx);
        }

        if self.view.button(ids!(clear_button)).clicked(actions) {
            self.clear_chat(cx);
        }

        if self.view.button(ids!(history_toggle)).clicked(actions) {
            self.history_collapsed = !self.history_collapsed;
            self.update_display(cx);
        }

        if self.view.button(ids!(response_toggle)).clicked(actions) {
            self.response_collapsed = !self.response_collapsed;
            self.update_display(cx);
        }
    }
}

impl ChatScreen {
    fn draw_messages(&mut self, cx: &mut Cx2d, list: &mut RefMut<PortalList>) {
        if self.history_collapsed {
            list.set_item_range(cx, 0, 0);
            return;
        }

        let item_count = self.messages.len();
        list.set_item_range(cx, 0, item_count);

        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < self.messages.len() {
                let msg = &self.messages[item_id];
                let template = match msg.role {
                    MessageRole::User => live_id!(UserBubble),
                    MessageRole::Assistant => live_id!(AssistantBubble),
                };

                let item = list.item(cx, item_id, template);
                item.label(ids!(label)).set_text(cx, &msg.content);
                item.draw_all(cx, &mut Scope::empty());
            }
        }
    }

    fn last_assistant_preview(&self) -> Option<String> {
        for m in self.messages.iter().rev() {
            if matches!(m.role, MessageRole::Assistant) {
                let c = m.content.trim();
                if c.is_empty() {
                    continue;
                }
                const MAX: usize = 600;
                if c.len() > MAX {
                    return Some(format!("{}…", &c[..MAX]));
                }
                return Some(c.to_string());
            }
        }
        None
    }

    fn sync_panel_heights(&mut self, cx: &mut Cx) {
        if self.history_collapsed {
            self.view
                .view(ids!(message_list))
                .apply_over(cx, live! { height: 0 });
        } else {
            self.view
                .view(ids!(message_list))
                .apply_over(cx, live! { height: Fill });
        }

        if self.response_collapsed {
            self.view
                .view(ids!(response_panel))
                .apply_over(cx, live! { height: 0 });
        } else {
            self.view
                .view(ids!(response_panel))
                .apply_over(cx, live! { height: Fit });
        }
    }

    fn update_toggle_labels(&mut self, cx: &mut Cx) {
        let h = if self.history_collapsed {
            "History ▸"
        } else {
            "History ▾"
        };
        let r = if self.response_collapsed {
            "Reply ▸"
        } else {
            "Reply ▾"
        };
        self.view.button(ids!(history_toggle)).set_text(cx, h);
        self.view.button(ids!(response_toggle)).set_text(cx, r);
    }

    fn update_display(&mut self, cx: &mut Cx) {
        let status = if self.is_loading {
            "Waiting for AI…".to_string()
        } else {
            format!("{} messages", self.messages.len())
        };
        self.view.label(ids!(status_label)).set_text(cx, &status);

        let typing = if self.is_loading {
            "AI is typing…"
        } else {
            ""
        };
        self.view.label(ids!(typing_indicator)).set_text(cx, typing);

        let preview = if let Some(p) = self.last_assistant_preview() {
            p
        } else if self.is_loading {
            "Awaiting reply…".to_string()
        } else {
            "Latest reply will appear here.".to_string()
        };
        self.view
            .label(ids!(response_preview))
            .set_text(cx, &preview);

        self.sync_panel_heights(cx);
        self.update_toggle_labels(cx);

        self.view.portal_list(ids!(message_list)).redraw(cx);
        self.redraw(cx);
    }

    fn clear_chat(&mut self, cx: &mut Cx) {
        self.messages.clear();
        self.is_loading = false;
        self.view.text_input(ids!(message_input)).set_text(cx, "");
        self.update_display(cx);
    }

    fn send_message(&mut self, cx: &mut Cx) {
        let input = self.view.text_input(ids!(message_input));
        let text = input.text();
        if text.trim().is_empty() {
            return;
        }

        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: text.clone(),
        });

        input.set_text(cx, "");
        self.is_loading = true;

        self.update_display(cx);

        self.next_frame = cx.new_next_frame();
        submit_chat_request(self.messages.clone());
    }
}
