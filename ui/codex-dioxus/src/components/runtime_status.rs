use dioxus::prelude::*;

use crate::bridge::types::ProofManifest;

#[component]
pub fn RuntimeStatusPanel(manifest: Option<ProofManifest>) -> Element {
    let manifest = manifest.unwrap_or_default();
    rsx! {
        section { class: "card",
            h3 { "Runtime Status" }
            div { class: "kv",
                div { class: "k", "internal codename" }
                div { "CODEX-main 32" }

                div { class: "k", "runtime authority" }
                div { "Rust-authoritative runtime" }

                div { class: "k", "python status" }
                div { "legacy/reference only" }

                div { class: "k", "proof status" }
                div {
                    if manifest.rust_verified { "verified" } else { "unknown" }
                    span { class: if manifest.rust_verified { "badge ok" } else { "badge warn" },
                        if manifest.rust_verified { "pass" } else { "pending" }
                    }
                }

                div { class: "k", "action schema" }
                div { "10 actions (fixed)" }

                div { class: "k", "memvid" }
                div { "inactive/stubbed" }

                div { class: "k", "tool execution" }
                div { "no real autonomous external tool executor" }
            }
            p { class: "muted", "Not sentient. Not conscious. Not AGI. Not production-ready." }
        }
    }
}
