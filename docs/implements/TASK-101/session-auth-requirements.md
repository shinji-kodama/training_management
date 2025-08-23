# TASK-101: セッションベース認証実装 - TDD要件定義書

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

### 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測

**機能名**: セッションベース認証実装

**何をする機能か**:
- 既存JWTベース認証からセッションベース認証への移行
- 安全なログイン・ログアウト処理の実装
- セッション管理とCSRF保護機能の提供
- セッションミドルウェアによる認証チェック

**解決する問題**:
- JWTトークンのクライアント側保存によるセキュリティリスク軽減
- サーバー側でのセッション制御（強制ログアウト、セッション管理）
- CSRF攻撃防御の強化
- セッション有効期限の柔軟な管理

**想定されるユーザー**:
- システム管理者（admin）
- 研修担当者（trainer）  
- 講師（instructor）

**システム内での位置づけ**:
- フェーズ2認証・認可システムの基盤機能
- 全てのコア機能（フェーズ3）へのアクセス制御基盤
- TASK-102（RBAC）の前提条件

**参照したEARS要件**: REQ-001（ユーザー認証）, REQ-002（役割ベース認証）, REQ-407（セッション管理）, NFR-102（セキュリティ要件）
**参照した設計文書**: CLAUDE.md（セッションベース認証の方針）

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

### 🟢 **青信号**: 既存実装から抽出

### 入力パラメータ

**ログイン処理**:
```rust
// 既存のLoginParams構造体を活用
pub struct LoginParams {
    pub email: String,        // バリデーション: email形式
    pub password: String,     // バリデーション: 最小8文字
}
```

**セッション作成処理**:
```rust
pub struct SessionCreateParams {
    pub user_id: i32,         // usersテーブルのID
    pub session_token: String, // 暗号学的に安全な乱数
    pub expires_at: DateTimeWithTimeZone, // セッション有効期限
}
```

### 出力値

**ログインレスポンス**:
```rust
// セッション方式に変更されたレスポンス
pub struct SessionLoginResponse {
    pub success: bool,
    pub user: UserInfo,       // ユーザー基本情報
    pub csrf_token: String,   // CSRF保護トークン
}
```

**セッション状態確認レスポンス**:
```rust
pub struct SessionCurrentResponse {
    pub user: UserInfo,       // 認証済みユーザー情報
    pub csrf_token: String,   // 新しいCSRFトークン
}
```

### データフロー

1. **ログインフロー**: POST /api/auth/login → 認証情報検証 → セッション作成 → Cookieセット
2. **認証チェックフロー**: リクエスト → セッションミドルウェア → セッション検証 → ユーザー情報注入
3. **ログアウトフロー**: POST /api/auth/logout → セッション削除 → Cookie削除

**参照したEARS要件**: REQ-001, REQ-407
**参照した設計文書**: src/controllers/auth.rs（既存JWT実装）, src/models/users.rs（認証ロジック）

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

### 🟢 **青信号**: EARS要件定義書・設計文書を参考

### セキュリティ要件（NFR-102）

**CSRF保護**:
- 全ての状態変更操作でCSRFトークン検証
- セッション作成時にCSRFトークン生成
- ワンタイムトークンによるリプレイ攻撃防止

**セッション管理**:
- セッション有効期限: 24時間（設定可能）
- アイドルタイムアウト: last_accessed_at の自動更新
- セッション固定攻撃対策（ログイン時のトークン再生成）

**Cookieセキュリティ**:
- HttpOnly属性: JavaScript からのアクセス防止
- Secure属性: HTTPS通信必須（本番環境）
- SameSite=Strict: CSRF攻撃防止

### パフォーマンス要件

**セッション検索性能**:
- session_tokenでの検索: 一意制約インデックス活用
- セッションクリーンアップ: 期限切れセッション定期削除
- メモリ使用量最適化: 必要最小限の情報のみセッションに格納

### アーキテクチャ制約

**Loco.rs 0.16.3フレームワーク制約**:
- Axumミドルウェアパターン準拠
- SeaORM 1.1.12 ORM活用
- 既存Authenticableトレイト実装の活用

**データベース制約**:
- 既存sessionsテーブル活用（CSRFトークン追加が必要）
- 外部キー制約: users.id → sessions.user_id
- session_tokenの一意制約活用

**API制約**:
- 既存auth エンドポイント構造維持
- JWTレスポンス形式からセッション形式への変更
- 後方互換性の維持（段階的移行）

**参照したEARS要件**: NFR-102（セキュリティ）, REQ-407（セッション管理）
**参照した設計文書**: CLAUDE.md（アーキテクチャ制約）, src/models/_entities/sessions.rs（データベース制約）

## 4. 想定される使用例（EARSEdgeケース・データフローベース）

### 🟡 **黄信号**: EARS要件定義書から妥当な推測

### 基本的な使用パターン

**正常ログインシナリオ**:
1. ユーザーがログインフォームに認証情報入力
2. POST /api/auth/login でメール・パスワード送信
3. users::Model::verify_password による認証検証
4. セッション作成（sessions テーブルへINSERT）
5. HttpOnly Cookieでセッショントークン設定
6. CSRFトークンをレスポンスで返却

**認証が必要な操作**:
1. 保護されたリソースへのリクエスト
2. セッションミドルウェアでCookieからセッショントークン取得
3. sessionsテーブルでトークン検証・有効期限チェック
4. last_accessed_at の更新
5. リクエストにユーザー情報注入

### エッジケース

**EDGE-001: セッション期限切れ**:
- expires_at 超過時の自動ログアウト
- 期限切れ検出時のセッション削除
- ログインページへの適切なリダイレクト

**EDGE-002: 不正セッション**:
- 存在しないセッショントークンでのアクセス
- セッショントークン改ざんの検出
- 不正アクセス試行のセキュリティログ記録

**EDGE-003: セッション競合**:
- 同一ユーザーの複数セッション許可
- セッション上限管理（ユーザーあたり最大5セッション）
- 古いセッションの自動無効化

**EDGE-004: CSRF攻撃防御**:
- 状態変更操作でのCSRFトークン検証
- トークン不一致時の処理拒否
- 攻撃試行の監査ログ記録

### エラーケース

**ERR-001: ログイン失敗**:
- 不正なメールアドレス・パスワード
- 複数回失敗時のアカウントロック機能
- ブルートフォース攻撃対策

**ERR-002: セッション作成失敗**:
- データベース接続エラー時の適切な処理
- セッション生成エラーのハンドリング
- 一時的な障害時のリトライ機構

**参照したEARS要件**: TASK-101完了条件（正しい認証情報でログイン成功、不正な認証情報でログイン拒否、セッション管理正常動作、CSRF攻撃防御）
**参照した設計文書**: 既存src/controllers/auth.rs の認証フロー

## 5. EARS要件・設計文書との対応関係

### 参照したユーザストーリー
- システム利用者の安全なログイン・ログアウト機能

### 参照した機能要件
- **REQ-001**: ユーザー認証機能
- **REQ-002**: 役割ベース認証（後続TASK-102の前提）
- **REQ-407**: セッション管理機能

### 参照した非機能要件
- **NFR-102**: セキュリティ要件（CSRF保護、セッション管理）

### 参照したEdgeケース
- TASK-101 完了条件に基づく基本的なエッジケース想定

### 参照した受け入れ基準
- 正しい認証情報でログイン成功
- 不正な認証情報でログイン拒否  
- セッション管理が正常動作
- CSRF攻撃が防御される

### 参照した設計文書
- **アーキテクチャ**: CLAUDE.md（Loco.rs 0.16.3、セッションベース認証方針）
- **データベース**: src/models/_entities/sessions.rs（既存sessionsテーブル）
- **既存実装**: src/controllers/auth.rs（JWTからセッションへの移行対象）, src/models/users.rs（認証ロジック活用）
- **ORM**: SeaORM 1.1.12 による既存モデル構造

## 技術実装仕様

### 必要な実装変更

**1. セッション管理ロジック追加**:
```rust
// src/models/sessions.rs に追加
impl Model {
    pub async fn create_session(db: &DatabaseConnection, user_id: i32) -> ModelResult<Self>
    pub async fn find_by_token(db: &DatabaseConnection, token: &str) -> ModelResult<Self>
    pub async fn invalidate_session(db: &DatabaseConnection, token: &str) -> ModelResult<()>
    pub async fn cleanup_expired_sessions(db: &DatabaseConnection) -> ModelResult<()>
}
```

**2. セッションミドルウェア実装**:
```rust
// src/middleware/session_auth.rs（新規作成）
pub async fn session_auth_middleware(
    State(ctx): State<AppContext>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode>
```

**3. 認証コントローラー修正**:
```rust
// src/controllers/auth.rs の修正
// JWTレスポンス → セッション＋CSRF方式への変更
async fn login(State(ctx): State<AppContext>, Json(params): Json<LoginParams>) -> Result<Response>
async fn logout(State(ctx): State<AppContext>) -> Result<Response>
async fn current(State(ctx): State<AppContext>) -> Result<Response>
```

### データベーススキーマ活用
既存のsessionsテーブル:
- id (UUID): 主キー
- user_id (i32): 外部キー（users.id） 
- session_token (String): セッション識別子（一意制約）
- expires_at (DateTimeWithTimeZone): セッション有効期限
- created_at (DateTimeWithTimeZone): 作成日時
- last_accessed_at (DateTimeWithTimeZone): 最終アクセス日時

**注意**: CSRF保護のため、マイグレーションでcsrf_tokenカラムの追加が必要になる可能性があります。

### 必要な依存関係
```toml
# Cargo.toml への追加（暗号学的機能用）
cookie = "0.18"      # セキュアCookie管理
rand = "0.8"         # 暗号学的に安全な乱数生成
```

---

**要件定義完了日**: 2025-08-23  
**次のステップ**: `/tdd-testcases` でテストケースの洗い出しを実行