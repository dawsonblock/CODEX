use super::types::{RuntimeCommand, RuntimeCommandResult, RuntimeCommandStatus};

#[derive(Debug, Clone)]
pub struct RuntimeTransport {
    disabled: bool,
}

impl RuntimeTransport {
    pub fn new_disabled() -> Self {
        Self { disabled: true }
    }
}

pub fn send_runtime_command(
    transport: &RuntimeTransport,
    command: &RuntimeCommand,
) -> RuntimeCommandResult {
    let message = if transport.disabled {
        "Runtime command bridge not enabled in UI v1.".to_string()
    } else {
        "Command accepted in dry-run mode.".to_string()
    };

    RuntimeCommandResult {
        command: command.clone(),
        status: if transport.disabled {
            RuntimeCommandStatus::Disabled
        } else {
            RuntimeCommandStatus::DryRunOnly
        },
        message,
    }
}
