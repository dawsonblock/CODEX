use super::types::{
    CommandApprovalState, RuntimeCommand, RuntimeCommandResult, RuntimeCommandStatus,
};

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
    approval: CommandApprovalState,
) -> RuntimeCommandResult {
    if approval != CommandApprovalState::Approved {
        return RuntimeCommandResult {
            command: command.clone(),
            status: RuntimeCommandStatus::AwaitingApproval,
            message: "Command blocked: approval required before dry-run.".to_string(),
        };
    }

    let message = if transport.disabled {
        "Runtime transport disabled; command accepted only as approved dry-run intent.".to_string()
    } else {
        "Command accepted in dry-run mode.".to_string()
    };

    RuntimeCommandResult {
        command: command.clone(),
        status: if transport.disabled {
            RuntimeCommandStatus::ApprovedDryRun
        } else {
            RuntimeCommandStatus::DryRunOnly
        },
        message,
    }
}

pub fn request_command_approval(state: CommandApprovalState) -> CommandApprovalState {
    match state {
        CommandApprovalState::Draft => CommandApprovalState::AwaitingApproval,
        other => other,
    }
}

pub fn grant_command_approval(state: CommandApprovalState) -> CommandApprovalState {
    match state {
        CommandApprovalState::AwaitingApproval => CommandApprovalState::Approved,
        other => other,
    }
}

pub fn reset_command_approval() -> CommandApprovalState {
    CommandApprovalState::Draft
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_flow_transitions() {
        let draft = CommandApprovalState::Draft;
        let requested = request_command_approval(draft);
        let approved = grant_command_approval(requested);
        assert_eq!(requested, CommandApprovalState::AwaitingApproval);
        assert_eq!(approved, CommandApprovalState::Approved);
        assert_eq!(reset_command_approval(), CommandApprovalState::Draft);
    }

    #[test]
    fn command_blocks_without_approval() {
        let transport = RuntimeTransport::new_disabled();
        let result = send_runtime_command(
            &transport,
            &RuntimeCommand::ReplayLast,
            CommandApprovalState::Draft,
        );
        assert_eq!(result.status, RuntimeCommandStatus::AwaitingApproval);
    }
}
