use dioxus::prelude::*;

#[component]
pub fn ChatInput(on_send: EventHandler<String>) -> Element {
    let mut input = use_signal(String::new);

    rsx! {
        div { class: "chat-input-row",
            textarea {
                class: "chat-input",
                value: "{input()}",
                rows: "3",
                placeholder: "Type a message for Codex...",
                oninput: move |evt| {
                    input.set(evt.value());
                }
            }
            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    let trimmed = input().trim().to_string();
                    if !trimmed.is_empty() {
                        on_send.call(trimmed);
                        input.set(String::new());
                    }
                },
                "Send"
            }
        }
        p { class: "muted", "Run through CODEX runtime bridge (bounded mode)." }
    }
}
