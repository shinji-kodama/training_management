# TASK-202: 企業管理機能 - TDD Red Phase 完了

**作成日時**: 2025-08-23  
**フェーズ**: TDD Red Phase（失敗するテスト作成）  
**ステータス**: ✅ **完了**

## 📝 作成したテストケース

### 【TC-202-008】企業削除制約違反エラーテスト 🔴
**ファイル**: `tests/models/companies.rs:130-193`

```rust
#[tokio::test]
#[serial]
async fn test_受講者存在時企業削除制約違反エラー() {
    // 【テスト目的】: 企業削除制約ビジネスロジックの確認
    // 【テスト内容】: 受講者が存在する企業の削除試行
    // 【期待される動作】: 制約違反エラー発生と削除処理失敗
    // 🔴 信頼性レベル: 未実装メソッド呼び出しのため失敗予定（Redフェーズ対象）
    
    // 未実装メソッド呼び出し
    let result = training_management::models::companies::Model::delete_with_constraints(
        &boot.app_context.db, 
        company.id
    ).await;
}
```

### 【TC-202-007】RBAC権限不足エラーテスト 🔴
**ファイル**: `tests/models/companies.rs:195-247`

```rust
#[tokio::test]
#[serial]
async fn test_非管理者権限による企業作成拒否() {
    // 【テスト目的】: RBAC権限制御の確認
    // 【テスト内容】: Trainer/Instructor権限での企業作成試行
    // 【期待される動作】: 権限不足エラー発生と操作拒否
    // 🔴 信頼性レベル: RBAC統合機能未実装のため失敗予定（Redフェーズ対象）
    
    // 未実装メソッド呼び出し
    let result = training_management::models::companies::Model::create_with_rbac(
        &boot.app_context.db,
        &auth_context,
        company_data
    ).await;
}
```

### 【TC-202-004】制約なし企業削除テスト 🔴
**ファイル**: `tests/models/companies.rs:249-293`

```rust
#[tokio::test]
#[serial]
async fn test_受講者なし企業の正常削除() {
    // 【テスト目的】: 制約なし企業の削除機能確認
    // 【テスト内容】: 受講者が存在しない企業の削除処理
    // 【期待される動作】: データベースから企業レコードの安全な削除
    // 🔴 信頼性レベル: delete_with_constraints()メソッド未実装のため失敗予定
    
    // 未実装メソッド呼び出し
    let result = training_management::models::companies::Model::delete_with_constraints(
        &boot.app_context.db,
        company.id
    ).await;
}
```

## 🔴 期待される失敗メッセージ

### **コンパイルエラー（期待通り）**
```
error[E0599]: no function or associated item named `delete_with_constraints` found for struct `training_management::models::companies::Model` in the current scope
   --> tests/models/companies.rs:173:65
    |
173 |     let result = training_management::models::companies::Model::delete_with_constraints(
    |                                                                 ^^^^^^^^^^^^^^^^^^^^^^^ function or associated item not found in `Model`

error[E0599]: no function or associated item named `create_with_rbac` found for struct `training_management::models::companies::Model` in the current scope
   --> tests/models/companies.rs:231:65
    |
231 |     let result = training_management::models::companies::Model::create_with_rbac(
    |                                                                 ^^^^^^^^^^^^^^^^ function or associated item not found in `Model`

error[E0599]: no function or associated item named `find_by_id` found for struct `training_management::models::companies::Model` in the current scope
   --> tests/models/companies.rs:190:79
    |
190 |     let company_still_exists = training_management::models::companies::Model::find_by_id(&boot.app_context.db, company.id).await;
    |                                                                               ^^^^^^^^^^ function or associated item not found in `Model`
```

## 🎯 未実装メソッド一覧

Greenフェーズで実装が必要なメソッド：

### **1. 企業削除制約機能**
```rust
impl Model {
    /// 制約チェック付き企業削除
    pub async fn delete_with_constraints(
        db: &DatabaseConnection, 
        company_id: Uuid
    ) -> ModelResult<()>
    
    /// ID指定での企業検索
    pub async fn find_by_id(
        db: &DatabaseConnection, 
        company_id: Uuid
    ) -> ModelResult<Option<Model>>
    
    /// 受講者数カウント（削除制約判定用）
    pub async fn count_students(
        db: &DatabaseConnection, 
        company_id: Uuid
    ) -> ModelResult<u64>
}
```

### **2. RBAC統合機能**
```rust
impl Model {
    /// RBAC権限チェック付き企業作成
    pub async fn create_with_rbac(
        db: &DatabaseConnection,
        auth_context: &rbac::AuthContext,
        company_data: ActiveModel
    ) -> ModelResult<Model>
    
    /// RBAC権限チェック付き企業更新
    pub async fn update_with_rbac(
        db: &DatabaseConnection,
        auth_context: &rbac::AuthContext,
        company_id: Uuid,
        company_data: ActiveModel
    ) -> ModelResult<Model>
}
```

## 📊 テストコード品質評価

### ✅ **高品質な日本語コメント**
- **テスト目的**: 各テストの意図が明確
- **期待動作**: 具体的な期待結果を記述
- **信頼性レベル**: 🔴で未実装メソッドを明示
- **処理説明**: Given-When-Then構造で整理

### ✅ **適切な失敗設計**
- **コンパイルエラー**: 未実装メソッド呼び出しで確実に失敗
- **テスト環境**: 既存パターン踏襲（serial_test, boot_test）
- **関連データ**: 外部キー制約テストの事前データ準備完了

### ✅ **TDDサイクル準備完了**
- **Red達成**: 失敗テストの確実な作成
- **Green準備**: 実装すべきメソッドシグネチャ明確化
- **Refactor準備**: コード品質向上の方針策定

## 🚀 Greenフェーズへの要求事項

### **Phase 1: 基本削除機能実装**
1. `find_by_id()` メソッド実装
2. `delete_with_constraints()` メソッド骨格実装
3. `count_students()` メソッド実装

### **Phase 2: 制約ロジック実装**
1. 受講者存在チェックロジック
2. 制約違反エラーハンドリング
3. 適切なエラーメッセージ生成

### **Phase 3: RBAC統合実装**
1. `create_with_rbac()` メソッド実装
2. 権限チェック処理統合
3. 権限不足エラーハンドリング

## 📈 成功条件

### **コンパイル成功**
- 全ての未実装メソッドが最小実装される
- コンパイルエラーが解消される

### **テスト失敗→成功**
- `test_受講者存在時企業削除制約違反エラー()` が期待通りのエラーで成功
- `test_非管理者権限による企業作成拒否()` が権限エラーで成功
- `test_受講者なし企業の正常削除()` が正常削除で成功

### **ビジネスロジック動作**
- 制約チェックが正しく機能する
- RBAC権限制御が正しく機能する
- エラーメッセージが適切に生成される

---

**次フェーズ**: Greenフェーズ（最小実装）  
**推定実装時間**: 2-3時間（基本機能 + 制約ロジック + RBAC統合）  
**Redフェーズ品質**: ✅ 高品質（確実な失敗テスト、明確な実装要求）