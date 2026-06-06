use crate::error::CdxctlError;

const AUTOMATION_ENV: &str = "CODEXIS_PUBLIC_AUTOMATION";

/// Refuse the given action when running inside an automation run, detected via the
/// `CODEXIS_PUBLIC_AUTOMATION` environment variable injected by the daemon.
pub fn refuse_if_in_automation(action: &str) -> Result<(), CdxctlError> {
    refuse(std::env::var(AUTOMATION_ENV).ok().as_deref(), action)
}

fn refuse(value: Option<&str>, action: &str) -> Result<(), CdxctlError> {
    if value == Some("true") {
        return Err(CdxctlError::Refused(format!(
            "{action} is not allowed from inside an automation run \
             ({AUTOMATION_ENV}=true). An automation must not create more automations."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_when_inside_automation() {
        let result = refuse(Some("true"), "automation create");
        assert!(matches!(result, Err(CdxctlError::Refused(_))));
    }

    #[test]
    fn allows_when_env_absent() {
        assert!(refuse(None, "automation create").is_ok());
    }

    #[test]
    fn allows_when_env_not_true() {
        assert!(refuse(Some("false"), "automation create").is_ok());
        assert!(refuse(Some(""), "automation create").is_ok());
        assert!(refuse(Some("1"), "automation create").is_ok());
    }
}
