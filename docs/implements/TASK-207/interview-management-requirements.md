# TASK-207 面談管理機能 - TDD要件定義書

**作成日**: 2025-08-24  
**対象機能**: TASK-207 面談管理機能（Interview Management System）  
**TDDフェーズ**: Requirements Definition  

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

**🟢 青信号**: TASK-207要件定義とデータベーススキーマを参考にしてほぼ推測していない場合

**機能名**: 面談管理機能（Interview Management System）

**機能概要**:
- **何をする機能か**: プロジェクト参加者との個別面談の予約・実施・記録を包括的に管理するシステム
- **解決する問題**: プロジェクト進行中の受講者とトレーナー間の面談スケジュール管理、面談記録の体系的保存、面談完了状況の可視化
- **想定ユーザー**: 
  - **管理者**: 全面談の監督・管理
  - **トレーナー**: 担当プロジェクトの面談実施・記録
  - **講師**: 面談記録の参照（読み取り専用）
- **システム内での位置づけ**: プロジェクト管理機能（TASK-206）の下位機能として、project_participant単位での個別面談管理を担当

**参照したEARS要件**: REQ-008, REQ-013, REQ-107, REQ-108, REQ-403
**参照した設計文書**: interviews テーブルスキーマ、models/interviews.rs実装

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

**🟢 青信号**: 既存のinterviewsモデル・データベーススキーマを参考にしてほぼ推測していない場合

### 入力パラメータ
```rust
// 面談作成・更新用の入力パラメータ
struct InterviewRequest {
    project_participant_id: Uuid,        // 必須: プロジェクト参加者ID
    interviewer_id: Uuid,                // 必須: 面談実施者ID（⚠️型不整合要解決）
    scheduled_at: DateTime<Utc>,         // 必須: 面談予定日時
    notes: Option<String>,               // 任意: Markdown形式の面談記録
    status: InterviewStatus,             // 必須: scheduled/completed/cancelled
}

// 検索フィルター用パラメータ
struct InterviewSearchParams {
    project_id: Option<Uuid>,            // プロジェクト絞り込み
    participant_id: Option<Uuid>,        // 参加者絞り込み
    interviewer_id: Option<Uuid>,        // 面談者絞り込み
    status: Option<InterviewStatus>,     // ステータス絞り込み
    date_from: Option<NaiveDate>,        // 期間絞り込み（開始）
    date_to: Option<NaiveDate>,          // 期間絞り込み（終了）
}
```

### 出力仕様
```rust
// 面談詳細情報
struct InterviewResponse {
    id: Uuid,
    project_participant: ProjectParticipantSummary,
    interviewer: UserSummary,
    scheduled_at: DateTime<Utc>,
    status: InterviewStatus,
    notes: Option<String>,              // Markdown形式
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// 面談一覧レスポンス
struct InterviewListResponse {
    interviews: Vec<InterviewSummary>,
    total_count: i64,
    completion_stats: CompletionStats,
}
```

### 制約条件
- **日時制約**: scheduled_at は現在時刻以降である必要がある
- **権限制約**: interviewer_id はトレーナー権限以上のユーザーである必要がある
- **一意制約**: 同一interviewer_idの同一時間帯での重複予約は不可
- **Markdown制約**: notes フィールドは10,000文字以内

**参照したEARS要件**: REQ-008（面談CRUD）、REQ-013（Markdown記録）
**参照した設計文書**: models/interviews.rs の構造体定義

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

**🟢 青信号**: 完了済みTASK-206のセキュリティパターンとTASK-102 RBAC設計を参考

### セキュリティ要件
- **認証**: SessionAuth必須（TASK-101パターン準拠）
- **認可**: RBAC統合（admin: 全操作、trainer: 担当プロジェクト、instructor: 読み取り専用）
- **CSRF保護**: 全CUD操作でCSRFトークン検証必須
- **XSS防止**: Markdownコンテンツの適切なサニタイゼーション
- **入力検証**: 全パラメータでバリデーション実施

### パフォーマンス要件
- **レスポンス時間**: 面談一覧表示 < 2秒
- **同時接続**: 50人同時アクセス対応
- **データベースクエリ**: N+1問題回避（eager loading使用）

### データ整合性制約
- **外部キー制約**: project_participant_id, interviewer_id の参照整合性保証
- **ステータス制約**: scheduled → completed/cancelled の一方向遷移
- **時間制約**: 過去日時での新規面談作成禁止

**参照したEARS要件**: REQ-403（1対1面談）、TASK-102 RBAC設計
**参照した設計文書**: interviews テーブル制約、TASK-206セキュリティパターン

## 4. 想定される使用例（EARSEdgeケース・データフローベース）

**🟢 青信号**: TASK-206プロジェクト管理機能の使用パターンと既存面談テストケースを参考

### 基本的な使用パターン
1. **面談予約フロー**:
   - トレーナーがプロジェクト参加者の面談を予約
   - カレンダーインターフェースでの日時選択
   - 時間競合チェックでの重複回避

2. **面談実施・記録フロー**:
   - 面談実施後のステータス更新（scheduled → completed）
   - Markdownエディタでの面談記録入力
   - 次回面談の設定促進（REQ-108）

3. **面談管理・監督フロー**:
   - 管理者による全面談状況の監督
   - プロジェクト別面談完了率の確認
   - アラート機能での未完了面談通知（REQ-107）

### エッジケース
- **時間競合**: 同一面談者の重複予約時のエラー処理
- **過去日時**: 過去日時での面談作成時のバリデーションエラー
- **権限外アクセス**: 他社プロジェクトへの不正アクセス時の403エラー
- **参加者削除**: project_participant削除時のカスケード削除処理
- **長文記録**: Markdown記録が制限文字数を超過した場合の処理

### エラーケース
- **存在しない参加者**: 無効なproject_participant_id指定時の404エラー
- **権限不足**: instructor権限でのCUD操作時の403エラー
- **同時更新**: 複数ユーザーによる同一面談の同時編集競合

**参照したEARS要件**: REQ-107（アラート）、REQ-108（次回面談促進）
**参照した設計文書**: tests/models/interviews.rs のテストケース

## 5. EARS要件・設計文書との対応関係

### 参照したユーザストーリー
- **面談予約ストーリー**: "トレーナーとして、受講者との面談を効率的に予約したい"
- **面談記録ストーリー**: "トレーナーとして、面談内容をMarkdown形式で詳細に記録したい"
- **進捗管理ストーリー**: "管理者として、全プロジェクトの面談完了状況を一目で把握したい"

### 参照した機能要件
- **REQ-008**: 個別面談の予約・実施・記録を管理
- **REQ-013**: 面談記録をMarkdown形式で入力・保存
- **REQ-107**: 全面談完了時のアラート機能
- **REQ-108**: 面談記録入力時の次回面談設定促進
- **REQ-403**: 面談を1対1でのみ実施

### 参照した非機能要件
- **TASK-102**: RBAC権限制御（admin/trainer/instructor）
- **TASK-101**: SessionAuth認証システム
- **TASK-206**: CSRFプロテクション・XSS防止パターン

### 参照した設計文書
- **アーキテクチャ**: Loco.rs MVC + HTMX パターン
- **データベース**: interviews テーブル設計（UUID主キー、外部キー制約）
- **型定義**: models/interviews.rs の構造体・enum定義
- **API仕様**: RESTful設計（プロジェクトスコープ + 個別アクセス）

## 6. 重要な技術的注意事項

### ⚠️ 解決必要な課題
1. **型不整合問題**: interviewer_id がスキーマ（UUID）とエンティティ（i32）で不一致
2. **Controller層未実装**: 完全に新規実装が必要
3. **Calendar UI設計**: 面談スケジュール可視化のUI設計必要

### ✅ 活用可能な既存資産
1. **Model層**: 高品質な業務ロジック実装済み（446行）
2. **Test層**: 包括的なモデルテスト実装済み（4テスト）
3. **TASK-206パターン**: セキュリティ・認証の成功パターン
4. **Database層**: 完全な制約・トリガー実装済み

---

**要件定義品質判定**: ✅ **高品質**
- 要件の曖昧さ: なし（既存実装と設計文書ベース）
- 入出力定義: 完全（型レベルで詳細定義）
- 制約条件: 明確（セキュリティ・性能・整合性）
- 実装可能性: 確実（既存パターンとModel層活用）

**次のステップ**: `/tdd-testcases` でテストケースの洗い出しを行います。