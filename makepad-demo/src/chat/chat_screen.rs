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

    // Loading indicator bubble
    LoadingBubble = <View> {
        width: Fill, height: Fit
        flow: Right
        align: { x: 0.0 }
        padding: { left: 16, right: 60, top: 4, bottom: 4 }

        <RoundedView> {
            width: Fit, height: Fit
            draw_bg: { color: (ASSISTANT_BUBBLE_COLOR) }
            padding: { left: 16, right: 16, top: 10, bottom: 10 }

            <Label> {
                width: Fit, height: Fit
                draw_text: {
                    text_style: { font_size: 14.0 }
                    color: #6b7280
                    wrap: Word
                }
                text: "Thinking..."
            }
        }
    }

    pub ChatScreen = {{ChatScreen}} {
        width: Fill, height: Fill
        flow: Down
        show_bg: true
        draw_bg: { color: (BG_COLOR) }

        // Status bar
        status_label = <Label> {
            width: Fill, height: Fit
            padding: { left: 20, top: 8, bottom: 8 }
            draw_text: { color: #6b7280, text_style: { font_size: 12.0 } }
            text: "Ready"
        }

        // Messages area with PortalList for dynamic rendering
        message_list = <PortalList> {
            width: Fill, height: Fill
            flow: Down
            auto_tail: true

            UserBubble = <UserBubble> {}
            AssistantBubble = <AssistantBubble> {}
            LoadingBubble = <LoadingBubble> {}
        }

        // Input area
        <View> {
            width: Fill, height: 72
            show_bg: true
            draw_bg: { color: #ffffff }
            padding: { left: 16, right: 16, top: 12, bottom: 12 }
            flow: Right
            spacing: 12
            align: { y: 0.5 }

            message_input = <TextInput> {
                width: Fill, height: 48
                empty_text: "Type a message..."
                draw_text: {
                    color: #000000
                    uniform color_hover: #000000
                    uniform color_focus: #000000
                    uniform color_down: #000000
                    uniform color_empty: #888888
                }
            }

            send_button = <Button> {
                width: 80, height: 48
                text: "Send"
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
        // Draw the view but handle PortalList specially
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
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
    }
}

impl ChatScreen {
    fn draw_messages(&mut self, cx: &mut Cx2d, list: &mut RefMut<PortalList>) {
        // Calculate total items: messages + loading indicator if loading
        let item_count = self.messages.len() + if self.is_loading { 1 } else { 0 };

        list.set_item_range(cx, 0, item_count);

        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < self.messages.len() {
                // Render actual message
                let msg = &self.messages[item_id];
                let template = match msg.role {
                    MessageRole::User => live_id!(UserBubble),
                    MessageRole::Assistant => live_id!(AssistantBubble),
                };

                let item = list.item(cx, item_id, template);
                item.label(ids!(label)).set_text(cx, &msg.content);
                item.draw_all(cx, &mut Scope::empty());
            } else if self.is_loading && item_id == self.messages.len() {
                // Render loading indicator (only one, right after messages)
                let item = list.item(cx, item_id, live_id!(LoadingBubble));
                item.draw_all(cx, &mut Scope::empty());
            }
        }
    }

    fn update_display(&mut self, cx: &mut Cx) {
        // Update status label
        let status = if self.is_loading {
            "Thinking...".to_string()
        } else {
            format!("{} messages", self.messages.len())
        };
        self.view.label(ids!(status_label)).set_text(cx, &status);
        self.redraw(cx);
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

        // Update display immediately
        self.update_display(cx);

        // Start polling and send request
        self.next_frame = cx.new_next_frame();
        submit_chat_request(self.messages.clone());
    }
}
