# TDD開発メモ: 監査ログ（AuditLogs）モデル実装

## 概要

- 機能名: 監査ログ（AuditLogs）モデルの実装
- 開発開始: 2025-08-23T09:00:00+09:00
- 現在のフェーズ: Red（失敗するテスト作成完了）

## 関連ファイル

- 要件定義: `docs/design/training-management/database-schema.sql`
- 実装ファイル: `src/models/audit_logs.rs`（要実装）
- テストファイル: `tests/models/audit_logs.rs`（作成済み）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2025-08-23T09:00:00+09:00

### テストケース

**対象**: 監査ログ管理機能の基本CRUD操作とビジネスルール制約

1. **test_監査ログの正常作成**: 基本的なCRUD操作（Create）の動作確認
2. **test_ユーザー別監査ログ検索**: ユーザーと監査ログ間の1対多リレーション機能確認
3. **test_アクション別監査ログ検索**: アクション種別による監査ログフィルタリング機能確認
4. **test_リソース別監査ログ検索**: リソース種別・IDによる監査ログ検索機能確認
5. **test_匿名監査ログ作成**: user_id NULL での監査ログ作成機能確認
6. **test_ユーザー参照整合性制約**: 外部キー制約（ON DELETE SET NULL）の動作確認

### テストコード

**データベーススキーマ分析結果**:
```sql
-- AuditLogs テーブル構造
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL, -- 'login', 'create_material', 'update_training', etc.
    resource_type VARCHAR(50), -- 'user', 'material', 'training', etc.
    resource_id UUID,
    details JSONB, -- 詳細情報（JSON形式）
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

**実際のデータベース構造**:
- users テーブルは integer 主キー（id）とUUID（pid）を持つ
- audit_logs.user_id は integer 型で users.id を参照
- resource_id は UUID 型（リソースの pid を参照）

**実装されたテストケース**:
```rust
// tests/models/audit_logs.rs

// 1. 基本的な監査ログ作成テスト
#[tokio::test]
#[serial]
async fn test_監査ログの正常作成() {
    // 監査ログ作成の基本機能をテスト
    // 外部キー関係（user_id）の正常動作を確認
    // UUID主キー自動生成とタイムスタンプ自動設定を確認
    // JSONB詳細情報、INET型IPアドレス、TEXT型ユーザーエージェントの保存確認
}

// 2. ユーザー別監査ログ一覧取得テスト
#[tokio::test]
#[serial]
async fn test_ユーザー別監査ログ検索() {
    // AuditLogs::find_by_user_id()メソッドのテスト
    // 1対多リレーション（ユーザー→監査ログ）の動作確認
    // 複数監査ログの管理機能をテスト
}

// 3. アクション別監査ログ検索テスト
#[tokio::test]
#[serial]
async fn test_アクション別監査ログ検索() {
    // AuditLogs::find_by_action() および find_by_actions()メソッドのテスト
    // アクション種別によるフィルタリング検索機能をテスト
    // 複数アクション条件での検索機能をテスト
}

// 4. リソース別監査ログ検索テスト
#[tokio::test]
#[serial]
async fn test_リソース別監査ログ検索() {
    // AuditLogs::find_by_resource()メソッドのテスト
    // resource_type + resource_id の複合条件検索機能をテスト
    // 特定リソースの操作履歴追跡機能をテスト
}

// 5. 匿名監査ログ作成テスト
#[tokio::test]
#[serial]
async fn test_匿名監査ログ作成() {
    // user_id が NULL の監査ログ作成機能をテスト
    // システム操作やゲスト操作の記録機能をテスト
    // ON DELETE SET NULL 制約の動作確認
}

// 6. ユーザー参照整合性制約テスト
#[tokio::test]
#[serial]
async fn test_ユーザー参照整合性制約() {
    // 外部キー制約の動作確認
    // 存在しないuser_idでの監査ログ作成防止機能をテスト
}
```

### 期待される失敗

**実際の失敗メッセージ**:
```
error[E0599]: no function or associated item named `find_by_user_id` found for struct `training_management::models::audit_logs::Model` in the current scope
error[E0599]: no function or associated item named `find_by_action` found for struct `training_management::models::audit_logs::Model` in the current scope
error[E0599]: no function or associated item named `find_by_actions` found for struct `training_management::models::audit_logs::Model` in the current scope  
error[E0599]: no function or associated item named `find_by_resource` found for struct `training_management::models::audit_logs::Model` in the current scope
```

**失敗理由**:
- `src/models/audit_logs.rs` モジュールに検索メソッドが未実装
- バリデーション機能が未実装
- UUID主キー自動生成機能（ActiveModelBehavior）が基本的な実装のみ
- 現在の実装は最小限の基本構造のみ

### 次のフェーズへの要求事項

Greenフェーズで実装すべき内容：

1. **AuditLogsモデル実装**:
   - `src/models/audit_logs.rs` の完全な実装
   - ActiveModel, Model の実装
   - バリデーション機能の実装（任意・監査ログは基本的に入力検証なし）
   - UUID主キー自動生成（audit_logsはupdated_atなし）

2. **検索機能**:
   - find_by_user_id メソッドの実装
   - find_by_action メソッドの実装
   - find_by_actions メソッドの実装（複数アクション検索）
   - find_by_resource メソッドの実装（resource_type + resource_id検索）

3. **UUID主キー自動生成**:
   - ActiveModelBehavior での before_save() でUUID自動生成
   - created_at の自動設定（updated_at なし）

4. **制約対応**:
   - ユーザー参照整合性制約の適切なエラーハンドリング
   - ON DELETE SET NULL の動作確認
   - JSONB、INET、TEXT型フィールドの適切な処理

5. **最小限の実装要件**:
   - テストが実行できる最小限のコード
   - ユーザーとの外部キー関係の確認
   - 1対多リレーション検索機能の基本実装
   - アクション・リソース別検索機能の基本実装

## テスト実行コマンド

```bash
# 監査ログ関連テスト実行
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test audit_logs

# 特定のテスト実行
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test test_監査ログの正常作成

# 全テスト実行
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test
```

## 期待される失敗メッセージ

- コンパイルエラー: `no function or associated item named find_by_user_id found`
- コンパイルエラー: `no function or associated item named find_by_action found`
- コンパイルエラー: `no function or associated item named find_by_actions found`
- コンパイルエラー: `no function or associated item named find_by_resource found`

## TDD品質評価

✅ **高品質のRedフェーズ**:
- **テスト実行**: 期待通りコンパイルエラーで失敗
- **期待値**: 明確で具体的（UUID生成、外部キーリレーション、検索機能等）
- **アサーション**: 適切（各フィールドの値確認、リレーション確認、検索結果確認）
- **実装方針**: 明確（SeaORM + Loco.rs パターン、既存モデルとの一貫性）

## 信頼性レベル

🟢 **高信頼性**: database-schema.sqlのテーブル定義、外部キー制約に完全準拠
🟢 **テストパターン**: 既存のモデルと同等のテスト品質
🟢 **ビジネスルール**: 監査ログ管理の実際の要件に即した現実的なテストケース
🟡 **データ型調整**: 実際のDB構造（users.id=integer）に合わせて調整済み

## 特徴的なビジネスロジック

**AuditLogsモデルの独自制約**:
1. **ユーザー参照整合性**: 監査ログは有効なユーザーに対してのみ関連付け可能（NULL許可）
2. **アクション制約**: アクション種別は文字列（'login', 'create_material', 'update_training'等）
3. **リソース情報**: resource_type + resource_id による操作対象の追跡
4. **JSONB詳細情報**: 操作の詳細情報をJSON形式で柔軟に記録
5. **SET NULL削除**: ユーザーが削除されると関連監査ログのuser_idがNULLに設定
6. **匿名操作対応**: システム操作等でuser_id=NULLの監査ログ作成可能
7. **IPアドレス・ユーザーエージェント**: セキュリティ監査のためのメタデータ記録
8. **タイムスタンプ**: created_atのみ（audit_logsは変更不可のため、updated_atなし）

## データ型対応

**実際のデータベース構造との整合性**:
- `user_id`: Option<i32> （users.id への参照）
- `resource_id`: Option<Uuid> （各リソースのpidへの参照）
- `details`: Option<Json> （JSONB型）
- `ip_address`: Option<String> （INET型をStringで処理）
- `user_agent`: Option<String> （TEXT型）

## Greenフェーズ（最小実装）

### 実装日時

2025-08-23T09:30:00+09:00

### 実装方針

**TDD Greenフェーズ原則に基づく最小実装**:
- 失敗していたテストを通すための最小限のコード実装
- UUID主キー自動生成機能の追加
- 監査ログ検索機能の基本実装（ユーザー別、アクション別、リソース別）
- integer型user_idとUUID型resource_idの適切な処理

### 実装コード

**src/models/audit_logs.rs の最小実装**:
```rust
/**
 * 【ActiveModelBehavior実装】: 監査ログエンティティのライフサイクル管理
 * 【実装方針】: TDD Greenフェーズの原則に従い、テストを通すための最小限実装
 * 【テスト対応】: Red フェーズで作成されたテストケースを通すための実装
 * 🟢 信頼性レベル: audit_logsテーブル構造に適合した実装
 */

// 1. UUID主キー自動生成機能（updated_at不要）
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr> {
        if insert {
            let mut this = self;
            this.id = sea_orm::ActiveValue::Set(uuid::Uuid::new_v4());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// 2. 監査ログ検索機能
impl Model {
    // ユーザー別監査ログ検索
    pub async fn find_by_user_id(db: &C, user_id: i32) -> ModelResult<Vec<Model>>
    
    // アクション別監査ログ検索
    pub async fn find_by_action(db: &C, action: &str) -> ModelResult<Vec<Model>>
    
    // 複数アクション別監査ログ検索
    pub async fn find_by_actions(db: &C, actions: &[&str]) -> ModelResult<Vec<Model>>
    
    // リソース別監査ログ検索
    pub async fn find_by_resource(db: &C, resource_type: &str, resource_id: uuid::Uuid) -> ModelResult<Vec<Model>>
}
```

**実装した機能**:
1. **UUID主キー自動生成**: `before_save()`でinsert時にUUID v4を自動生成（audit_logsはupdated_atなし）
2. **ユーザー別監査ログ検索**: `find_by_user_id()`メソッドでuser_idによる絞り込み検索
3. **アクション別監査ログ検索**: `find_by_action()`および`find_by_actions()`メソッドでアクション条件による検索
4. **リソース別監査ログ検索**: `find_by_resource()`メソッドでresource_type + resource_idによる複合条件検索
5. **適切な型処理**: integer型user_idとUUID型resource_idの正しい処理

### テスト結果

**実行コマンド**:
```bash
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test audit_logs
```

**テスト結果**:
```
running 6 tests
test models::audit_logs::test_リソース別監査ログ検索 ... ok
test models::audit_logs::test_監査ログの正常作成 ... ok
test models::audit_logs::test_ユーザー別監査ログ検索 ... ok
test models::audit_logs::test_アクション別監査ログ検索 ... ok
test models::audit_logs::test_ユーザー参照整合性制約 ... ok
test models::audit_logs::test_匿名監査ログ作成 ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 52 filtered out; finished in 1.62s
```

**✅ 全テストケースが正常に通過**:
- **基本CRUD操作**: UUID生成、外部キー関係、タイムスタンプ自動設定が正常動作
- **ユーザー別検索**: ユーザー別監査ログ一覧取得が正常動作
- **アクション別検索**: 単一・複数アクション条件による検索が正常動作
- **リソース別検索**: resource_type + resource_id複合条件検索が正常動作
- **匿名監査ログ**: user_id=NULLでの監査ログ作成が正常動作
- **外部キー参照整合性**: データベース制約による制約チェックが正常動作

### 課題・改善点

**現在の最小実装の制限**:
1. **検索機能の拡張**: 日付範囲別、IPアドレス別検索等の追加検索機能
2. **エラーハンドリング**: 制約違反時の詳細なエラーメッセージ対応
3. **ドキュメント**: メソッドの詳細なドキュメンテーション
4. **パフォーマンス最適化**: 大量ログ検索時のページネーション対応

**Refactorフェーズで改善予定の項目**:
- 検索機能の拡張（日付範囲、IPアドレス、複合条件）
- エラーハンドリングの強化
- メソッドのドキュメンテーション追加
- パフォーマンス最適化（インデックス活用、ページネーション）
- セキュリティ強化

## 次のフェーズ

Refactorフェーズではコード品質の向上と機能拡張を行う。

## TASK-004完了確認

**AuditLogsモデル実装完了**: TASK-004で要求されていた最後のエンティティ（AuditLogs）のTDD実装が完了。
- ✅ Red フェーズ完了: 6つの包括的テストケースで期待される失敗を確認
- ✅ Green フェーズ完了: 全6テストが正常に通過する最小実装を完了