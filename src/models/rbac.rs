use loco_rs::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

// 【RBAC コア構造体・enum定義】: 役割ベースアクセス制御のための基本型 🟢
// 【設計方針】: 要件定義書の3役割階層システムを厳密に実装 🟢

// 【設定定数】: RBAC システムの設定値 🟢
// 【保守性向上】: 設定値の一元管理による変更時の影響範囲明確化 🟢

/// デフォルトのセキュアフェイル時の必要権限レベル
/// 【セキュリティ設計】: 不明なリソースへのアクセス時の安全な権限要求
/// 【調整可能性】: セキュリティポリシーに応じて変更可能
const DEFAULT_SECURE_FAIL_ROLE: UserRole = UserRole::Admin;

/// 権限不足時のHTTPステータスコード  
/// 【HTTP仕様準拠】: RFC 7231に基づく403 Forbidden
/// 【統一性確保】: アプリケーション全体でのステータスコード統一
const AUTHORIZATION_DENIED_STATUS: u16 = 403;

/// ユーザー役割の定義
/// 【階層構造】: Admin > Trainer > Instructor の権限階層
/// 【セキュリティ】: 明確な権限境界による不正アクセス防止
/// 🟢 青信号: 要件定義書から直接抽出した確実な仕様
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserRole {
    Instructor, // 講師: 読み取り専用の最小権限
    Trainer,    // 研修担当者: 教材・研修管理権限
    Admin,      // 管理者: 全機能アクセス権限
}

impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "trainer" => Ok(UserRole::Trainer), 
            "instructor" => Ok(UserRole::Instructor),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Trainer => write!(f, "trainer"),
            UserRole::Instructor => write!(f, "instructor"),
        }
    }
}

/// 認証コンテキスト - セッション情報とユーザー役割
/// 【統合設計】: TASK-101のセッション認証との完全統合
/// 【セキュリティ】: セッション認証後のRBAC認可処理のための情報保持
/// 🟢 青信号: 要件定義書の統合仕様から直接実装
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i32,
    pub user_role: UserRole,
    pub session_id: String,
}

/// 認可結果の構造体
/// 【判定結果】: 許可/拒否の明確な結果と必要権限レベルの提示
/// 【エラー処理】: 明確なエラーメッセージによるユーザビリティ向上
/// 🟢 青信号: 要件定義書の認可結果仕様から実装
#[derive(Debug, Clone)]
pub struct AuthorizationResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_role: Option<UserRole>,
}

/// 認可エラーの構造体
/// 【エラー処理】: HTTP ステータスコードと詳細メッセージによる適切なレスポンス
/// 【セキュリティ】: 技術的詳細を隠した安全なエラーメッセージ
/// 🟢 青信号: セキュリティ要件から実装
#[derive(Debug, Clone)]
pub struct AuthorizationError {
    pub status: u16,
    pub message: String,
    pub required_role: Option<String>,
}

impl std::fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Authorization Error [{}]: {}", self.status, self.message)
    }
}

impl std::error::Error for AuthorizationError {}

/// 権限の種類を定義するenum
/// 【権限分類】: 機能別の詳細な権限制御
/// 【拡張性】: 将来的な機能追加に対応可能な構造
/// 🟢 青信号: 要件定義書の機能権限から抽出
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    // 管理者専用権限
    UserManagement,
    SystemConfig,
    
    // 研修担当者権限
    MaterialManagement,
    TrainingManagement,
    StudentManagement,
    ProjectView,
    
    // 講師権限
    MaterialView,
    TrainingView,
    ProfileManagement,
}

// 【権限チェック機能の実装】: リソースアクセス制御の中核機能 🟢
// 【設計方針】: デフォルト拒否原則による安全性優先設計 🟢

/// リソースへのアクセス権限をチェックする関数
/// 【中核機能】: RBAC システムの権限判定処理
/// 【セキュリティ】: デフォルト拒否原則による安全な認可処理
/// 【性能】: 効率的な権限マトリックス照合による高速判定
/// 🟢 青信号: 要件定義書の権限チェック仕様を完全実装
///
/// # Arguments
/// * `auth_context` - 認証済みユーザーのコンテキスト情報
/// * `endpoint` - アクセス対象のAPIエンドポイント
/// * `method` - HTTPメソッド（GET, POST, PUT, DELETE等）
///
/// # Returns
/// * `AuthorizationResult` - 認可判定の結果（許可/拒否と詳細情報）
pub async fn check_permission(
    auth_context: &AuthContext,
    endpoint: &str,
    method: &str,
) -> AuthorizationResult {
    // 【権限マトリックス取得】: 役割別の許可権限一覧を取得 🟢
    // 【パフォーマンス最適化】: 静的キャッシュされた権限マトリックスを使用 🟢
    let role_permissions = get_role_permissions();
    let empty_permissions = vec![];
    let user_permissions = role_permissions.get(&auth_context.user_role).unwrap_or(&empty_permissions);

    // 【エンドポイント解析】: URLパターンから必要な権限を判定 🟢
    let required_permission = determine_required_permission(endpoint, method);
    
    match required_permission {
        Some(permission) => {
            // 【権限判定】: ユーザーの権限に必要な権限が含まれているかチェック 🟢
            if user_permissions.contains(&permission) {
                AuthorizationResult {
                    allowed: true,
                    reason: None,
                    required_role: None,
                }
            } else {
                // 【拒否処理】: 必要な最小権限レベルを特定 🟢
                let required_role = determine_minimum_required_role(&permission);
                AuthorizationResult {
                    allowed: false,
                    reason: Some(format!("Insufficient permissions for {}", endpoint)),
                    required_role,
                }
            }
        }
        None => {
            // 【不明エンドポイント】: デフォルト拒否原則を適用 🟢
            // 【セキュアフェイル】: 設定された最高権限レベルを要求 🟢
            AuthorizationResult {
                allowed: false,
                reason: Some(format!("Unknown endpoint: {}", endpoint)),
                required_role: Some(DEFAULT_SECURE_FAIL_ROLE),
            }
        }
    }
}

/// 文字列から UserRole を解析する関数
/// 【文字列解析】: データベースやリクエストから受信した役割文字列を安全に解析
/// 【エラー処理】: 不正な役割データに対する堅牢なエラーハンドリング
/// 🟢 青信号: セキュリティベストプラクティスに基づく実装
///
/// # Arguments
/// * `role_str` - 解析対象の役割文字列
///
/// # Returns
/// * `Result<UserRole, AuthorizationError>` - 解析された役割またはエラー
pub fn parse_user_role(role_str: &str) -> Result<UserRole, AuthorizationError> {
    // 【空文字列チェック】: 基本的な入力値検証 🟢
    if role_str.is_empty() {
        return Err(AuthorizationError {
            status: AUTHORIZATION_DENIED_STATUS,
            message: "役割が設定されていません".to_string(),
            required_role: None,
        });
    }

    // 【文字列解析】: FromStr トレイトを使用した型安全な変換 🟢
    role_str.parse::<UserRole>().map_err(|_| AuthorizationError {
        status: AUTHORIZATION_DENIED_STATUS,
        message: "無効な役割です".to_string(),
        required_role: None,
    })
}

// 【内部ヘルパー関数】: 権限判定のためのサポート機能 🟢
// 【設計方針】: 権限マトリックス管理と効率的な判定処理 🟢

/// 【パフォーマンス最適化】: 静的キャッシュされた権限マトリックスを取得
/// 【改善内容】: OnceLockを使用して権限マトリックスを一度だけ初期化し、メモリ効率を向上
/// 【設計方針】: 権限情報は不変データなのでスレッドセーフな静的キャッシュが最適
/// 【パフォーマンス】: 関数呼び出し毎のHashMap生成を削除し、レスポンス時間を短縮
/// 【保守性】: 権限定義の一元管理により設定変更時の保守性を向上
/// 🟢 青信号: パフォーマンステスト結果に基づく確実な最適化
fn get_role_permissions() -> &'static HashMap<UserRole, Vec<Permission>> {
    // 【静的初期化】: プロセス起動時に一度だけ権限マトリックスを生成 🟢
    // 【メモリ効率】: 重複するHashMap生成を防止しメモリ使用量を削減 🟢
    static ROLE_PERMISSIONS: OnceLock<HashMap<UserRole, Vec<Permission>>> = OnceLock::new();
    
    ROLE_PERMISSIONS.get_or_init(|| {
        let mut permissions = HashMap::new();

        // 【管理者権限】: 全機能への無制限アクセス 🟢
        // 【階層管理】: 全ての権限を含むことで管理者の最高権限を保証 🟢
        permissions.insert(UserRole::Admin, vec![
            Permission::UserManagement,
            Permission::SystemConfig,
            Permission::MaterialManagement,
            Permission::TrainingManagement,
            Permission::StudentManagement,
            Permission::ProjectView,
            Permission::MaterialView,
            Permission::TrainingView,
            Permission::ProfileManagement,
        ]);

        // 【研修担当者権限】: 教材・研修管理と受講者管理 🟢
        // 【機能分離】: 管理者権限を除いた教材・研修関連権限のみを付与 🟢
        permissions.insert(UserRole::Trainer, vec![
            Permission::MaterialManagement,
            Permission::TrainingManagement,
            Permission::StudentManagement,
            Permission::ProjectView,
            Permission::MaterialView,
            Permission::TrainingView,
            Permission::ProfileManagement,
        ]);

        // 【講師権限】: 読み取り専用の限定機能 🟢
        // 【最小権限原則】: 最小限の閲覧権限のみを付与しセキュリティを確保 🟢
        permissions.insert(UserRole::Instructor, vec![
            Permission::MaterialView,
            Permission::TrainingView,
            Permission::ProfileManagement,
        ]);

        permissions
    })
}

/// 【機能概要】: エンドポイントとHTTPメソッドから必要な権限を判定
/// 【改善内容】: 明確なパターンマッチングによる権限要件の高速判定機能
/// 【設計方針】: RESTful APIの標準規約に基づく権限マッピング
/// 【パフォーマンス】: O(1)のパターンマッチングによる高速権限判定
/// 【保守性】: エンドポイント追加時の明確な権限設定ガイダンス
/// 【拡張性】: 新しいAPIエンドポイント追加時の簡単な権限設定
/// 🟢 青信号: 要件定義書のAPIアクセス制御から実装
///
/// # Arguments
/// * `endpoint` - APIエンドポイントパス（例: "/api/users"）
/// * `method` - HTTPメソッド（GET, POST, PUT, DELETE等）
///
/// # Returns  
/// * `Option<Permission>` - 必要な権限、未定義の場合はNone
///
/// # Examples
/// ```rust
/// // 管理者専用のユーザー作成操作
/// assert_eq!(determine_required_permission("/api/users", "POST"), 
///           Some(Permission::UserManagement));
/// 
/// // 講師でもアクセス可能な教材閲覧
/// assert_eq!(determine_required_permission("/api/materials", "GET"), 
///           Some(Permission::MaterialView));
/// ```
fn determine_required_permission(endpoint: &str, method: &str) -> Option<Permission> {
    match (endpoint, method) {
        // === 【管理者専用エンドポイント】: Admin権限が必要な操作 === 🟢
        ("/api/users", "POST") => Some(Permission::UserManagement),
        ("/api/users", "PUT") => Some(Permission::UserManagement), 
        ("/api/users", "DELETE") => Some(Permission::UserManagement),
        ("/api/admin/settings", _) => Some(Permission::SystemConfig),
        ("/api/admin/users", _) => Some(Permission::UserManagement),
        ("/api/admin/system", _) => Some(Permission::SystemConfig),

        // === 【研修担当者権限エンドポイント】: Trainer権限が必要な操作 === 🟢
        // 【教材管理】: 教材の作成・更新・削除操作 🟢
        ("/api/materials", "POST") => Some(Permission::MaterialManagement),
        ("/api/materials", "PUT") => Some(Permission::MaterialManagement),
        ("/api/materials", "DELETE") => Some(Permission::MaterialManagement),
        
        // 【研修管理】: 研修コースの作成・更新・削除操作 🟢
        ("/api/trainings", "POST") => Some(Permission::TrainingManagement),
        ("/api/trainings", "PUT") => Some(Permission::TrainingManagement),
        ("/api/trainings", "DELETE") => Some(Permission::TrainingManagement),

        // === 【読み取り専用エンドポイント】: Instructor権限でもアクセス可能 === 🟢
        // 【教材閲覧】: 教材の参照操作（全役割対応） 🟢
        ("/api/materials", "GET") => Some(Permission::MaterialView),
        
        // 【研修閲覧】: 研修情報の参照操作（全役割対応） 🟢
        ("/api/trainings", "GET") => Some(Permission::TrainingView),
        
        // 【プロフィール管理】: 自身の情報管理（全役割対応） 🟢
        ("/api/profile", "GET") => Some(Permission::ProfileManagement),
        ("/api/profile", "PUT") => Some(Permission::ProfileManagement),

        // === 【未定義エンドポイント】: セキュアフェイルの対象 === 🟢
        // 【セキュリティ】: 未知のエンドポイントはデフォルト拒否 🟢
        _ => None,
    }
}

/// 【機能概要】: 権限に必要な最小役割レベルを逆引き判定
/// 【改善内容】: 権限不足エラー時のユーザビリティを向上させる役割レベル特定機能
/// 【設計方針】: 権限階層の逆マッピングによる明確なエラーメッセージ生成
/// 【パフォーマンス】: O(1)のパターンマッチングによる高速役割判定
/// 【保守性】: 権限追加時の役割レベル設定の明確化
/// 【ユーザビリティ】: 必要な権限レベルを明確に提示しユーザー体験を向上
/// 🟢 青信号: 要件定義書の権限階層から実装
///
/// # Arguments
/// * `permission` - 判定対象の権限種別
///
/// # Returns
/// * `Option<UserRole>` - その権限に必要な最小役割、システム権限の場合はNone
///
/// # Examples
/// ```rust
/// // 管理者専用権限
/// assert_eq!(determine_minimum_required_role(&Permission::UserManagement), 
///           Some(UserRole::Admin));
///
/// // 研修担当者権限
/// assert_eq!(determine_minimum_required_role(&Permission::MaterialManagement), 
///           Some(UserRole::Trainer));
/// ```
fn determine_minimum_required_role(permission: &Permission) -> Option<UserRole> {
    match permission {
        // === 【管理者専用権限】: Admin役割が最小要件 === 🟢
        Permission::UserManagement | Permission::SystemConfig => Some(UserRole::Admin),

        // === 【研修担当者権限】: Trainer役割が最小要件 === 🟢  
        Permission::MaterialManagement | Permission::TrainingManagement | 
        Permission::StudentManagement | Permission::ProjectView => Some(UserRole::Trainer),

        // === 【講師権限（最小権限）】: Instructor役割が最小要件 === 🟢
        Permission::MaterialView | Permission::TrainingView | 
        Permission::ProfileManagement => Some(UserRole::Instructor),
    }
}