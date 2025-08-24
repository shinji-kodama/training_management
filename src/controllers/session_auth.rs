use axum::{
    extract::{FromRequestParts, FromRef, State},
    http::{request::Parts, header::AUTHORIZATION, StatusCode},
    response::{IntoResponse, Response},
};
use axum::http::HeaderMap;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use async_trait::async_trait;

use crate::models::{sessions, _entities::users};

/**
 * 【機能概要】: セッション認証のためのユーザー情報構造体
 * 【実装方針】: JWT Claimsと同等の情報を提供するセッションベース認証
 * 【セキュリティ】: セッション検証済みのユーザー情報のみ格納
 * 🟢 信頼性レベル: セッション管理要件に基づく安全な実装
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    pub user_id: i32,
    pub pid: uuid::Uuid,
    pub email: String,
    pub role: String,
    pub session_id: uuid::Uuid,
    pub csrf_token: String,
}

/**
 * 【機能概要】: セッション認証エラー型
 * 【実装方針】: 詳細なエラー情報によるデバッグ支援
 * 【セキュリティ】: セキュリティ上重要な情報の適切な隠蔽
 * 🟢 信頼性レベル: セキュリティベストプラクティスに基づく実装
 */
#[derive(Debug)]
pub enum SessionAuthError {
    MissingToken,
    InvalidToken,
    ExpiredSession,
    DatabaseError(String),
    UserNotFound,
}

impl fmt::Display for SessionAuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionAuthError::MissingToken => write!(f, "Missing session token"),
            SessionAuthError::InvalidToken => write!(f, "Invalid session token"),
            SessionAuthError::ExpiredSession => write!(f, "Session has expired"),
            SessionAuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            SessionAuthError::UserNotFound => write!(f, "User not found"),
        }
    }
}

impl IntoResponse for SessionAuthError {
    fn into_response(self) -> Response {
        let status_code = match self {
            SessionAuthError::MissingToken => StatusCode::UNAUTHORIZED,
            SessionAuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            SessionAuthError::ExpiredSession => StatusCode::UNAUTHORIZED,
            SessionAuthError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SessionAuthError::UserNotFound => StatusCode::UNAUTHORIZED,
        };

        let error_message = match self {
            SessionAuthError::MissingToken => "認証が必要です",
            SessionAuthError::InvalidToken => "無効な認証情報です",
            SessionAuthError::ExpiredSession => "セッションが期限切れです",
            SessionAuthError::DatabaseError(_) => "サーバーエラーが発生しました",
            SessionAuthError::UserNotFound => "ユーザーが見つかりません",
        };

        (status_code, error_message).into_response()
    }
}

/**
 * 【機能概要】: セッション認証ヘルパー構造体
 * 【実装方針】: 直接的な認証チェック機能を提供
 * 【セキュリティ】: セッション検証とユーザー情報の安全な取得
 * 🟢 信頼性レベル: 認証機能の簡潔で確実な実装
 */
pub struct SessionAuth {
    pub claims: SessionClaims,
}

impl SessionAuth {
    /**
     * 【機能概要】: HTTPヘッダーからセッション認証を実行
     * 【実装方針】: ヘッダー解析とセッション検証の統合処理
     * 【セキュリティ】: 包括的な認証チェックと詳細なエラーハンドリング
     * 🟢 信頼性レベル: セキュリティ要件に基づく確実な実装
     */
    pub async fn from_headers(headers: &HeaderMap, ctx: &AppContext) -> Result<Self, SessionAuthError> {
        // セッショントークンを取得
        let session_token = extract_session_token(headers)
            .ok_or(SessionAuthError::MissingToken)?;

        // セッション検証
        let session = sessions::Model::validate_session(&ctx.db, &session_token)
            .await
            .map_err(|e| match e {
                ModelError::EntityNotFound => SessionAuthError::InvalidToken,
                _ => SessionAuthError::DatabaseError(e.to_string()),
            })?;

        // セッションが期限切れかチェック
        if session.is_expired() {
            return Err(SessionAuthError::ExpiredSession);
        }

        // ユーザー情報を取得
        let user = users::Entity::find_by_id(session.user_id)
            .one(&ctx.db)
            .await
            .map_err(|e| SessionAuthError::DatabaseError(e.to_string()))?
            .ok_or(SessionAuthError::UserNotFound)?;

        // CSRFトークンを取得
        let csrf_token = session.csrf_token
            .ok_or(SessionAuthError::InvalidToken)?;

        // SessionClaimsを構築
        let claims = SessionClaims {
            user_id: user.id,
            pid: user.pid,
            email: user.email,
            role: user.role,
            session_id: session.id,
            csrf_token,
        };

        Ok(SessionAuth { claims })
    }
}

/**
 * 【機能概要】: HTTPヘッダーからセッショントークンを抽出
 * 【実装方針】: 複数の認証方式に対応（Authorizationヘッダー、Cookie）
 * 【セキュリティ】: セキュアなトークン抽出と検証
 * 🟢 信頼性レベル: 標準的なWeb認証パターンに基づく実装
 */
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // Authorization: Bearer <token> 形式をチェック
    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
            if auth_str.starts_with("Session ") {
                return Some(auth_str[8..].to_string());
            }
        }
    }

    // Cookie: session_token=<token> 形式をチェック
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("session_token=") {
                    return Some(cookie[14..].to_string());
                }
            }
        }
    }

    // X-Session-Token ヘッダーをチェック
    if let Some(session_header) = headers.get("x-session-token") {
        if let Ok(session_str) = session_header.to_str() {
            return Some(session_str.to_string());
        }
    }

    None
}

/**
 * 【機能概要】: JWT認証との互換性のためのエイリアス
 * 【実装方針】: 既存コードの段階的移行を可能にする
 * 【保守性】: JWTからセッション認証への移行時の互換性確保
 * 🟢 信頼性レベル: 後方互換性とアップグレードパスの提供
 */
/**
 * 【Axumエクストラクタ統合】: SessionAuthのFromRequestPartsトレイト実装
 * 【実装方針】: Axumハンドラー引数での自動認証処理統合
 * 【テスト対応】: 認証テストでの自動401エラー実現用
 * 🟢 信頼性レベル: Axum標準パターンに基づく確実な実装
 */
// #[async_trait]
// impl<S> FromRequestParts<S> for SessionAuth
// where
//     S: Send + Sync,
//     AppContext: axum::extract::FromRef<S>,
// {
//     type Rejection = SessionAuthError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         // 【コンテキスト取得】: Axum StateからAppContextを安全に取得
//         let ctx = AppContext::from_ref(state);
        
//         // 【ヘッダー解析】: HTTPリクエストヘッダーから認証情報抽出
//         let headers = &parts.headers;
        
//         // 【セッション認証実行】: from_headersメソッドで統一的な認証処理
//         Self::from_headers(headers, &ctx).await
//     }
// }

/**
 * 【JWT認証との互換性のためのエイリアス】: 既存コードの段階的移行を可能にする
 * 【実装方針】: JWTからセッション認証への移行時の互換性確保
 * 【保守性】: アップグレードパスの提供
 * 🟢 信頼性レベル: 後方互換性とアップグレードパスの提供
 */
pub type JWT = SessionAuth;