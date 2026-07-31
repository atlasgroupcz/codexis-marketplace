use crate::error::CdxctlError;

const AUTOMATION_ENV: &str = "CODEXIS_PUBLIC_AUTOMATION";

/// Refuse the given action when running inside an automation run, detected via the
/// `CODEXIS_PUBLIC_AUTOMATION` environment variable injected by the daemon. `reason`
/// says why that action in particular cannot run unattended.
pub fn refuse_if_in_automation(action: &str, reason: &str) -> Result<(), CdxctlError> {
    refuse(
        std::env::var(AUTOMATION_ENV).ok().as_deref(),
        action,
        reason,
    )
}

fn refuse(value: Option<&str>, action: &str, reason: &str) -> Result<(), CdxctlError> {
    if value == Some("true") {
        return Err(CdxctlError::Refused(format!(
            "{action} is not allowed from inside an automation run \
             ({AUTOMATION_ENV}=true). {reason}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTOMATION_REASON: &str = "An automation must not create more automations.";

    #[test]
    fn refuses_when_inside_automation() {
        let result = refuse(Some("true"), "automation create", AUTOMATION_REASON);
        assert!(matches!(result, Err(CdxctlError::Refused(_))));
    }

    #[test]
    fn allows_when_env_absent() {
        assert!(refuse(None, "automation create", AUTOMATION_REASON).is_ok());
    }

    #[test]
    fn allows_when_env_not_true() {
        assert!(refuse(Some("false"), "automation create", AUTOMATION_REASON).is_ok());
        assert!(refuse(Some(""), "automation create", AUTOMATION_REASON).is_ok());
        assert!(refuse(Some("1"), "automation create", AUTOMATION_REASON).is_ok());
    }

    #[test]
    fn the_refusal_says_which_action_and_why_it_cannot_run_unattended() {
        let error = refuse(
            Some("true"),
            "workflow start",
            "A workflow run needs the user in the chat to approve it.",
        )
        .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("workflow start"));
        assert!(message.contains("CODEXIS_PUBLIC_AUTOMATION=true"));
        assert!(message.contains("needs the user in the chat to approve it"));
    }
}
