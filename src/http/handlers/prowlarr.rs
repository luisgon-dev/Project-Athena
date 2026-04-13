use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{app::AppState, http::error::AppError};

pub async fn get_system_status() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "appName": "Project Athena",
        "instanceName": "Project Athena",
        "authentication": "none"
    }))
}

pub async fn get_indexers(State(state): State<AppState>) -> Result<Json<Vec<Value>>, AppError> {
    Ok(Json(state.settings.list_synced_indexer_resources().await?))
}

pub async fn get_indexer_schema() -> Json<Vec<Value>> {
    Json(vec![torznab_schema(), newznab_schema()])
}

pub async fn get_indexer(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let resource = state
        .settings
        .get_synced_indexer_resource(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Indexer with ID {id} not found")))?;

    Ok(Json(resource))
}

pub async fn post_indexer(
    State(state): State<AppState>,
    Query(_query): Query<ForceSaveQuery>,
    Json(payload): Json<Value>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    validate_indexer_payload(&payload)?;
    let resource = state
        .settings
        .create_synced_indexer_resource(&payload)
        .await?;
    Ok((StatusCode::CREATED, Json(resource)))
}

pub async fn update_indexer(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(_query): Query<ForceSaveQuery>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    validate_indexer_payload(&payload)?;
    let resource = state
        .settings
        .update_synced_indexer_resource(id, &payload)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Indexer with ID {id} not found")))?;
    Ok(Json(resource))
}

pub async fn delete_indexer(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    if !state.settings.delete_synced_indexer(id).await? {
        return Err(AppError::NotFound(format!(
            "Indexer with ID {id} not found"
        )));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_indexer(
    _state: State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<StatusCode, AppError> {
    validate_indexer_payload(&payload)?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize, Default)]
pub struct ForceSaveQuery {
    #[serde(rename = "forceSave")]
    pub force_save: Option<bool>,
}

fn validate_indexer_payload(payload: &Value) -> Result<(), AppError> {
    let fields = payload
        .get("fields")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("indexer payload must include fields".to_string()))?;

    let implementation = payload
        .get("implementation")
        .and_then(Value::as_str)
        .unwrap_or("Torznab");

    if implementation != "Torznab" && implementation != "Newznab" {
        return Err(AppError::BadRequest(
            "only Torznab and Newznab indexers are supported".to_string(),
        ));
    }

    if field_value(fields, "baseUrl").is_none() {
        return Err(AppError::BadRequest(
            "indexer payload must include a baseUrl field".to_string(),
        ));
    }

    if field_value(fields, "categories").is_none() {
        return Err(AppError::BadRequest(
            "indexer payload must include a categories field".to_string(),
        ));
    }

    Ok(())
}

fn field_value<'a>(fields: &'a [Value], name: &str) -> Option<&'a Value> {
    fields
        .iter()
        .find(|field| field.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|field| field.get("value"))
}

fn torznab_schema() -> Value {
    json!({
        "id": 0,
        "name": "Torznab",
        "implementationName": "Torznab",
        "implementation": "Torznab",
        "configContract": "TorznabSettings",
        "enableRss": true,
        "enableAutomaticSearch": true,
        "enableInteractiveSearch": true,
        "priority": 25,
        "fields": schema_fields("torrent")
    })
}

fn newznab_schema() -> Value {
    json!({
        "id": 0,
        "name": "Newznab",
        "implementationName": "Newznab",
        "implementation": "Newznab",
        "configContract": "NewznabSettings",
        "enableRss": true,
        "enableAutomaticSearch": true,
        "enableInteractiveSearch": true,
        "priority": 25,
        "fields": schema_fields("usenet")
    })
}

fn schema_fields(protocol: &str) -> Vec<Value> {
    let mut fields = vec![
        json!({
            "name": "baseUrl",
            "value": "",
            "type": "textbox",
            "advanced": false,
            "section": "connectivity",
            "hidden": "never"
        }),
        json!({
            "name": "apiPath",
            "value": "/api",
            "type": "textbox",
            "advanced": false,
            "section": "connectivity",
            "hidden": "never"
        }),
        json!({
            "name": "apiKey",
            "value": "",
            "type": "password",
            "advanced": false,
            "section": "connectivity",
            "hidden": "never"
        }),
        json!({
            "name": "categories",
            "value": [3030],
            "type": "tag",
            "advanced": false,
            "section": "indexer",
            "hidden": "never"
        }),
    ];

    if protocol == "torrent" {
        fields.extend([
            json!({
                "name": "minimumSeeders",
                "value": 0,
                "type": "number",
                "advanced": false,
                "section": "torrent",
                "hidden": "never"
            }),
            json!({
                "name": "seedCriteria.seedRatio",
                "value": 1.0,
                "type": "number",
                "advanced": false,
                "section": "torrent",
                "hidden": "never"
            }),
            json!({
                "name": "seedCriteria.seedTime",
                "value": 60,
                "type": "number",
                "advanced": false,
                "section": "torrent",
                "hidden": "never"
            }),
            json!({
                "name": "seedCriteria.discographySeedTime",
                "value": 60,
                "type": "number",
                "advanced": true,
                "section": "torrent",
                "hidden": "never"
            }),
            json!({
                "name": "rejectBlocklistedTorrentHashesWhileGrabbing",
                "value": false,
                "type": "checkbox",
                "advanced": true,
                "section": "torrent",
                "hidden": "never"
            }),
        ]);
    }

    fields
}
