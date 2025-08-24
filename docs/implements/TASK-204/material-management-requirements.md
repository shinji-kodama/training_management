# TASK-204: 教材管理機能 - TDD要件定義書

**タスクID**: TASK-204  
**作成日**: 2025-08-24  
**フェーズ**: TDD Requirements Definition  
**実装対象**: 教材管理機能（Controller層・View層）

## 事前準備完了

✅ **TDD関連ファイル読み込み完了**
- 既存Model層実装状況確認（materials.rs: 完全実装済み）
- アーキテクチャ設計文書確認（architecture.md）
- 既存Controller/View実装パターン確認（auth.rs, dashboard.rs）
- テスト実行結果確認（materials関連7テスト全通過）

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

🟢 **何をする機能か（ユーザストーリーから抽出）**
- 研修担当者が教材情報をWebブラウザ経由で管理する機能
- 教材のCRUD操作（作成・参照・更新・削除）をWebインターフェース提供
- URLからのドメイン自動抽出によるユーザビリティ向上
- おすすめ度の表示制御による情報セキュリティ対応

🟢 **どのような問題を解決するか（As a / So that から抽出）**
- As a 研修担当者, So that 教材情報を効率的に管理できる
- As a システム利用者, So that おすすめ度情報が適切にアクセス制御される
- As a データ入力者, So that URL入力時にドメインが自動設定される

🟢 **想定されるユーザー（As a から抽出）**
- 研修担当者（trainer role）
- システム管理者（admin role）
- 研修講師（instructor role）: 読み取り専用アクセス

🟢 **システム内での位置づけ（アーキテクチャ設計から抽出）**
- Loco.rsモノリシックMVC + HTMXアーキテクチャのController・View層
- 既存Model層（materials.rs）との統合
- セッションベース認証・RBAC統合
- TASK-205研修コース管理機能との連携準備

**参照したEARS要件**: REQ-003, REQ-004, REQ-102, REQ-106  
**参照した設計文書**: architecture.md（MVCアーキテクチャ）

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

🟢 **入力パラメータ（materials.rs ActiveModel構造ベース）**
```rust
// 教材作成・編集フォーム入力
struct MaterialForm {
    title: String,              // 必須、1-255文字
    url: String,                // 必須、有効なURL形式
    description: String,        // 必須、1文字以上
    recommendation_level: i32,  // 必須、1-5の範囲
    // domain: String,          // URL自動抽出により設定
    // created_by: i32,         // セッションから自動設定
}
```

🟢 **出力値（View層レスポンス形式）**
- **HTMLページ**: 教材一覧・詳細・作成・編集画面
- **HTMX部分更新**: フォームバリデーション、検索結果表示
- **リダイレクト**: CRUD操作成功時の画面遷移
- **エラーページ**: バリデーション失敗・権限不足時の表示

🟢 **データフロー（既存materials.rs統合）**
```
Browser → Controller → Model(materials.rs) → Database
    ↑                    ↓
  View ← Template ← View Layer
```

**参照したEARS要件**: REQ-003, REQ-004  
**参照した設計文書**: materials.rs（Model実装）、architecture.md（データフロー）

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

🟢 **パフォーマンス要件（アーキテクチャ設計から抽出）**
- 教材一覧表示: 2秒以内の応答時間
- ページネーション: 最大100件/ページの制限
- サーバーサイドテンプレートキャッシュ活用
- 既存materials.rsのインデックス活用

🟢 **セキュリティ要件（NFR-101〜105から抽出）**
- CSRF対策: フォームトークンとHTMX統合
- セッションベース認証: Cookie + サーバーサイドセッション
- RBAC統合: role別のアクセス制御
- おすすめ度表示制御: ログイン状態による情報制御

🟢 **アーキテクチャ制約（architecture.mdから抽出）**
- モノリシックMVC + HTMXパターン準拠
- Loco.rsフレームワーク機能活用
- 必要最小限のHTMX利用（フォームバリデーション、検索結果表示のみ）
- サーバーサイドレンダリング中心

🟡 **互換性要件（既存実装との統合）**
- 既存materials.rsモデルとの100%互換性
- 既存auth/dashboard Controller実装パターン踏襲
- 既存RBAC middleware統合

**参照したEARS要件**: NFR-101〜105, NFR-201〜205, REQ-406  
**参照した設計文書**: architecture.md（セキュリティ・パフォーマンス設計）

## 4. 想定される使用例（EARSEdgeケース・データフローベース）

🟢 **基本的な使用パターン（通常要件から抽出）**
- **教材登録フロー**: フォーム表示 → 入力 → バリデーション → 保存 → 一覧画面
- **教材検索フロー**: 一覧画面 → 検索条件入力 → HTMX部分更新 → 結果表示
- **教材編集フロー**: 一覧 → 詳細 → 編集フォーム → 更新 → 詳細画面
- **ドメイン自動抽出**: URL入力 → フィールドフォーカス外す → ドメイン自動設定

🟡 **エッジケース（既存materials.rsテスト結果ベース）**
- 不正URL入力: HTMLバリデーション + サーバーサイドバリデーション
- 推奨レベル範囲外: HTMLフォーム制限 + サーバーサイドバリデーション  
- 長すぎるタイトル: 文字数カウンタ表示 + バリデーションエラー
- セッション期限切れ: 自動ログイン画面リダイレクト

🟡 **アクセス制御ケース（RBAC統合）**
- admin権限: 全CRUD操作可能
- trainer権限: 全CRUD操作可能
- instructor権限: 読み取り専用（一覧・詳細のみ）
- 未ログイン: ログイン画面リダイレクト

**参照したEARS要件**: REQ-003, REQ-004, REQ-102, REQ-106  
**参照した設計文書**: materials.rsテスト結果、RBAC実装パターン

## 5. EARS要件・設計文書との対応関係

🟢 **参照したユーザストーリー**: [教材管理ストーリー（研修担当者向け）]  
🟢 **参照した機能要件**: [REQ-003: 教材CRUD, REQ-004: 教材検索, REQ-102: RBAC統合, REQ-106: UI制御]  
🟡 **参照した非機能要件**: [NFR-101〜105: セキュリティ, NFR-201〜205: パフォーマンス, NFR-204: HTMX統合]  
🟡 **参照したEdgeケース**: [materials.rsテスト結果ベースのエッジケース]  
🟢 **参照した受け入れ基準**: [UI操作テスト, CRUD機能テスト, RBAC統合テスト]

🟢 **参照した設計文書**:
- **アーキテクチャ**: architecture.md（MVCパターン、HTMXガイドライン）
- **既存実装**: materials.rs（Model層）、auth.rs/dashboard.rs（Controller実装パターン）
- **データベース**: materials table（materials.rsテストで検証済み）
- **認証統合**: セッションベース認証 + RBAC middleware統合

## 6. APIエンドポイント仕様（Controller層実装要件）

🟢 **教材管理画面エンドポイント（MVCパターンベース）**
```
GET    /materials                    # 教材一覧画面表示
GET    /materials/new                # 教材作成フォーム表示
POST   /materials                    # 教材作成処理
GET    /materials/{id}               # 教材詳細画面表示
GET    /materials/{id}/edit          # 教材編集フォーム表示
POST   /materials/{id}               # 教材更新処理
DELETE /materials/{id}               # 教材削除処理
GET    /materials/search             # 教材検索（HTMX）
```

🟡 **HTMX部分更新エンドポイント（必要最小限）**
```
POST   /materials/validate           # フォームバリデーション
GET    /materials/search-results     # 検索結果部分更新
POST   /materials/extract-domain     # URL入力時ドメイン抽出
```

## 7. View層実装要件（Loco.rsテンプレート）

🟢 **テンプレートファイル構成**
```
src/views/materials/
├── mod.rs                     # View layer export
├── list.html                  # 教材一覧画面
├── detail.html                # 教材詳細画面  
├── form.html                  # 教材作成・編集フォーム
├── search_results.html        # 検索結果部分テンプレート（HTMX）
└── components/
    ├── material_card.html     # 教材カードコンポーネント
    └── form_fields.html       # フォームフィールドコンポーネント
```

🟡 **テンプレート機能要件**
- **レスポンシブデザイン**: Tailwind CSS活用
- **アクセシビリティ**: ARIA属性、キーボード操作対応
- **HTMX統合**: 必要最小限の部分更新機能
- **エラー表示**: フィールド単位のバリデーションエラー表示

## 8. 実装優先度とフェーズ分け

### 🟢 Phase 1: Controller層実装（高優先度）
- MaterialController基本CRUD実装
- 既存auth/dashboardパターン踏襲
- RBAC middleware統合
- basic view templating

### 🟡 Phase 2: View層実装（中優先度）  
- HTMLテンプレート作成
- フォームバリデーション実装
- エラーハンドリング表示

### 🟡 Phase 3: HTMX機能実装（低優先度）
- 検索結果部分更新
- フォームライブバリデーション
- ドメイン自動抽出UI

## 9. 成功条件と品質基準

### ✅ 実装完了基準
- [ ] 全エンドポイントが正常レスポンス
- [ ] RBAC権限制御が正常動作
- [ ] 既存materials.rsモデルとの統合
- [ ] フォームバリデーションが適切に動作
- [ ] おすすめ度の表示制御が機能

### ✅ 品質基準
- [ ] UI操作テスト: 全画面で正常操作可能
- [ ] セキュリティテスト: CSRF・RBAC対策済み
- [ ] パフォーマンステスト: 2秒以内の応答時間
- [ ] アクセシビリティテスト: WCAG 2.1 AA準拠

---

**要件定義品質判定**: ✅ 高品質
- 要件の曖昧さ: なし（既存Model層との統合により明確）
- 入出力定義: 完全（materials.rsベースで確実）
- 制約条件: 明確（architecture.mdベース）
- 実装可能性: 確実（既存パターン踏襲）

**次のお勧めステップ**: `/tdd-testcases` でテストケースの洗い出しを行います。