# TASK-004: ORM設定とモデル実装 - TDD要件定義書

## 機能概要

**機能名**: ORM設定とモデル実装
**実装日**: 2025-08-17
**担当者**: システム管理者

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

🟢 **データベーススキーマに基づく全エンティティモデルの実装**
- SeaORMを使用した全13エンティティの実装（Users, Companies, Students, Materials, Trainings, TrainingMaterials, Projects, ProjectParticipants, Interviews, Meetings, Sessions, AuditLogs）
- エンティティ間のリレーション設定とクエリ最適化
- ビジネスロジックバリデーションの実装

🟢 **システム内での位置づけ**
- Loco.rsフレームワークのMVCアーキテクチャのModelレイヤー
- データベースアクセスレイヤーとして全コントローラー・サービスから利用
- セッションベース認証システムとの統合

**参照したEARS要件**: 全エンティティ要件、database-schema.sql
**参照した設計文書**: architecture.md（MVCアーキテクチャ）、interfaces.ts（型定義）

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

🟢 **エンティティ構造（interfaces.tsベース）**
```rust
// Company エンティティ例
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub contact_person: String,
    pub contact_email: String,
    pub chat_link: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

🟢 **CRUD操作仕様**
- Create: ActiveModel→insert()→Result<Model, DbErr>
- Read: Entity::find()→filter()→one/all()→Result<Option<Model>, DbErr>
- Update: ActiveModel→update()→Result<Model, DbErr>
- Delete: Entity::delete()→Result<DeleteResult, DbErr>

🟢 **リレーション仕様**
- 1対多: Company→Students, Training→TrainingMaterials
- 多対多: Training↔Material（through TrainingMaterial）
- 参照制約: projects.company_id→companies.id (RESTRICT)

**参照したEARS要件**: データ関連要件全般
**参照した設計文書**: interfaces.ts（全エンティティ型定義）、database-schema.sql

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

🟢 **データベース制約（database-schema.sqlベース）**
- UUID主キーの必須実装
- 外部キー制約18個の実装（CASCADE, RESTRICT, SET NULL）
- 一意制約4個（メール重複防止、教材順序重複防止等）
- CHECK制約（日付妥当性、enumeration値等）

🟢 **パフォーマンス要件**
- インデックス51個以上の活用
- N+1問題回避のためのeager loading実装
- クエリ最適化（select_only, join活用）

🟡 **セキュリティ制約**
- パスワードハッシュ化（bcrypt）必須
- 監査ログ自動記録
- SQLインジェクション防御（SeaORM標準）

**参照したEARS要件**: NFR-101〜105（セキュリティ）、NFR-201〜205（パフォーマンス）
**参照した設計文書**: architecture.md（パフォーマンス設計）、database-schema.sql

## 4. 想定される使用例（EARSEdgeケース・データフローベース）

🟢 **基本的な使用パターン（通常要件）**
- 企業作成→受講者追加→研修コース設計→プロジェクト実施のワークフロー
- 教材検索→研修コースへの紐付け→期間設定
- プロジェクト参加者の面談スケジューリング

🟡 **エッジケース**
- 外部キー制約違反（企業削除時の受講者存在チェック）
- 同一プロジェクトへの重複参加防止
- 研修コース内での教材順序重複防止

🟡 **エラーケース**
- データベース接続失敗時のエラーハンドリング
- バリデーション失敗時の詳細エラーメッセージ
- トランザクション失敗時のロールバック

**参照したEARS要件**: 基本機能要件、制約違反処理要件
**参照した設計文書**: dataflow.md（データフロー図）

## 5. EARS要件・設計文書との対応関係

🟢 **参照したユーザストーリー**: [教材管理ストーリー、研修コース設計ストーリー、プロジェクト管理ストーリー]
🟢 **参照した機能要件**: [全エンティティ関連要件、CRUD操作要件]
🟢 **参照した非機能要件**: [NFR-101〜105（セキュリティ）、NFR-201〜205（パフォーマンス）]
🟡 **参照したEdgeケース**: [データ整合性チェック、制約違反処理]
🟢 **参照した受け入れ基準**: [CRUD操作テスト、リレーション機能テスト、バリデーションテスト]

🟢 **参照した設計文書**:
- **アーキテクチャ**: architecture.md（MVCアーキテクチャ、SeaORM設計）
- **データフロー**: dataflow.md（エンティティ間データフロー）
- **型定義**: interfaces.ts（全エンティティインターフェース）
- **データベース**: database-schema.sql（テーブル設計、制約、インデックス）
- **API仕様**: api-endpoints.md（CRUD APIエンドポイント）

## 実装計画

### Phase 1: エンティティ生成
- SeaORM CLIによる全エンティティファイル生成
- 既存Usersモデルとの整合性確認

### Phase 2: リレーション実装
- 1対多、多対多リレーションの設定
- Eager loading設定

### Phase 3: バリデーション実装
- フィールドレベルバリデーション
- ビジネスロジックバリデーション

### Phase 4: テスト実装
- 単体テスト（各モデルCRUD）
- 統合テスト（リレーション機能）
- バリデーションテスト

## 品質基準

- コードカバレッジ: 80%以上
- 全CRUD操作の正常動作確認
- リレーション機能の正常動作確認
- バリデーションエラーの適切な発生確認