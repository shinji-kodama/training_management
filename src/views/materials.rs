use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::_entities::materials as materials_entity;

/**
 * 【機能概要】: 教材一覧表示のビューレスポンス構造体
 * 【実装方針】: HTML テンプレートレンダリングのためのデータ構造
 * 【セキュリティ】: XSS防止のためテンプレートエンジンの自動エスケープを活用
 * 🟢 信頼性レベル: Loco.rs標準のビューパターンに基づく実装
 */
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialListView {
    pub materials: Vec<materials_entity::Model>,
    pub total_count: usize,
    pub current_user_role: String,
    pub can_create: bool,
}

/**
 * 【機能概要】: 教材作成フォーム表示のビューレスポンス構造体
 * 【実装方針】: CSRF トークンを含むフォーム表示データ
 * 【セキュリティ】: CSRF 保護とフォーム検証のための構造体
 * 🟢 信頼性レベル: セキュリティベストプラクティスに基づく実装
 */
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialNewView {
    pub csrf_token: String,
    pub form_action: String,
    pub form_method: String,
    pub current_user_role: String,
}

/**
 * 【機能概要】: 教材詳細表示のビューレスポンス構造体
 * 【実装方針】: 教材詳細情報とユーザー権限情報を含む表示データ
 * 【セキュリティ】: 権限ベースの表示制御のための構造体
 * 🟢 信頼性レベル: RBAC統合による安全な詳細表示実装
 */
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialShowView {
    pub material: materials_entity::Model,
    pub current_user_role: String,
    pub can_edit: bool,
    pub can_delete: bool,
}

/**
 * 【機能概要】: 教材作成成功後のリダイレクトレスポンス構造体
 * 【実装方針】: 作成完了メッセージとリダイレクト先を含む応答
 * 🟢 信頼性レベル: RESTful API設計パターンに基づく実装
 */
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialCreateResponse {
    pub success: bool,
    pub material_id: uuid::Uuid,
    pub message: String,
    pub redirect_url: String,
}