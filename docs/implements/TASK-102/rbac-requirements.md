# TASK-102: 役割ベースアクセス制御（RBAC）実装 - TDD要件定義書

**作成日**: 2025-08-23
**対象機能**: 役割ベースアクセス制御（RBAC）システム
**依存タスク**: TASK-101（セッションベース認証）

---

## 1. 機能の概要

### コア機能
- 🟢 **階層的権限制御**: 3つの役割（admin, trainer, instructor）に基づく権限管理システム
- 🟢 **セキュリティ強化**: 未認可ユーザーによる機能への不正アクセス防止
- 🟢 **システム統合**: TASK-101のセッション認証の上位層として動作

### 想定ユーザーと役割
- **管理者（admin）**: 全機能へのアクセス権限
- **研修担当者（trainer）**: 教材管理、研修管理、受講者管理が可能
- **講師（instructor）**: 読み取り専用、限定的な機能のみ

### システム位置づけ
- Loco.rs middlewareとして実装
- 全てのAPIエンドポイントとUI要素への認可制御
- セッションベース認証（TASK-101）との完全統合

## 2. 入力・出力の仕様

### 入力パラメータ
```rust
// 🟢 既存のセッション情報から取得
pub struct AuthContext {
    pub user_id: i32,
    pub user_role: UserRole,
    pub session_id: String,
}

// 🟢 ユーザー役割の定義
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,      // 管理者: 全機能アクセス
    Trainer,    // 研修担当者: 教材・研修管理
    Instructor, // 講師: 限定的な読み取り専用
}

// 🟡 リソース識別子
pub struct ResourceRequest {
    pub endpoint: String,
    pub method: HttpMethod,
    pub resource_id: Option<i32>,
}
```

### 出力値
```rust
// 🟢 認可結果
pub struct AuthorizationResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_role: Option<UserRole>,
}

// 🟢 エラーレスポンス
pub struct AuthorizationError {
    pub status: u16,        // 403 Forbidden
    pub message: String,
    pub required_role: Option<String>,
}
```

### 権限マトリックス
```rust
// 🟢 タスク定義書から直接抽出
pub fn get_role_permissions() -> HashMap<UserRole, Vec<Permission>> {
    HashMap::from([
        (UserRole::Admin, vec![
            Permission::UserManagement,
            Permission::MaterialManagement,
            Permission::TrainingManagement,
            Permission::ProjectManagement,
            Permission::SystemConfig,
        ]),
        (UserRole::Trainer, vec![
            Permission::MaterialManagement,
            Permission::TrainingManagement,
            Permission::StudentManagement,
            Permission::ProjectView,
        ]),
        (UserRole::Instructor, vec![
            Permission::MaterialView,
            Permission::TrainingView,
            Permission::ProfileManagement,
        ]),
    ])
}
```

## 3. 制約条件

### セキュリティ要件
- 🟢 **デフォルト拒否原則**: 明示的に許可されていない限り全てアクセス拒否
- 🟢 **階層的権限**: admin > trainer > instructor の厳格な権限階層
- 🟡 **権限昇格防止**: 下位役割による上位役割への不正昇格防止
- 🟡 **セッション連携**: TASK-101のセッション管理との完全統合

### パフォーマンス要件
- 🟡 **応答時間**: 権限チェック処理は10ms以内で完了
- 🟡 **キャッシュ戦略**: セッション期間中の権限情報キャッシュ
- 🟡 **スケーラビリティ**: 100ユーザー同時アクセス時も性能劣化なし

### アーキテクチャ制約
- 🟢 **Loco.rs middleware**: 既存認証システムの拡張として実装
- 🟢 **PostgreSQL統合**: usersテーブルのroleカラムを活用
- 🟡 **API一貫性**: 全APIエンドポイントで統一的な権限チェック実装

## 4. 想定される使用例

### 正常系シナリオ

#### シナリオ1: 管理者による全機能アクセス
```rust
// 🟢 管理者がユーザー管理機能にアクセス
let context = AuthContext {
    user_role: UserRole::Admin,
    // ...
};
let result = check_permission(&context, "/api/users", HttpMethod::GET);
assert_eq!(result.allowed, true);
```

#### シナリオ2: 研修担当者による教材管理
```rust
// 🟢 研修担当者が教材管理機能にアクセス
let context = AuthContext {
    user_role: UserRole::Trainer,
    // ...
};
let result = check_permission(&context, "/api/materials", HttpMethod::POST);
assert_eq!(result.allowed, true);
```

#### シナリオ3: 講師による読み取り専用アクセス
```rust
// 🟢 講師が教材閲覧機能にアクセス
let context = AuthContext {
    user_role: UserRole::Instructor,
    // ...
};
let result = check_permission(&context, "/api/materials", HttpMethod::GET);
assert_eq!(result.allowed, true);
```

### エッジケース

#### エッジケース1: 役割変更時のセッション無効化
- 🟡 **状況**: ユーザーの役割がadminからinstructorに変更された場合
- 🟡 **期待動作**: 既存セッションの即座の無効化と権限の再評価

#### エッジケース2: 無効な役割データ
- 🟡 **状況**: データベースに不正な役割値が保存されている場合
- 🟡 **期待動作**: デフォルトで最小権限（instructor相当）を適用

### エラーケース

#### エラーケース1: 権限不足アクセス
```rust
// 🟢 講師がユーザー管理機能にアクセス（権限不足）
let context = AuthContext {
    user_role: UserRole::Instructor,
    // ...
};
let result = check_permission(&context, "/api/users", HttpMethod::POST);
assert_eq!(result.allowed, false);
assert_eq!(result.required_role, Some(UserRole::Admin));
```

#### エラーケース2: 未認証セッション
- 🟢 **状況**: セッションが存在しないか期限切れの状態でのアクセス
- 🟢 **期待動作**: HTTP 401 Unauthorized + ログイン画面リダイレクト

#### エラーケース3: データベース接続エラー
- 🟡 **状況**: 権限確認中のデータベース接続失敗
- 🟡 **期待動作**: セキュアフェイル（アクセス拒否）+ エラーログ記録

## 5. データフロー

```
HTTP Request
    ↓
Session Middleware (TASK-101)
    ↓ session validation
Session Information
    ↓
RBAC Middleware (TASK-102)
    ↓ permission check
Authorization Decision
    ↓
[Allowed] → Route Handler → Response
[Denied]  → 403 Forbidden Response
```

### 権限チェックプロセス
1. **セッション検証**: TASK-101のセッションミドルウェアで認証確認
2. **ユーザー情報取得**: セッションからuser_idを取得してユーザー情報をDB取得
3. **役割確認**: ユーザーのroleフィールドから権限レベル判定
4. **リソース権限チェック**: 要求されたリソースへのアクセス権限確認
5. **認可決定**: 許可/拒否の決定とレスポンス生成

## 6. 実装要件

### 必須実装項目
- 🟢 **RBACミドルウェア**: Loco.rs middleware pattern準拠
- 🟢 **権限チェック関数**: リソースと役割の組み合わせ検証
- 🟢 **エラーハンドリング**: HTTP 403 + 適切なエラーメッセージ
- 🟢 **UI権限制御**: 役割に応じたメニュー表示/非表示

### テスト要件
- 🟢 **単体テスト**: 権限チェックロジックの網羅的テスト
- 🟢 **統合テスト**: 各役割のアクセス制御統合テスト
- 🟢 **エラーテスト**: 権限違反時のエラーレスポンステスト

## 7. 完了条件

### 機能要件
- 🟢 **管理者専用機能**: adminのみがユーザー管理機能にアクセス可能
- 🟢 **研修担当者機能**: trainerが教材管理機能を利用可能
- 🟢 **講師機能**: instructorが適切な読み取り専用機能にアクセス可能
- 🟢 **権限エラー**: 権限なしアクセス時の適切なエラー表示

### 非機能要件
- 🟡 **性能**: 権限チェックによる応答時間の劣化が10ms未満
- 🟡 **セキュリティ**: 権限昇格攻撃の防御
- 🟡 **運用性**: 権限関連の詳細なログ記録

## 8. 参照要件・タスク

### 参照したタスク定義
- **メインタスク**: TASK-102（役割ベースアクセス制御実装）
- **前提タスク**: TASK-101（セッション認証）、TASK-004（ORM・モデル）

### 参照した要件
- **REQ-002**: 認可システム（役割ベースアクセス制御）
- **REQ-101**: 管理者権限（ユーザー管理機能へのアクセス）
- **REQ-102**: 研修担当者権限（教材・研修管理機能へのアクセス）
- **REQ-103**: 講師権限（限定的な機能アクセス）

---

**要件定義完了日**: 2025-08-23
**品質判定**: ✅ 高品質（要件明確、入出力定義完全、制約条件明確、実装可能性確実）