# TASK-201: ユーザー管理機能実装 - TDD要件定義書

**作成日**: 2025-08-23
**対象機能**: ユーザー管理機能（CRUD + パスワード変更 + 役割管理）
**依存タスク**: TASK-102（RBAC実装）

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

### コア機能
- 🟢 **ユーザーCRUD操作**: 管理者がシステム利用者の作成・編集・削除・一覧表示を行う機能
- 🟢 **役割管理**: 3つの役割（admin, trainer, instructor）の設定・変更機能
- 🟢 **パスワード管理**: パスワード変更機能とパスワード複雑性チェック
- 🟡 **メール重複チェック**: メールアドレス重複防止機能（既存実装拡張）

### 想定されるユーザー（As a から抽出）
- **システム管理者（admin）**: 全ユーザーの管理権限
- **研修担当者（trainer）**: 限定的なユーザー情報閲覧権限
- **講師（instructor）**: 自己のプロフィール管理のみ

### システム内での位置づけ（アーキテクチャ設計から抽出）
- 🟢 **RBAC統合**: TASK-102で実装した役割ベースアクセス制御との完全統合
- 🟢 **認証システム統合**: TASK-101のセッション認証システムとの連携
- 🟢 **モデル基盤**: TASK-004で実装したusersモデルの拡張

### **参照したEARS要件**: REQ-101（管理者権限）, REQ-102（研修担当者権限）
### **参照した設計文書**: アーキテクチャ設計の認証・認可レイヤー

---

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

### 入力パラメータ（型、範囲、制約）
```rust
// 🟢 ユーザー作成・編集用の入力データ（既存RegisterParams拡張）
#[derive(Debug, Validate, Deserialize)]
pub struct UserParams {
    #[validate(length(min = 2, max = 100, message = "名前は2文字以上100文字以下で入力してください"))]
    pub name: String,
    #[validate(email(message = "有効なメールアドレスを入力してください"))]
    pub email: String,
    #[validate(length(min = 8, message = "パスワードは8文字以上で入力してください"))]
    pub password: Option<String>, // 編集時はOption（変更しない場合はNone）
    pub role: UserRole, // admin, trainer, instructor
}

// 🟢 パスワード変更専用の入力データ
#[derive(Debug, Validate, Deserialize)]
pub struct PasswordChangeParams {
    pub current_password: String,
    #[validate(length(min = 8, message = "新しいパスワードは8文字以上で入力してください"))]
    pub new_password: String,
    #[validate(must_match(other = "new_password", message = "パスワードが一致しません"))]
    pub confirm_password: String,
}

// 🟡 検索・フィルタ用パラメータ
#[derive(Debug, Deserialize)]
pub struct UserSearchParams {
    pub email: Option<String>,     // メールアドレス部分検索
    pub role: Option<UserRole>,    // 役割フィルタ
    pub page: Option<u32>,         // ページ番号
    pub per_page: Option<u32>,     // 1ページあたりの件数
}
```

### 出力値（型、形式、例）
```rust
// 🟢 ユーザー情報レスポンス（機密情報除外）
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub pid: String,
    pub name: String,
    pub email: String,
    pub role: String,                // "admin", "trainer", "instructor"
    pub email_verified: bool,        // email_verified_atから判定
    pub created_at: String,
    pub updated_at: String,
}

// 🟡 ユーザー一覧レスポンス（ページネーション対応）
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

// 🟢 操作結果レスポンス
#[derive(Debug, Serialize)]
pub struct UserOperationResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserResponse>, // 作成・更新時のみ
}
```

### **参照したEARS要件**: REQ-101（ユーザー管理機能の入出力定義）
### **参照した設計文書**: 既存models/users.rsの型定義

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

### セキュリティ要件（NFR-XXXから抽出）
- 🟢 **RBAC統合**: 管理者のみがユーザーCRUD操作可能（TASK-102統合）
- 🟢 **パスワードハッシュ化**: bcryptによる安全なパスワード保存（既存実装継続）
- 🟡 **パスワード複雑性**: 最小8文字、大文字小文字数字を含む（新規実装）
- 🟡 **セッション検証**: 全操作でセッション有効性チェック（TASK-101統合）

### パフォーマンス要件（NFR-XXXから抽出）
- 🟡 **一覧表示**: 1000ユーザーまでのページネーション対応（20件/ページ）
- 🟡 **メール重複チェック**: データベースインデックスによる高速チェック（既存活用）
- 🟡 **応答時間**: ユーザー操作は2秒以内でレスポンス

### アーキテクチャ制約（architecture.mdから抽出）
- 🟢 **Loco.rs MVC**: 既存のモデル・コントローラー・ビューパターンに準拠
- 🟢 **SeaORM統合**: 既存のORM実装パターンとの一貫性
- 🟢 **PostgreSQL制約**: usersテーブル構造との整合性（roleカラム追加要）

### データベース制約（database-schema.sqlから抽出）
- 🟢 **メール一意制約**: 既存のemail uniqueインデックス活用
- 🔴 **役割カラム追加**: usersテーブルへのroleカラム追加マイグレーション必要
- 🟡 **外部キー整合性**: 他テーブルとの関連性維持

### **参照したEARS要件**: NFR-101（セキュリティ）, NFR-201（パフォーマンス）
### **参照した設計文書**: architecture.md認証セクション、既存usersテーブル定義

---

## 4. 想定される使用例（EARSEdgeケース・データフローベース）

### 正常系シナリオ

#### シナリオ1: 管理者による新規ユーザー作成
```rust
// 🟢 管理者が新しい研修担当者を作成
let params = UserParams {
    name: "田中太郎".to_string(),
    email: "tanaka@example.com".to_string(),
    password: Some("SecurePass123".to_string()),
    role: UserRole::Trainer,
};
let result = User::create_user(&db, &params).await;
assert_eq!(result.success, true);
```

#### シナリオ2: 役割変更によるアクセス権限変更
```rust
// 🟢 trainerからadminへの権限昇格
let updated_user = User::update_role(&db, user_id, UserRole::Admin).await;
assert_eq!(updated_user.role, "admin");
```

#### シナリオ3: ユーザー自身によるパスワード変更
```rust
// 🟢 現在のパスワードを確認して新しいパスワードに変更
let change_params = PasswordChangeParams {
    current_password: "OldPass123".to_string(),
    new_password: "NewPass456".to_string(),
    confirm_password: "NewPass456".to_string(),
};
let result = User::change_password(&db, user_id, &change_params).await;
assert_eq!(result.success, true);
```

### エッジケース（EDGE-XXXから抽出）

#### エッジケース1: 最後の管理者の削除防止
- 🟡 **状況**: システム内の唯一の管理者を削除しようとする場合
- 🟡 **期待動作**: 削除を拒否し、適切なエラーメッセージを表示

#### エッジケース2: 自己の役割降格
- 🟡 **状況**: 管理者が自分自身の役割をtrainerやinstructorに変更する場合
- 🟡 **期待動作**: 確認ダイアログ表示後に実行、セッション再認証要求

### エラーケース（EDGE-XXXエラー処理から抽出）

#### エラーケース1: メールアドレス重複
```rust
// 🟢 既に存在するメールアドレスでの登録試行
let result = User::create_user(&db, &duplicate_params).await;
assert_eq!(result.success, false);
assert!(result.message.contains("メールアドレスが既に使用されています"));
```

#### エラーケース2: 権限不足による操作拒否
```rust
// 🟢 trainer役割によるユーザー作成試行（権限不足）
let result = User::create_user(&db, &params, trainer_context).await;
assert_eq!(result.success, false);
assert!(result.message.contains("この操作を実行する権限がありません"));
```

#### エラーケース3: 不正なパスワード変更
```rust
// 🟢 現在のパスワードが間違っている場合
let result = User::change_password(&db, user_id, &invalid_params).await;
assert_eq!(result.success, false);
assert!(result.message.contains("現在のパスワードが正しくありません"));
```

### **参照したEARS要件**: REQ-101（管理者専用操作）, EDGE-001（データ整合性）
### **参照した設計文書**: データフロー図のユーザー管理フロー

---

## 5. 実装要件・技術制約

### 必須実装項目
- 🔴 **データベースマイグレーション**: usersテーブルへのroleカラム追加
- 🟢 **UserRole enum拡張**: 既存RBAC実装との統合
- 🟢 **CRUD API実装**: RESTfulなユーザー管理API
- 🟡 **フロントエンド**: ユーザー管理画面（一覧・作成・編集・削除）

### テスト要件
- 🟢 **単体テスト**: ユーザーモデルのCRUD操作とバリデーション
- 🟢 **統合テスト**: RBAC統合とセッション認証連携
- 🟡 **UIテスト**: フォーム操作とエラーハンドリング
- 🟡 **セキュリティテスト**: 権限昇格攻撃防止

### UI/UX要件
- 🟡 **レスポンシブデザイン**: テーブルとフォームのモバイル対応
- 🟡 **バリデーションフィードバック**: リアルタイムフォームバリデーション
- 🟡 **操作確認**: 削除・役割変更時の確認ダイアログ
- 🟡 **アクセシビリティ**: ARIA属性とキーボード操作対応

---

## 6. EARS要件・設計文書との対応関係

### 参照したユーザストーリー
- **管理者ストーリー**: システム利用者の管理を行いたい
- **セキュリティストーリー**: 適切な権限管理を行いたい

### 参照した機能要件
- **REQ-101**: 管理者権限（ユーザー管理機能へのアクセス）
- **REQ-102**: 研修担当者権限（限定的なユーザー情報アクセス）

### 参照した非機能要件
- **NFR-101**: セキュリティ要件（認証・認可）
- **NFR-201**: パフォーマンス要件（レスポンス時間）

### 参照した設計文書
- **アーキテクチャ**: 既存の認証・認可システム構成
- **データベース**: usersテーブル構造（roleカラム追加要）
- **型定義**: 既存models/users.rsの構造体
- **API設計**: RESTfulなエンドポイント設計

---

## 7. データフロー

```
HTTP Request (Admin Only)
    ↓
Session + RBAC Middleware
    ↓ authentication & authorization
User Management Controller
    ↓ validation & business logic
User Model + Database
    ↓ CRUD operations
Response (JSON/HTML)
```

### 権限チェックプロセス
1. **セッション検証**: TASK-101のセッション有効性確認
2. **RBAC認可**: TASK-102の権限チェック（admin権限要求）
3. **ビジネスロジック**: ユーザー操作の実行
4. **データベース操作**: CRUDクエリの実行
5. **レスポンス生成**: 成功・エラーレスポンスの返却

---

## 8. 完了条件

### 機能要件
- 🟢 **管理者専用**: adminのみがユーザーCRUD操作を実行可能
- 🟢 **役割管理**: 3つの役割の設定・変更が正常動作
- 🟢 **メール重複防止**: 重複チェック機能が確実に動作
- 🟢 **パスワード管理**: 安全なパスワード変更機能

### 非機能要件
- 🟡 **性能**: ユーザー一覧表示が2秒以内で完了
- 🟡 **セキュリティ**: 権限昇格攻撃の防御
- 🟡 **UI/UX**: 直感的で使いやすいインターフェース

---

**要件定義完了日**: 2025-08-23
**品質判定**: ⚠️ 要改善（データベースマイグレーション要、UI実装詳細要確認）

**重要な実装前提**: usersテーブルにroleカラムを追加するマイグレーションが必須