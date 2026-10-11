use crate::auth::rbac::{CanReadConfig, CanWriteConfig};
use crate::models::setting;
use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(list_settings).put(update_settings))
}

#[derive(Serialize)]
pub struct SettingsView {
    pub recordings_retention_days: i64,
}

async fn list_settings(
    State(state): State<AppState>,
    _perm: CanReadConfig,
) -> Result<Json<SettingsView>, StatusCode> {
    let days = setting::get_int(
        &state.pool,
        setting::KEY_RECORDINGS_RETENTION_DAYS,
        0,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SettingsView {
        recordings_retention_days: days,
    }))
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
    pub recordings_retention_days: Option<i64>,
}

async fn update_settings(
    State(state): State<AppState>,
    _perm: CanWriteConfig,
    Json(input): Json<SettingsUpdate>,
) -> Result<Json<SettingsView>, StatusCode> {
    if let Some(days) = input.recordings_retention_days {
        if days < 0 || days > 3650 {
            return Err(StatusCode::BAD_REQUEST);
        }
        setting::set(
            &state.pool,
            setting::KEY_RECORDINGS_RETENTION_DAYS,
            &days.to_string(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let days = setting::get_int(
        &state.pool,
        setting::KEY_RECORDINGS_RETENTION_DAYS,
        0,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SettingsView {
        recordings_retention_days: days,
    }))
}
