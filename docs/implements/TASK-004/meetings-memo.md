# TDD開発メモ: 定例会（Meetings）モデル実装

## 概要

- 機能名: 定例会（Meetings）モデルの実装
- 開発開始: 2025-08-22T16:00:00+09:00
- 現在のフェーズ: Red（失敗するテスト作成完了）

## 関連ファイル

- 要件定義: `docs/design/training-management/database-schema.sql`
- 実装ファイル: `src/models/meetings.rs`（要実装）
- テストファイル: `tests/models/meetings.rs`（作成済み）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2025-08-22T16:00:00+09:00

### テストケース

**対象**: 定例会管理機能の基本CRUD操作とビジネスルール制約

1. **test_定例会の正常作成**: 基本的なCRUD操作（Create）の動作確認
2. **test_プロジェクト別定例会一覧取得**: プロジェクトと定例会間の1対多リレーション機能確認
3. **test_繰り返し設定制約バリデーション**: CHECK制約（繰り返し設定と終了日の整合性）の動作確認
4. **test_繰り返し種別制約バリデーション**: CHECK制約（'none', 'weekly', 'biweekly'）の動作確認
5. **test_プロジェクト参照整合性制約**: 外部キー制約の動作確認

### テストコード

**データベーススキーマ分析結果**:
```sql
-- Meetings テーブル構造
CREATE TABLE meetings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    scheduled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    recurrence_type VARCHAR(20) NOT NULL DEFAULT 'none' CHECK (recurrence_type IN ('none', 'weekly', 'biweekly')),
    recurrence_end_date DATE, -- 繰り返し終了日
    instructor_id UUID REFERENCES users(id) ON DELETE SET NULL, -- 任意参加の研修講師
    notes TEXT, -- Markdown形式の研修記録
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    -- 繰り返し設定がある場合は終了日が必須
    CHECK (
        (recurrence_type = 'none') OR
        (recurrence_type != 'none' AND recurrence_end_date IS NOT NULL)
    )
);
```

**実装されたテストケース**:
```rust
// tests/models/meetings.rs

// 1. 基本的な定例会作成テスト
#[tokio::test]
#[serial]
async fn test_定例会の正常作成() {
    // 定例会作成の基本機能をテスト
    // 外部キー関係（project_id, created_by, instructor_id）の正常動作を確認
    // UUID主キー自動生成とタイムスタンプ自動設定を確認
    // 繰り返し設定制約とMarkdown記録保存の正常動作を確認
}

// 2. プロジェクト別定例会一覧取得テスト
#[tokio::test]
#[serial]
async fn test_プロジェクト別定例会一覧取得() {
    // Meeting::find_by_project_id()メソッドのテスト
    // 1対多リレーション（プロジェクト→定例会）の動作確認
    // 複数定例会の管理機能をテスト
}

// 3. 繰り返し設定制約テスト
#[tokio::test]
#[serial]
async fn test_繰り返し設定制約バリデーション() {
    // CHECK制約（繰り返し設定と終了日の整合性）の動作確認
    // 'weekly'/'biweekly'で終了日NULL時の制約違反防止機能をテスト
}

// 4. 繰り返し種別制約テスト
#[tokio::test]
#[serial]
async fn test_繰り返し種別制約バリデーション() {
    // CHECK制約（'none', 'weekly', 'biweekly'）の動作確認
    // 無効な繰り返し種別値での定例会作成防止機能をテスト
}

// 5. プロジェクト参照整合性制約テスト
#[tokio::test]
#[serial]
async fn test_プロジェクト参照整合性制約() {
    // 外部キー制約の動作確認
    // 存在しないproject_idでの定例会作成防止機能をテスト
}
```

### 期待される失敗

**実際の失敗メッセージ**:
```
error[E0599]: no function or associated item named `find_by_project_id` found for struct `training_management::models::meetings::Model` in the current scope
   --> tests/models/meetings.rs:227:66
    |
227 |     let meetings = training_management::models::meetings::Model::find_by_project_id(&boot.app_context.db, project.id).await.unwrap();
    |                                                                  ^^^^^^^^^^^^^^^^^^ function or associated item not found in `Model`
```

**失敗理由**:
- `src/models/meetings.rs` モジュールに `find_by_project_id` メソッドが未実装
- バリデーション機能が未実装
- UUID主キー自動生成機能（ActiveModelBehavior）が基本的な実装のみ
- 現在の実装はSeaORMエンティティの基本構造のみ

### 次のフェーズへの要求事項

Greenフェーズで実装すべき内容：

1. **Meetingsモデル実装**:
   - `src/models/meetings.rs` の完全な実装
   - ActiveModel, Model の実装
   - バリデーション機能の実装
   - find_by_project_id メソッドの実装

2. **バリデーション機能**:
   - 必須フィールド（project_id, title, scheduled_at, recurrence_type, created_by）の検証
   - recurrence_type値の妥当性チェック（'none', 'weekly', 'biweekly'）
   - 繰り返し設定と終了日の整合性チェック
   - 定例会時刻の妥当性チェック
   - Markdown記録の保存機能

3. **UUID主キー自動生成**:
   - ActiveModelBehavior での before_save() でUUID自動生成
   - created_at/updated_at の自動設定

4. **検索機能**:
   - プロジェクト別定例会一覧検索（project_id条件）
   - 繰り返し種別別定例会検索
   - 講師別定例会検索

5. **制約対応**:
   - プロジェクト参照整合性制約の適切なエラーハンドリング
   - 繰り返し種別制約の適切なエラーハンドリング
   - 繰り返し設定と終了日制約の適切なエラーハンドリング
   - CASCADE削除の動作確認

6. **最小限の実装要件**:
   - テストが実行できる最小限のコード
   - プロジェクトとの外部キー関係の確認
   - 作成者・講師との外部キー関係の確認
   - 1対多リレーション検索機能の基本実装

## テスト実行コマンド

```bash
# 定例会関連テスト実行
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test meetings

# 特定のテスト実行
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test test_定例会の正常作成

# 全テスト実行
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test
```

## 期待される失敗メッセージ

- コンパイルエラー: `no function or associated item named find_by_project_id found`
- モジュール機能不在エラー: バリデーション機能、UUID生成機能等が未実装

## TDD品質評価

✅ **高品質のRedフェーズ**:
- **テスト実行**: 期待通りコンパイルエラーで失敗
- **期待値**: 明確で具体的（UUID生成、外部キーリレーション、制約確認等）
- **アサーション**: 適切（各フィールドの値確認、リレーション確認、制約確認）
- **実装方針**: 明確（SeaORM + Loco.rs パターン、既存モデルとの一貫性）

## 信頼性レベル

🟢 **高信頼性**: database-schema.sqlのテーブル定義、外部キー制約、CHECK制約に完全準拠
🟢 **テストパターン**: 既存Interviews、Projects、Companies、Studentsモデルと同等のテスト品質
🟢 **ビジネスルール**: 定例会管理の実際の要件に即した現実的なテストケース

## 特徴的なビジネスロジック

**Meetingsモデルの独自制約**:
1. **プロジェクト参照整合性**: 定例会は有効なプロジェクトに対してのみ開催可能
2. **繰り返し種別制約**: 繰り返し種別は'none', 'weekly', 'biweekly'のいずれか
3. **繰り返し設定整合性**: 繰り返し設定が'none'以外の場合、終了日が必須
4. **講師任意参加**: 講師の参加は任意でNULL許可
5. **CASCADE削除**: プロジェクトが削除されると関連定例会も自動削除
6. **RESTRICT削除**: 定例会作成者（ユーザー）は関連定例会がある限り削除不可
7. **Markdown記録**: 定例会記録はMarkdown形式でのリッチテキスト保存

## Greenフェーズ（最小実装）

### 実装日時

2025-08-22T16:30:00+09:00

### 実装方針

**TDD Greenフェーズ原則に基づく最小実装**:
- 失敗していたテストを通すための最小限のコード実装
- UUID主キー自動生成機能の追加
- プロジェクト別定例会検索機能の基本実装
- 繰り返し種別制約バリデーション機能の実装

### 実装コード

**src/models/meetings.rs の最小実装**:
```rust
/**
 * 【ActiveModelBehavior実装】: 定例会エンティティのライフサイクル管理
 * 【実装方針】: TDD Greenフェーズの原則に従い、テストを通すための最小限実装
 * 【テスト対応】: Red フェーズで作成されたテストケースを通すための実装
 * 🟢 信頼性レベル: SeaORM標準パターンと既存モデルの実装パターンに準拠
 */

// 1. UUID主キー自動生成機能
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr> {
        self.validate()?; // バリデーション実行
        
        if insert {
            let mut this = self;
            this.id = sea_orm::ActiveValue::Set(uuid::Uuid::new_v4());
            Ok(this)
        } else if self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// 2. プロジェクト別定例会検索機能
impl Model {
    pub async fn find_by_project_id<C>(
        db: &C,
        project_id: uuid::Uuid
    ) -> ModelResult<Vec<Model>> {
        Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .order_by_asc(Column::ScheduledAt)
            .all(db)
            .await
    }
}

// 3. 繰り返し種別バリデーション機能
#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(custom(function = "validate_recurrence_type"))]
    pub recurrence_type: String,
}

fn validate_recurrence_type(recurrence_type: &str) -> Result<(), validator::ValidationError> {
    const ALLOWED_RECURRENCE_TYPES: &[&str] = &["none", "weekly", "biweekly"];
    if ALLOWED_RECURRENCE_TYPES.contains(&recurrence_type) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_recurrence_type"))
    }
}
```

**実装した機能**:
1. **UUID主キー自動生成**: `before_save()`でinsert時にUUID v4を自動生成
2. **プロジェクト別定例会検索**: `find_by_project_id()`メソッドでproject_idによる絞り込み検索
3. **繰り返し種別制約バリデーション**: CHECK制約と連動したアプリケーション層での事前検証
4. **updated_at自動更新**: 更新時の自動タイムスタンプ設定

### テスト結果

**実行コマンド**:
```bash
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test meetings
```

**テスト結果**:
```
running 5 tests
test models::meetings::test_繰り返し設定制約バリデーション ... ok
test models::meetings::test_繰り返し種別制約バリデーション ... ok
test models::meetings::test_プロジェクト別定例会一覧取得 ... ok
test models::meetings::test_プロジェクト参照整合性制約 ... ok
test models::meetings::test_定例会の正常作成 ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 2.38s
```

**✅ 全テストケースが正常に通過**:
- **基本CRUD操作**: UUID生成、外部キー関係、タイムスタンプ自動設定が正常動作
- **1対多リレーション検索**: プロジェクト別定例会一覧取得が正常動作
- **繰り返し種別制約バリデーション**: CHECK制約と連動したバリデーションが正常動作
- **繰り返し設定制約**: データベース制約による整合性チェックが正常動作
- **外部キー参照整合性**: データベース制約による制約チェックが正常動作

### 課題・改善点

**現在の最小実装の制限**:
1. **検索機能の拡張**: 講師別、繰り返し種別別検索等の追加検索機能
2. **エラーハンドリング**: 制約違反時の詳細なエラーメッセージ対応
3. **ドキュメント**: メソッドの詳細なドキュメンテーション
4. **パフォーマンス最適化**: 検索クエリのさらなる最適化

**Refactorフェーズで改善予定の項目**:
- 検索機能の拡張（講師別、繰り返し種別別、日付範囲別）
- エラーハンドリングの強化
- メソッドのドキュメンテーション追加
- パフォーマンス最適化
- セキュリティ強化

## 次のフェーズ

Refactorフェーズではコード品質の向上と機能拡張を行う。