//! katastr notify settings - thin wrappers over the shared per-app store.

pub use cdx_notify::notify_settings::NotifyConfig;
use cdx_notify::notify_settings::NotifySettings;

use crate::error::Result;

const APP_ID: &str = "katastr";

fn store() -> NotifySettings {
    NotifySettings::new(APP_ID)
}

pub fn get_notify() -> NotifyConfig {
    store().get_notify()
}

pub fn set_notify(
    email: Option<bool>,
    in_app: Option<bool>,
    recipients: Option<Vec<String>>,
) -> Result<NotifyConfig> {
    Ok(store().set_notify(email, in_app, recipients)?)
}

pub fn ensure_settings() -> Result<()> {
    Ok(store().ensure_settings()?)
}
