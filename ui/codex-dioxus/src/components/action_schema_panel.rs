use crate::bridge::types::ACTION_SCHEMA;
use dioxus::prelude::*;

pub const ACTIONS: [&str; 10] = ACTION_SCHEMA;

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
