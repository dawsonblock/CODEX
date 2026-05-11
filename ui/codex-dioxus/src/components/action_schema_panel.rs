use dioxus::prelude::*;

pub const ACTIONS: [&str; 10] = [
    "answer",
    "ask_clarification",
    "retrieve_memory",
    "refuse_unsafe",
    "defer_insufficient_evidence",
    "summarize",
    "plan",
    "execute_bounded_tool",
    "no_op",
    "internal_diagnostic",
];

#[component]
pub fn ActionSchemaPanel() -> Element {
    rsx! {
        section { class: "card",
            h3 { "10-Action Schema" }
            ul { class: "list",
                for action in ACTIONS {
                    li { "{action}" }
                }
            }
        }
    }
}
