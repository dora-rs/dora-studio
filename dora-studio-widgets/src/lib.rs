pub use makepad_widgets;

pub mod chat_screen;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    chat_screen::live_design(cx);
}
