use makepad_widgets::Cx;

pub mod chat_screen;

/// Register the chat module's widgets with the Makepad framework.
///
/// This must be called during application initialization to ensure `ChatScreen`
/// and its associated UI components are available.
///
/// # Examples
///
/// ```rust
/// use makepad_widgets::Cx;
///
/// pub fn app_register(cx: &mut Cx) {
///     dora_studio::chat::live_design(cx);
/// }
/// ```
pub fn live_design(cx: &mut Cx) {
    self::chat_screen::live_design(cx);
}
