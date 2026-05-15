use crate::bridge::ui_state::UIRuntimeState;
use dioxus::prelude::*;

/// Initialize and provide UIRuntimeState context for the entire UI tree.
/// Call this once at the start of your App component.
pub fn provide_ui_state() -> Signal<UIRuntimeState> {
    let app_state: Signal<UIRuntimeState> = use_signal(UIRuntimeState::new);
    provide_context(app_state);
    app_state
}

/// Helper hook to consume UIRuntimeState from context.
/// Use this in any child component: `let state = use_ui_runtime_state();`
pub fn use_ui_runtime_state() -> Signal<UIRuntimeState> {
    consume_context::<Signal<UIRuntimeState>>()
}

/// Legacy StateProvider component - wraps children with state context
#[component]
pub fn StateProvider(children: Element) -> Element {
    let _state = provide_ui_state();
    rsx! {
        {children}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]  // Requires Dioxus runtime - Signals cannot be created in unit tests. Test in UI context instead.
    fn state_creation_works() {
        let state = UIRuntimeState::new();
        assert!(state.timeline_events.read().is_empty());
    }
}
