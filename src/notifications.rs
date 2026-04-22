use anyhow::{Context, Result};
use serde_json::Value;

use crate::{
    db::repositories::SqliteSettingsRepository,
    domain::settings::{NotificationTargetKind, PersistedNotificationSettings},
};

pub async fn send_notification(
    settings_repo: &SqliteSettingsRepository,
    event: &str,
    title: &str,
    body: &str,
    data: Value,
) -> Result<()> {
    let settings = settings_repo
        .get_persisted_runtime_settings()
        .await?
        .context("runtime settings row missing")?;
    dispatch_notification(&settings.notifications, event, title, body, data).await
}

async fn dispatch_notification(
    settings: &PersistedNotificationSettings,
    event: &str,
    title: &str,
    body: &str,
    data: Value,
) -> Result<()> {
    if !settings.enabled || settings.target_url.trim().is_empty() {
        return Ok(());
    }

    let client = reqwest::Client::new();
    let mut request = client.post(&settings.target_url);

    if let Some(auth_token) = settings.auth_token.as_deref() {
        let header_name = settings
            .auth_header
            .clone()
            .unwrap_or_else(|| "Authorization".to_string());
        let header_value = if header_name.eq_ignore_ascii_case("authorization") {
            format!("Bearer {auth_token}")
        } else {
            auth_token.to_string()
        };
        request = request.header(&header_name, header_value);
    }

    match settings.target_kind {
        NotificationTargetKind::Webhook => {
            request = request.json(&serde_json::json!({
                "event": event,
                "title": title,
                "body": body,
                "data": data,
            }));
        }
        NotificationTargetKind::Ntfy => {
            request = request
                .header("Title", title)
                .header("Tags", "books")
                .body(format!("{body}\n\n{}", data));
        }
    }

    request.send().await?.error_for_status()?;
    Ok(())
}
