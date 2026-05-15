use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::Config;
use dioxus::prelude::*;

mod app;
mod bridge;
mod components;

#[cfg(test)]
mod tests;

pub const MAIN_CSS: Asset = asset!("/assets/main.css");
pub const LOGO_SVG: Asset = asset!("/assets/logo.svg");

fn main() {
    let cfg =
        Config::default().with_window(WindowBuilder::new().with_title("Codex Runtime Dashboard"));
    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(app::App);
}
