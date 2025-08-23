# 研修管理システム

ToB IT研修提供者向けの包括的な研修管理システムです。[Loco.rs](https://loco.rs) フレームワークを使用して構築されており、研修の計画から実施、評価まで一元管理できます。

## プロジェクト概要

本システムは研修提供者が企業向けIT研修を効率的に管理するためのWebアプリケーションです。受講者管理、教材管理、研修コース設計、プロジェクト管理、個別面談・定例会のスケジューリングなど、研修運営に必要な全機能を提供します。

## 完了した機能

### TASK-001: 開発環境セットアップ ✅
- **実装日**: 2025-08-17
- **概要**: Loco.rs開発環境とPostgreSQL環境の構築
- **設定内容**:
  - Rust + Cargo環境構築
  - Docker Compose設定（PostgreSQL 15）
  - 環境変数設定（.env）
- **動作確認**: `cargo --version`, `docker-compose up`, PostgreSQL接続確認

### TASK-002: Loco.rsプロジェクト初期化 ✅
- **実装日**: 2025-08-17
- **概要**: Loco.rsプロジェクトの初期化と基本設定
- **設定内容**:
  - Loco.rsプロジェクト初期化
  - 基本設定ファイル作成（config/development.yaml）
  - フォルダ構造整理
  - 基本ルーティング設定（ダッシュボード）
- **動作確認**: サーバー起動、ヘルスチェックエンドポイント確認

### TASK-003: データベーススキーマ実装 ✅
- **実装日**: 2025-08-17
- **概要**: PostgreSQLデータベーススキーマの完全実装
- **設定内容**:
  - 13個のテーブル作成（users, sessions, companies, students, materials, trainings, training_materials, projects, project_participants, interviews, meetings, audit_logs）
  - 51個以上のインデックス設定
  - 18個の外部キー制約
  - 4個の一意制約
  - 9個のトリガー（updated_at自動更新、データ整合性チェック）
  - 3個の関数（update_updated_at_column, check_project_participant_company, cleanup_expired_sessions）
  - 初期データ投入（管理者ユーザー、サンプル企業）
- **動作確認**: 全テーブル作成、制約動作、トリガー動作、初期データ確認

## セットアップ手順

### 前提条件
- Rust 1.70+
- Docker 20.0+
- Docker Compose v2.0+
- PostgreSQL 15+（Dockerで自動セットアップ）

### インストール

```bash
# リポジトリのクローン
git clone <repository-url>
cd training_management

# 依存関係のインストール
cargo build

# PostgreSQLの起動
docker-compose up -d postgres

# データベースマイグレーション
cargo loco db migrate

# アプリケーションの起動
cargo loco start
```

## 設定

### 環境変数（.env）

```bash
# データベース設定
DATABASE_URL=postgres://postgres:postgres@localhost:6543/training_management

# アプリケーション設定
RUST_LOG=debug
APP_PORT=5150

# セキュリティ設定（開発環境用）
JWT_SECRET=development_secret_key
APP_SECRET=development_app_secret
```

### 設定ファイル

- `config/development.yaml`: Loco.rs開発環境設定
- `docker-compose.yml`: PostgreSQL環境設定
- `.env`: 環境変数設定

## 起動方法

### 開発サーバーの起動

```bash
# 通常起動
cargo loco start

# ファイル監視付き起動（推奨）
cargo loco watch
```

### データベース操作

```bash
# マイグレーション実行
cargo loco db migrate

# マイグレーション状態確認
cargo loco db status

# データベースリセット
cargo loco db reset

# テーブル構造確認
cargo loco db schema
```

## アクセス情報

### アプリケーション
- **URL**: http://localhost:5150
- **ヘルスチェック**: http://localhost:5150/_health

### 初期管理者ユーザー
- **Email**: admin@example.com
- **Password**: admin123

### データベース
- **Host**: localhost:6543
- **Database**: training_management
- **User**: postgres
- **Password**: postgres

## 使用技術

### バックエンド
- **フレームワーク**: Loco.rs 0.16.3
- **言語**: Rust 1.87.0
- **ORM**: SeaORM 1.1.14
- **データベース**: PostgreSQL 15

### フロントエンド
- **テンプレート**: Tera（Server-side rendering）
- **CSS**: Tailwind CSS
- **JavaScript**: HTMX（必要最小限）

### インフラ
- **コンテナ**: Docker & Docker Compose
- **開発環境**: 本プロジェクト設定

## 開発

### 開発環境の準備

1. 依存関係のインストール: `cargo build`
2. PostgreSQL起動: `docker-compose up -d postgres`
3. マイグレーション実行: `cargo loco db migrate`
4. 開発サーバー起動: `cargo loco watch`

### Loco CLI の活用

```bash
# モデル生成
cargo loco generate model <name>

# コントローラー生成
cargo loco generate controller <name>

# マイグレーション生成
cargo loco generate migration <name>

# システム健全性チェック
cargo loco doctor

# ルート一覧確認
cargo loco routes
```

### テスト

```bash
# 単体テスト実行
cargo test

# 統合テスト実行（今後実装予定）
cargo test --test integration

# カバレッジ測定（今後実装予定）
cargo tarpaulin
```

## データベース構造

### 主要テーブル

| テーブル名 | 説明 | 主要カラム |
|-----------|------|----------|
| users | ユーザー（研修提供者） | id, email, name, role |
| companies | 企業情報 | id, name, contact_person, contact_email |
| students | 受講者情報 | id, name, email, company_id, role_type |
| materials | 教材情報 | id, title, url, domain, recommendation_level |
| trainings | 研修コース | id, title, description, prerequisites, goals |
| projects | 実施プロジェクト | id, training_id, company_id, title |
| interviews | 個別面談 | id, project_participant_id, interviewer_id |
| meetings | 定例会 | id, project_id, title, recurrence_type |

### 制約・関係性

- **外部キー制約**: 18個（参照整合性保証）
- **一意制約**: 4個（重複防止）
- **トリガー**: 9個（自動更新・整合性チェック）
- **インデックス**: 51個以上（検索性能向上）

## トラブルシューティング

### よくある問題

#### 問題1: データベース接続エラー
- **症状**: `DB connection failed`
- **解決方法**:
  ```bash
  docker-compose up -d postgres
  # PostgreSQLの起動を確認
  docker logs training_management_postgres
  ```

#### 問題2: マイグレーションエラー
- **症状**: `Migration failed`
- **解決方法**:
  ```bash
  # マイグレーション状態確認
  cargo loco db status
  # 必要に応じてリセット
  cargo loco db reset
  ```

#### 問題3: ポート競合
- **症状**: `Address already in use`
- **解決方法**:
  ```bash
  # ポート使用状況確認
  lsof -i :5150
  # 他のプロセス終了またはポート変更
  ```

### ログ確認

```bash
# アプリケーションログ
cargo loco start --environment development

# PostgreSQLログ
docker logs training_management_postgres

# 詳細ログ（デバッグ時）
RUST_LOG=trace cargo loco start
```

## 貢献・開発参加

### 開発フロー
1. 設計確認（要件・アーキテクチャ）
2. 実装（TDDまたはDIRECT）
3. テスト（単体・統合・UI）
4. レビュー（コード・ドキュメント）
5. デプロイ（開発環境での確認）

### コード品質基準
- テストカバレッジ80%以上
- セキュリティ要件充足
- UI/UX要件充足
- パフォーマンス要件充足

## 更新履歴

- **2025-08-17**: TASK-001 開発環境セットアップ完了
- **2025-08-17**: TASK-002 Loco.rsプロジェクト初期化完了
- **2025-08-17**: TASK-003 データベーススキーマ実装完了
- **2025-08-17**: Loco.rs 0.16.3へアップデート完了

## 今後の予定

### 次のマイルストーン（フェーズ2）
- **TASK-004**: ORM設定とモデル実装
- **TASK-101**: セッションベース認証実装
- **TASK-102**: 役割ベースアクセス制御（RBAC）実装

### 長期計画
- 全42タスクの段階的実装
- CI/CD パイプライン構築
- 本番環境デプロイ
- 運用ドキュメント整備

## ライセンス

本プロジェクトは開発中です。ライセンスについては後日決定予定です。

## サポート・問い合わせ

開発に関する質問や提案については、プロジェクトの課題管理システムをご利用ください。
