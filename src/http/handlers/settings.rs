use axum::{Json, extract::State, http::StatusCode};

use crate::{
    app::AppState,
    domain::settings::{
        AcquisitionSettingsRecord, AcquisitionSettingsUpdate, ConnectionTestResult,
        ImportSettingsRecord, ImportSettingsUpdate, PersistedRuntimeSettings,
        ProwlarrIntegrationRecord, ProwlarrIntegrationUpdate, QbittorrentSettingsRecord,
        QbittorrentSettingsUpdate, RuntimeSettingsRecord, RuntimeSettingsUpdate,
        StorageSettingsRecord, StorageSettingsUpdate, SyncedIndexerRecord,
    },
    downloads::qbittorrent::QbittorrentClient,
    http::error::AppError,
};

pub async fn get_runtime_settings(
    State(state): State<AppState>,
) -> Result<Json<RuntimeSettingsRecord>, AppError> {
    Ok(Json(state.settings.get_runtime_settings().await?))
}

pub async fn update_runtime_settings(
    State(state): State<AppState>,
    Json(payload): Json<RuntimeSettingsUpdate>,
) -> Result<Json<RuntimeSettingsRecord>, AppError> {
    Ok(Json(state.settings.update_runtime_settings(payload).await?))
}

pub async fn get_storage_settings(
    State(state): State<AppState>,
) -> Result<Json<StorageSettingsRecord>, AppError> {
    Ok(Json(state.settings.get_runtime_settings().await?.storage))
}

pub async fn update_storage_settings(
    State(state): State<AppState>,
    Json(payload): Json<StorageSettingsUpdate>,
) -> Result<Json<StorageSettingsRecord>, AppError> {
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            storage: Some(payload),
            ..RuntimeSettingsUpdate::default()
        })
        .await?;
    Ok(Json(settings.storage))
}

pub async fn get_import_settings(
    State(state): State<AppState>,
) -> Result<Json<ImportSettingsRecord>, AppError> {
    Ok(Json(state.settings.get_runtime_settings().await?.import))
}

pub async fn update_import_settings(
    State(state): State<AppState>,
    Json(payload): Json<ImportSettingsUpdate>,
) -> Result<Json<ImportSettingsRecord>, AppError> {
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            import: Some(payload),
            ..RuntimeSettingsUpdate::default()
        })
        .await?;
    Ok(Json(settings.import))
}

pub async fn get_acquisition_settings(
    State(state): State<AppState>,
) -> Result<Json<AcquisitionSettingsRecord>, AppError> {
    Ok(Json(
        state.settings.get_runtime_settings().await?.acquisition,
    ))
}

pub async fn update_acquisition_settings(
    State(state): State<AppState>,
    Json(payload): Json<AcquisitionSettingsUpdate>,
) -> Result<Json<AcquisitionSettingsRecord>, AppError> {
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            acquisition: Some(payload),
            ..RuntimeSettingsUpdate::default()
        })
        .await?;
    Ok(Json(settings.acquisition))
}

pub async fn get_qbittorrent_settings(
    State(state): State<AppState>,
) -> Result<Json<QbittorrentSettingsRecord>, AppError> {
    Ok(Json(
        state
            .settings
            .get_runtime_settings()
            .await?
            .download_clients
            .qbittorrent,
    ))
}

pub async fn update_qbittorrent_settings(
    State(state): State<AppState>,
    Json(payload): Json<QbittorrentSettingsUpdate>,
) -> Result<Json<QbittorrentSettingsRecord>, AppError> {
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            download_clients: Some(crate::domain::settings::DownloadClientSettingsUpdate {
                qbittorrent: Some(payload),
            }),
            ..RuntimeSettingsUpdate::default()
        })
        .await?;
    Ok(Json(settings.download_clients.qbittorrent))
}

pub async fn test_qbittorrent_settings(
    State(state): State<AppState>,
    Json(payload): Json<QbittorrentSettingsUpdate>,
) -> Result<(StatusCode, Json<ConnectionTestResult>), AppError> {
    let settings = preview_settings(
        &state,
        RuntimeSettingsUpdate {
            download_clients: Some(crate::domain::settings::DownloadClientSettingsUpdate {
                qbittorrent: Some(payload),
            }),
            ..RuntimeSettingsUpdate::default()
        },
    )
    .await?;
    let qb = settings.download_clients.qbittorrent;

    if !qb.enabled {
        return Ok((
            StatusCode::OK,
            Json(ConnectionTestResult {
                ok: true,
                message: "qBittorrent is disabled".to_string(),
            }),
        ));
    }

    let client = QbittorrentClient::new(qb.base_url, qb.username, qb.password.unwrap_or_default());
    client.list(None).await.map_err(AppError::from)?;

    Ok((
        StatusCode::OK,
        Json(ConnectionTestResult {
            ok: true,
            message: "qBittorrent connection succeeded".to_string(),
        }),
    ))
}

pub async fn get_prowlarr_settings(
    State(state): State<AppState>,
) -> Result<Json<ProwlarrIntegrationRecord>, AppError> {
    Ok(Json(
        state
            .settings
            .get_runtime_settings()
            .await?
            .integrations
            .prowlarr,
    ))
}

pub async fn update_prowlarr_settings(
    State(state): State<AppState>,
    Json(payload): Json<ProwlarrIntegrationUpdate>,
) -> Result<Json<ProwlarrIntegrationRecord>, AppError> {
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            integrations: Some(crate::domain::settings::IntegrationSettingsUpdate {
                prowlarr: Some(payload),
            }),
            ..RuntimeSettingsUpdate::default()
        })
        .await?;
    Ok(Json(settings.integrations.prowlarr))
}

pub async fn test_prowlarr_settings(
    State(state): State<AppState>,
    Json(payload): Json<ProwlarrIntegrationUpdate>,
) -> Result<(StatusCode, Json<ConnectionTestResult>), AppError> {
    let settings = preview_settings(
        &state,
        RuntimeSettingsUpdate {
            integrations: Some(crate::domain::settings::IntegrationSettingsUpdate {
                prowlarr: Some(payload),
            }),
            ..RuntimeSettingsUpdate::default()
        },
    )
    .await?;
    let prowlarr = settings.integrations.prowlarr;

    if !prowlarr.enabled && !prowlarr.sync_enabled {
        return Ok((
            StatusCode::OK,
            Json(ConnectionTestResult {
                ok: true,
                message: "Prowlarr integration is disabled".to_string(),
            }),
        ));
    }

    let response = reqwest::Client::new()
        .get(format!(
            "{}/api/v1/system/status",
            prowlarr.base_url.trim_end_matches('/')
        ))
        .header("X-Api-Key", prowlarr.api_key.unwrap_or_default())
        .send()
        .await
        .map_err(anyhow::Error::from)?
        .error_for_status()
        .map_err(anyhow::Error::from)?;

    let status: serde_json::Value = response.json().await.map_err(anyhow::Error::from)?;
    let version = status
        .get("version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");

    Ok((
        StatusCode::OK,
        Json(ConnectionTestResult {
            ok: true,
            message: format!("Prowlarr connection succeeded (version {version})"),
        }),
    ))
}

pub async fn list_synced_indexers(
    State(state): State<AppState>,
) -> Result<Json<Vec<SyncedIndexerRecord>>, AppError> {
    Ok(Json(state.settings.list_synced_indexers().await?))
}

async fn preview_settings(
    state: &AppState,
    update: RuntimeSettingsUpdate,
) -> Result<PersistedRuntimeSettings, AppError> {
    let mut settings = state
        .settings
        .get_persisted_runtime_settings()
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("runtime settings row missing")))?;
    settings.apply_update(update);
    settings.validate()?;
    Ok(settings)
}
