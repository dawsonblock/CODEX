use crate::bridge::types::ChatMessage;
use dioxus::prelude::*;

use super::message_bubble::MessageBubble;

#[component]
pub fn ChatView(
    messages: Vec<ChatMessage>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        section { class: "card chat-view",
            h3 { "Chat" }
            div { class: "chat-thread",
                if messages.is_empty() {
                    p { class: "muted", "Start a chat by sending a message." }
                } else {
                    for msg in messages {
                        MessageBubble {
                            selected: selected_id.as_ref().is_some_and(|id| id == &msg.id),
                            on_select: on_select,
                            message: msg,
                        }
                    }
                }
            }
        }
    }
}
