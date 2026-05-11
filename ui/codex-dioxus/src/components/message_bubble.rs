use crate::bridge::types::{ChatMessage, ChatRole};
use dioxus::prelude::*;

#[component]
pub fn MessageBubble(
    message: ChatMessage,
    selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let role_class = match message.role {
        ChatRole::User => "bubble-user",
        ChatRole::Codex => "bubble-codex",
        ChatRole::System => "bubble-system",
    };
    let selected_class = if selected { "bubble-selected" } else { "" };
    let id = message.id.clone();

    rsx! {
        div {
            class: "message-wrap {role_class}",
            onclick: move |_| on_select.call(id.clone()),
            div { class: "message-bubble {selected_class}",
                p { class: "message-content", "{message.content}" }
                div { class: "message-meta",
                    span { "{message.timestamp}" }
                    if let Some(trace) = &message.runtime {
                        span { "action: {trace.selected_action}" }
                        span { "metadata: {trace.metadata_quality.label()}" }
                    }
                }
            }
        }
    }
}
