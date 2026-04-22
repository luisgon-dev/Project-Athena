use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{
    app::AppState,
    domain::settings::{
        AcquisitionSettingsRecord, AcquisitionSettingsUpdate, AudiobookshelfIntegrationRecord,
        AudiobookshelfIntegrationUpdate, ConnectionTestResult, ImportSettingsRecord,
        ImportSettingsUpdate, NotificationSettingsRecord, NotificationSettingsUpdate,
        PersistedRuntimeSettings, ProwlarrIntegrationRecord, ProwlarrIntegrationUpdate,
        QbittorrentSettingsRecord, QbittorrentSettingsUpdate, RuntimeSettingsRecord,
        RuntimeSettingsUpdate, StorageSettingsRecord, StorageSettingsUpdate, SyncedIndexerRecord,
    },
    downloads::qbittorrent::QbittorrentClient,
    http::{auth::require_admin, error::AppError},
    sync::audiobookshelf::AudiobookshelfClient,
};

pub async fn get_runtime_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RuntimeSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(state.settings.get_runtime_settings().await?))
}

pub async fn update_runtime_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RuntimeSettingsUpdate>,
) -> Result<Json<RuntimeSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(
        state
            .settings
            .update_runtime_settings(payload)
            .await
            .map_err(map_settings_error)?,
    ))
}

pub async fn get_storage_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<StorageSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(state.settings.get_runtime_settings().await?.storage))
}

pub async fn update_storage_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<StorageSettingsUpdate>,
) -> Result<Json<StorageSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            storage: Some(payload),
            ..RuntimeSettingsUpdate::default()
        })
        .await
        .map_err(map_settings_error)?;
    Ok(Json(settings.storage))
}

pub async fn get_import_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ImportSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(state.settings.get_runtime_settings().await?.import))
}

pub async fn update_import_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ImportSettingsUpdate>,
) -> Result<Json<ImportSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            import: Some(payload),
            ..RuntimeSettingsUpdate::default()
        })
        .await
        .map_err(map_settings_error)?;
    Ok(Json(settings.import))
}

pub async fn get_acquisition_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AcquisitionSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(
        state.settings.get_runtime_settings().await?.acquisition,
    ))
}

pub async fn update_acquisition_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AcquisitionSettingsUpdate>,
) -> Result<Json<AcquisitionSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            acquisition: Some(payload),
            ..RuntimeSettingsUpdate::default()
        })
        .await
        .map_err(map_settings_error)?;
    Ok(Json(settings.acquisition))
}

pub async fn get_qbittorrent_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<QbittorrentSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
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
    headers: HeaderMap,
    Json(payload): Json<QbittorrentSettingsUpdate>,
) -> Result<Json<QbittorrentSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            download_clients: Some(crate::domain::settings::DownloadClientSettingsUpdate {
                qbittorrent: Some(payload),
            }),
            ..RuntimeSettingsUpdate::default()
        })
        .await
        .map_err(map_settings_error)?;
    Ok(Json(settings.download_clients.qbittorrent))
}

pub async fn test_qbittorrent_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<QbittorrentSettingsUpdate>,
) -> Result<(StatusCode, Json<ConnectionTestResult>), AppError> {
    require_admin(&state, &headers).await?;
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
    headers: HeaderMap,
) -> Result<Json<ProwlarrIntegrationRecord>, AppError> {
    require_admin(&state, &headers).await?;
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
    headers: HeaderMap,
    Json(payload): Json<ProwlarrIntegrationUpdate>,
) -> Result<Json<ProwlarrIntegrationRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            integrations: Some(crate::domain::settings::IntegrationSettingsUpdate {
                prowlarr: Some(payload),
                ..Default::default()
            }),
            ..RuntimeSettingsUpdate::default()
        })
        .await
        .map_err(map_settings_error)?;
    Ok(Json(settings.integrations.prowlarr))
}

pub async fn get_audiobookshelf_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AudiobookshelfIntegrationRecord>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(
        state
            .settings
            .get_runtime_settings()
            .await?
            .integrations
            .audiobookshelf,
    ))
}

pub async fn update_audiobookshelf_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AudiobookshelfIntegrationUpdate>,
) -> Result<Json<AudiobookshelfIntegrationRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            integrations: Some(crate::domain::settings::IntegrationSettingsUpdate {
                audiobookshelf: Some(payload),
                ..Default::default()
            }),
            ..RuntimeSettingsUpdate::default()
        })
        .await
        .map_err(map_settings_error)?;
    Ok(Json(settings.integrations.audiobookshelf))
}

pub async fn test_prowlarr_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ProwlarrIntegrationUpdate>,
) -> Result<(StatusCode, Json<ConnectionTestResult>), AppError> {
    require_admin(&state, &headers).await?;
    let settings = preview_settings(
        &state,
        RuntimeSettingsUpdate {
            integrations: Some(crate::domain::settings::IntegrationSettingsUpdate {
                prowlarr: Some(payload),
                ..Default::default()
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
    headers: HeaderMap,
) -> Result<Json<Vec<SyncedIndexerRecord>>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(state.settings.list_synced_indexers().await?))
}

pub async fn test_audiobookshelf_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AudiobookshelfIntegrationUpdate>,
) -> Result<(StatusCode, Json<ConnectionTestResult>), AppError> {
    require_admin(&state, &headers).await?;
    let settings = preview_settings(
        &state,
        RuntimeSettingsUpdate {
            integrations: Some(crate::domain::settings::IntegrationSettingsUpdate {
                audiobookshelf: Some(payload),
                ..Default::default()
            }),
            ..RuntimeSettingsUpdate::default()
        },
    )
    .await?;
    let audiobookshelf = settings.integrations.audiobookshelf;

    if !audiobookshelf.enabled {
        return Ok((
            StatusCode::OK,
            Json(ConnectionTestResult {
                ok: true,
                message: "Audiobookshelf integration is disabled".to_string(),
            }),
        ));
    }

    let client = AudiobookshelfClient::new(
        audiobookshelf.base_url,
        audiobookshelf.api_key.unwrap_or_default(),
    );
    client
        .scan_library(&audiobookshelf.library_id)
        .send()
        .await
        .map_err(anyhow::Error::from)?
        .error_for_status()
        .map_err(anyhow::Error::from)?;

    Ok((
        StatusCode::OK,
        Json(ConnectionTestResult {
            ok: true,
            message: "Audiobookshelf connection succeeded".to_string(),
        }),
    ))
}

pub async fn get_notifications_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<NotificationSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(state.settings.get_runtime_settings().await?.notifications))
}

pub async fn update_notifications_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<NotificationSettingsUpdate>,
) -> Result<Json<NotificationSettingsRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            notifications: Some(payload),
            ..RuntimeSettingsUpdate::default()
        })
        .await
        .map_err(map_settings_error)?;
    Ok(Json(settings.notifications))
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
    settings.validate().map_err(map_settings_error)?;
    Ok(settings)
}

fn map_settings_error(error: anyhow::Error) -> AppError {
    AppError::BadRequest(error.to_string())
}
