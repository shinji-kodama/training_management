use axum::debug_handler;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardResponse {
    pub message: String,
    pub user_count: i64,
    pub project_count: i64,
}

/// Dashboard home page for the training management system
#[debug_handler]
async fn index(State(_ctx): State<AppContext>) -> Result<Response> {
    // For now, return a simple dashboard response
    // Later this will be replaced with proper view rendering
    let response = DashboardResponse {
        message: "研修管理システムダッシュボード".to_string(),
        user_count: 0, // TODO: Get from database
        project_count: 0, // TODO: Get from database
    };
    
    format::json(response)
}

/// Health check endpoint
#[debug_handler]
async fn health() -> Result<Response> {
    format::json(serde_json::json!({
        "status": "ok",
        "service": "training_management",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(index))
        .add("/health", get(health))
}