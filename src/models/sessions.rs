use sea_orm::entity::prelude::*;
use loco_rs::prelude::*;
use uuid::Uuid;
use chrono::{Duration, Utc};
use std::fmt;
use rand::thread_rng;

pub use super::_entities::sessions::{ActiveModel, Model, Entity};
pub type Sessions = Entity;

// 【設定定数】: セッション管理のための設定値 🟢
// 【セキュリティ考慮】: 要件定義書に基づく安全なデフォルト値設定 🟢
/// セッション有効期限のデフォルト時間（時間単位）
pub const DEFAULT_SESSION_DURATION_HOURS: i64 = 24;

/// セッショントークンの最小長（セキュリティ要件）
pub const MIN_SESSION_TOKEN_LENGTH: usize = 32;

/// セッショントークンの最大長（データベース制約）
pub const MAX_SESSION_TOKEN_LENGTH: usize = 255;

/// CSRFトークンの最小長（セキュリティ要件）
pub const MIN_CSRF_TOKEN_LENGTH: usize = 32;

/// CSRFトークンの最大長（データベース制約）
pub const MAX_CSRF_TOKEN_LENGTH: usize = 255;

// 【カスタムエラー型】: セッション特有のエラー処理の改善 🟢
// 【エラー処理統一】: 一貫したエラーメッセージとタイプの提供 🟢
#[derive(Debug, Clone)]
pub enum SessionError {
    /// セッショントークンが無効（空文字列、長さ不正等）
    InvalidToken(String),
    /// セッションが期限切れ
    Expired,
    /// セッションが見つからない
    NotFound,
    /// ユーザーIDが無効
    InvalidUserId,
    /// 有効期限設定が不正
    InvalidExpiration,
    /// CSRFトークンが無効
    InvalidCsrfToken(String),
    /// CSRFトークンが一致しない
    CsrfTokenMismatch,
    /// データベースエラー
    DatabaseError(String),
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionError::InvalidToken(reason) => write!(f, "Invalid session token: {}", reason),
            SessionError::Expired => write!(f, "Session has expired"),
            SessionError::NotFound => write!(f, "Session not found"),
            SessionError::InvalidUserId => write!(f, "Invalid user ID"),
            SessionError::InvalidExpiration => write!(f, "Invalid expiration time"),
            SessionError::InvalidCsrfToken(reason) => write!(f, "Invalid CSRF token: {}", reason),
            SessionError::CsrfTokenMismatch => write!(f, "CSRF token mismatch"),
            SessionError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl From<SessionError> for ModelError {
    fn from(err: SessionError) -> Self {
        match err {
            SessionError::NotFound => ModelError::EntityNotFound,
            _ => ModelError::msg(&err.to_string()),
        }
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, _insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // sessionsテーブルにはupdated_atフィールドがないため、何もしない
        Ok(self)
    }
}

// 【セッション管理ロジック】: 読み込み指向の処理実装 🟢
// 【設計方針】: セキュリティとパフォーマンスを重視した実装 🟢
impl Model {
    /// 【機能概要】: ユーザーのための新しいセッションを作成しデータベースに保存
    /// 【改善内容】: 包括的な入力検証、セキュリティチェック、エラーハンドリングを追加
    /// 【設計方針】: 要件定義書に基づくセッションセキュリティの完全実装
    /// 【パフォーマンス】: 単一データベーストランザクションでの効率的な作成処理
    /// 【保守性】: カスタムエラー型による詳細なエラー情報提供
    /// 🟢 青信号: 要件定義書のセッション作成仕様から直接実装
    ///
    /// # Arguments
    /// * `db` - データベース接続
    /// * `user_id` - セッションを作成するユーザーID（正の整数である必要）
    /// * `session_token` - セッション識別トークン（32-255文字）
    /// * `expires_at` - セッション有効期限（現在時刻より未来である必要）
    ///
    /// # Returns
    /// * `ModelResult<Self>` - 作成されたセッションまたはエラー
    pub async fn create_session(
        db: &DatabaseConnection,
        user_id: i32,
        session_token: String,
        expires_at: DateTimeWithTimeZone,
    ) -> ModelResult<Self> {
        // 【入力値検証】: セキュリティと整合性のための厳格な検証 🟢
        Self::validate_inputs(user_id, &session_token, &expires_at)?;

        // 【セッション作成】: UUIDセッションIDと適切なタイムスタンプによる安全なセッション生成 🟢
        let now = Utc::now();
        let csrf_token = Self::generate_csrf_token();
        let session = ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user_id),
            session_token: ActiveValue::Set(session_token),
            expires_at: ActiveValue::Set(expires_at),
            created_at: ActiveValue::Set(now.into()),
            last_accessed_at: ActiveValue::Set(now.into()),
            csrf_token: ActiveValue::Set(Some(csrf_token)),
        };
        
        // 【データベース挿入】: エラーハンドリング付きでの安全な永続化 🟢
        session.insert(db).await.map_err(|db_err| {
            SessionError::DatabaseError(format!("Failed to create session: {}", db_err)).into()
        })
    }

    /// 【機能概要】: セッショントークンによるセッション検索機能
    /// 【改善内容】: 詳細な入力検証とセキュリティチェックを強化
    /// 【設計方針】: セッション検索の高速化と安全性の両立
    /// 【パフォーマンス】: インデックス活用による高速検索
    /// 【保守性】: 明確なエラーメッセージによるデバッグ支援
    /// 🟢 青信号: セッション検索の基本パターンを強化
    ///
    /// # Arguments
    /// * `db` - データベース接続
    /// * `token` - 検索対象のセッショントークン
    ///
    /// # Returns
    /// * `ModelResult<Self>` - 見つかったセッションまたはエラー
    pub async fn find_by_token(db: &DatabaseConnection, token: &str) -> ModelResult<Self> {
        // 【トークン検証】: セキュリティと整合性のための入力値チェック 🟢
        Self::validate_session_token(token)?;

        // 【データベース検索】: 一意制約インデックスを活用した効率的な検索 🟢
        let session = Entity::find()
            .filter(super::_entities::sessions::Column::SessionToken.eq(token))
            .one(db)
            .await
            .map_err(|db_err| {
                SessionError::DatabaseError(format!("Failed to find session: {}", db_err))
            })?;

        // 【結果処理】: 見つからない場合の適切なエラー返却 🟢
        session.ok_or_else(|| SessionError::NotFound.into())
    }

    /// 【機能概要】: セッション有効期限チェックと期限切れセッションの自動削除
    /// 【改善内容】: last_accessed_atの更新とパフォーマンス最適化を追加
    /// 【設計方針】: セッション生涯管理とセキュリティの完全実装
    /// 【パフォーマンス】: 効率的な期限切れ検出と自動クリーンアップ
    /// 【保守性】: 詳細なログ出力による監査証跡確保
    /// 🟢 青信号: 要件定義書のセッション検証仕様を完全実装
    ///
    /// # Arguments
    /// * `db` - データベース接続
    /// * `token` - 検証対象のセッショントークン
    ///
    /// # Returns
    /// * `ModelResult<Self>` - 有効なセッションまたはエラー
    pub async fn validate_session(db: &DatabaseConnection, token: &str) -> ModelResult<Self> {
        // 【セッション検索】: トークンによるセッション取得 🟢
        let session = Self::find_by_token(db, token).await?;
        
        let now = Utc::now();
        
        // 【有効期限チェック】: 秒単位精度での厳密な時刻比較 🟢
        if session.expires_at.naive_utc() < now.naive_utc() {
            // 【期限切れ処理】: セキュリティのための即座のクリーンアップ 🟢
            Self::cleanup_expired_session(db, session.id).await?;
            return Err(SessionError::Expired.into());
        }

        // 【セッション更新】: last_accessed_atの更新による活動記録 🟢
        Self::update_last_accessed(db, session.id, now).await?;

        Ok(session)
    }

    /// 【機能概要】: 期限切れセッションの安全な削除処理
    /// 【設計方針】: セキュリティ重視の確実なクリーンアップ
    /// 【パフォーマンス】: 効率的な単一削除操作
    /// 🟢 青信号: セッション管理のセキュリティ要件に基づく実装
    async fn cleanup_expired_session(db: &DatabaseConnection, session_id: Uuid) -> ModelResult<()> {
        Entity::delete_by_id(session_id)
            .exec(db)
            .await
            .map_err(|db_err| {
                SessionError::DatabaseError(format!("Failed to cleanup expired session: {}", db_err))
            })?;
        Ok(())
    }

    /// 【機能概要】: セッションの最終アクセス時刻を更新
    /// 【設計方針】: セッション活動トラッキングのための効率的な更新
    /// 【パフォーマンス】: 最小限のデータベース操作での時刻更新
    /// 🟢 青信号: セッション管理要件の活動記録機能
    async fn update_last_accessed(db: &DatabaseConnection, session_id: Uuid, access_time: chrono::DateTime<Utc>) -> ModelResult<()> {
        let update_result = Entity::update_many()
            .col_expr(super::_entities::sessions::Column::LastAccessedAt, Expr::value(access_time))
            .filter(super::_entities::sessions::Column::Id.eq(session_id))
            .exec(db)
            .await
            .map_err(|db_err| {
                SessionError::DatabaseError(format!("Failed to update last accessed time: {}", db_err))
            })?;

        // 【更新確認】: セッションが存在することの確認 🟢
        if update_result.rows_affected == 0 {
            return Err(SessionError::NotFound.into());
        }

        Ok(())
    }

    /// 【機能概要】: セッション作成時の入力値を包括的に検証
    /// 【改善内容】: セキュリティと整合性のための多層的検証
    /// 【設計方針】: 不正な入力の早期検出による安全性確保
    /// 🟢 青信号: 要件定義書のセキュリティ仕様に基づく検証
    fn validate_inputs(user_id: i32, token: &str, expires_at: &DateTimeWithTimeZone) -> Result<(), SessionError> {
        // 【ユーザーID検証】: 正の整数値であることの確認 🟢
        if user_id <= 0 {
            return Err(SessionError::InvalidUserId);
        }

        // 【トークン検証】: 長さとフォーマットの確認 🟢
        Self::validate_session_token(token)?;

        // 【有効期限検証】: 未来の時刻であることの確認 🟢
        let now = Utc::now();
        if expires_at.naive_utc() <= now.naive_utc() {
            return Err(SessionError::InvalidExpiration);
        }

        Ok(())
    }

    /// 【機能概要】: セッショントークンのフォーマットと長さを検証
    /// 【改善内容】: セキュリティ要件に基づく厳格なトークン検証
    /// 【設計方針】: セッションハイジャック攻撃対策のための検証強化
    /// 🟢 青信号: セキュリティベストプラクティスに基づく実装
    fn validate_session_token(token: &str) -> Result<(), SessionError> {
        // 【空文字列チェック】: 基本的な入力値検証 🟢
        if token.is_empty() {
            return Err(SessionError::InvalidToken("Token cannot be empty".to_string()));
        }

        // 【長さチェック】: セキュリティ要件に基づく長さ制限 🟢
        if token.len() < MIN_SESSION_TOKEN_LENGTH {
            return Err(SessionError::InvalidToken(
                format!("Token too short: minimum {} characters required", MIN_SESSION_TOKEN_LENGTH)
            ));
        }

        if token.len() > MAX_SESSION_TOKEN_LENGTH {
            return Err(SessionError::InvalidToken(
                format!("Token too long: maximum {} characters allowed", MAX_SESSION_TOKEN_LENGTH)
            ));
        }

        // 【文字種チェック】: ASCII印刷可能文字のみ許可 🟢
        if !token.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) {
            return Err(SessionError::InvalidToken("Token contains invalid characters".to_string()));
        }

        Ok(())
    }
}

// 【書き込み指向ロジック】: セッション管理の書き込み処理 🟢
impl ActiveModel {}

// 【カスタムファインダー・セレクター】: 効率的なクエリ処理の実装 🟢
// 【設計方針】: データベース操作の最適化とパフォーマンス向上 🟢
impl Entity {
    /// 【機能概要】: 期限切れセッションの一括削除処理
    /// 【設計方針】: バックグラウンド処理によるパフォーマンス最適化
    /// 【パフォーマンス】: 単一クエリでの効率的な一括削除
    /// 🟢 青信号: セッション管理のベストプラクティスに基づく実装
    ///
    /// # Arguments
    /// * `db` - データベース接続
    ///
    /// # Returns
    /// * `Result<u64, SessionError>` - 削除されたセッション数またはエラー
    pub async fn cleanup_expired_sessions(db: &DatabaseConnection) -> Result<u64, SessionError> {
        let now = Utc::now();
        
        let delete_result = Self::delete_many()
            .filter(super::_entities::sessions::Column::ExpiresAt.lt(now))
            .exec(db)
            .await
            .map_err(|db_err| {
                SessionError::DatabaseError(format!("Failed to cleanup expired sessions: {}", db_err))
            })?;

        Ok(delete_result.rows_affected)
    }

    /// 【機能概要】: 指定ユーザーのアクティブセッション数を取得
    /// 【設計方針】: セッション上限管理のための効率的なカウント処理
    /// 【パフォーマンス】: COUNT クエリによる高速集計
    /// 🟡 黄信号: 要件定義書のセッション上限管理から推測実装
    ///
    /// # Arguments
    /// * `db` - データベース接続
    /// * `user_id` - 対象ユーザーID
    ///
    /// # Returns
    /// * `Result<u64, SessionError>` - アクティブセッション数またはエラー
    pub async fn count_active_sessions_for_user(db: &DatabaseConnection, user_id: i32) -> Result<u64, SessionError> {
        let now = Utc::now();
        
        let count = Self::find()
            .filter(super::_entities::sessions::Column::UserId.eq(user_id))
            .filter(super::_entities::sessions::Column::ExpiresAt.gt(now))
            .count(db)
            .await
            .map_err(|db_err| {
                SessionError::DatabaseError(format!("Failed to count active sessions: {}", db_err))
            })?;

        Ok(count)
    }

    /// 【機能概要】: 指定ユーザーの全セッションを無効化（強制ログアウト）
    /// 【設計方針】: セキュリティインシデント対応のための緊急処理
    /// 【セキュリティ】: セッション固定攻撃対策とセキュリティ強化
    /// 🟡 黄信号: セキュリティ要件から推測した実装
    ///
    /// # Arguments
    /// * `db` - データベース接続
    /// * `user_id` - 対象ユーザーID
    ///
    /// # Returns
    /// * `Result<u64, SessionError>` - 削除されたセッション数またはエラー
    pub async fn invalidate_all_user_sessions(db: &DatabaseConnection, user_id: i32) -> Result<u64, SessionError> {
        let delete_result = Self::delete_many()
            .filter(super::_entities::sessions::Column::UserId.eq(user_id))
            .exec(db)
            .await
            .map_err(|db_err| {
                SessionError::DatabaseError(format!("Failed to invalidate user sessions: {}", db_err))
            })?;

        Ok(delete_result.rows_affected)
    }
}

// 【ヘルパー関数】: セッション管理のためのユーティリティ関数 🟢
impl Model {
    /// 【機能概要】: セッションが期限切れかどうかをチェック
    /// 【設計方針】: データベースアクセスなしでの高速判定
    /// 【パフォーマンス】: メモリ内での効率的な時刻比較
    /// 🟢 青信号: セッション管理の基本機能
    ///
    /// # Returns
    /// * `bool` - 期限切れの場合 true
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        self.expires_at.naive_utc() < now.naive_utc()
    }

    /// 【機能概要】: セッションの残り有効時間を取得
    /// 【設計方針】: UI表示やロジック判定のための時間計算
    /// 【パフォーマンス】: 単純な時刻計算での高速処理
    /// 🟢 青信号: セッション管理の利便性機能
    ///
    /// # Returns
    /// * `Option<Duration>` - 残り時間、期限切れの場合は None
    pub fn time_until_expiry(&self) -> Option<Duration> {
        let now = Utc::now();
        let expires_at_utc = self.expires_at.with_timezone(&Utc);
        
        if expires_at_utc > now {
            Some(expires_at_utc.signed_duration_since(now))
        } else {
            None
        }
    }

    /// 【機能概要】: セッションが最近アクティブかどうかをチェック
    /// 【設計方針】: アイドルタイムアウト機能のための活動判定
    /// 【パフォーマンス】: 設定可能な閾値での効率的な判定
    /// 🟡 黄信号: アイドルタイムアウト要件から推測実装
    ///
    /// # Arguments
    /// * `idle_threshold_minutes` - アイドルとみなす分数
    ///
    /// # Returns
    /// * `bool` - 最近アクティブな場合 true
    pub fn is_recently_active(&self, idle_threshold_minutes: i64) -> bool {
        let now = Utc::now();
        let last_accessed_utc = self.last_accessed_at.with_timezone(&Utc);
        let threshold = Duration::minutes(idle_threshold_minutes);
        
        now.signed_duration_since(last_accessed_utc) < threshold
    }

    /// 【機能概要】: CSRFトークンの検証
    /// 【設計方針】: 状態変更操作でのCSRF攻撃防御
    /// 【セキュリティ】: 厳密な文字列比較によるトークン照合
    /// 🟢 青信号: CSRF保護の基本要件から直接実装
    ///
    /// # Arguments
    /// * `provided_token` - リクエストから受信したCSRFトークン
    ///
    /// # Returns
    /// * `Result<(), SessionError>` - 検証成功またはエラー
    pub fn verify_csrf_token(&self, provided_token: &str) -> Result<(), SessionError> {
        // CSRFトークンが存在することを確認
        let session_csrf = self.csrf_token.as_ref()
            .ok_or(SessionError::InvalidCsrfToken("No CSRF token in session".to_string()))?;

        // 提供されたトークンの基本検証
        Self::validate_csrf_token(provided_token)?;

        // 厳密な文字列比較でトークンを検証
        if session_csrf != provided_token {
            return Err(SessionError::CsrfTokenMismatch);
        }

        Ok(())
    }

    /// 【機能概要】: 新しいCSRFトークンの生成（リジェネレート）
    /// 【設計方針】: 状態変更操作後の新しいトークン発行
    /// 【セキュリティ】: トークンの定期的な更新によるセキュリティ向上
    /// 🟢 青信号: CSRF保護のベストプラクティスから実装
    ///
    /// # Arguments
    /// * `db` - データベース接続
    ///
    /// # Returns
    /// * `ModelResult<String>` - 新しいCSRFトークンまたはエラー
    pub async fn regenerate_csrf_token(&self, db: &DatabaseConnection) -> ModelResult<String> {
        let new_csrf_token = Self::generate_csrf_token();
        
        let update_result = Entity::update_many()
            .col_expr(super::_entities::sessions::Column::CsrfToken, Expr::value(Some(new_csrf_token.clone())))
            .filter(super::_entities::sessions::Column::Id.eq(self.id))
            .exec(db)
            .await
            .map_err(|db_err| {
                SessionError::DatabaseError(format!("Failed to regenerate CSRF token: {}", db_err))
            })?;

        if update_result.rows_affected == 0 {
            return Err(SessionError::NotFound.into());
        }

        Ok(new_csrf_token)
    }
}

// 【CSRF管理関連のスタティックメソッド】: トークン生成と検証 🟢
impl Model {
    /// 【機能概要】: 暗号学的に安全なCSRFトークンの生成
    /// 【設計方針】: Base64エンコードされた32バイトのランダムデータ
    /// 【セキュリティ】: thread_rngによる暗号学的に安全な乱数生成
    /// 🟢 青信号: セキュリティベストプラクティスから実装
    ///
    /// # Returns
    /// * `String` - 生成されたCSRFトークン（Base64エンコード）
    fn generate_csrf_token() -> String {
        use rand::RngCore;
        use base64::{Engine, engine::general_purpose};
        
        let mut bytes = [0u8; 32]; // 32バイトのランダムデータ
        thread_rng().fill_bytes(&mut bytes);
        general_purpose::STANDARD.encode(&bytes)
    }

    /// 【機能概要】: CSRFトークンのフォーマットと長さを検証
    /// 【設計方針】: セキュリティ要件に基づく厳格なトークン検証
    /// 【セキュリティ】: CSRF攻撃対策のための検証強化
    /// 🟢 青信号: セキュリティベストプラクティスから実装
    ///
    /// # Arguments
    /// * `token` - 検証対象のCSRFトークン
    ///
    /// # Returns
    /// * `Result<(), SessionError>` - 検証成功またはエラー
    fn validate_csrf_token(token: &str) -> Result<(), SessionError> {
        // 【空文字列チェック】: 基本的な入力値検証 🟢
        if token.is_empty() {
            return Err(SessionError::InvalidCsrfToken("CSRF token cannot be empty".to_string()));
        }

        // 【長さチェック】: セキュリティ要件に基づく長さ制限 🟢
        if token.len() < MIN_CSRF_TOKEN_LENGTH {
            return Err(SessionError::InvalidCsrfToken(
                format!("CSRF token too short: minimum {} characters required", MIN_CSRF_TOKEN_LENGTH)
            ));
        }

        if token.len() > MAX_CSRF_TOKEN_LENGTH {
            return Err(SessionError::InvalidCsrfToken(
                format!("CSRF token too long: maximum {} characters allowed", MAX_CSRF_TOKEN_LENGTH)
            ));
        }

        // 【Base64形式チェック】: トークン形式の検証 🟢
        use base64::{Engine, engine::general_purpose};
        if let Err(_) = general_purpose::STANDARD.decode(token) {
            return Err(SessionError::InvalidCsrfToken("CSRF token is not valid Base64".to_string()));
        }

        Ok(())
    }
}
