use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App,
    models::companies::ActiveModel,
};

#[tokio::test]
#[serial]
async fn test_企業情報の正常作成() {
    // 【テスト目的】: 企業エンティティの基本的な作成機能の動作確認
    // 【テスト内容】: 正常な企業データでの作成処理とデータベース保存
    // 【期待される動作】: 有効な企業データが正常にデータベースに保存される
    // 🟢 信頼性レベル: database-schema.sqlとtestcases.mdの定義に基づく確実なテストケース

    // 【テスト前準備】: 各テスト実行前にテスト環境を初期化し、一貫したテスト条件を保証
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 実際の企業登録で使用される標準的な企業情報
    // 【初期条件設定】: 企業テーブルの制約とインデックスが正常に設定済み
    let company_data = ActiveModel {
        name: ActiveValue::set("テスト株式会社".to_string()),
        contact_person: ActiveValue::set("田中太郎".to_string()),
        contact_email: ActiveValue::set("tanaka@test.co.jp".to_string()),
        chat_link: ActiveValue::set(Some("https://chat.test.co.jp".to_string())),
        ..Default::default()
    };

    // 【実際の処理実行】: Company::create()メソッドによる企業データ作成
    // 【処理内容】: ActiveModelを使用したSeaORM経由でのデータベース保存
    // 【実行タイミング】: トランザクション内での企業レコード作成実行
    let result = company_data.insert(&boot.app_context.db).await;

    // 【結果検証】: 作成された企業データの各フィールド値とタイムスタンプ確認
    // 【期待値確認】: UUID主キー生成、created_at/updated_at自動設定の検証
    // 【品質保証】: データベース制約とビジネスルールの整合性確認
    assert!(result.is_ok(), "企業作成が失敗しました: {:?}", result.err()); // 【確認内容】: 企業作成処理が正常完了することを確認 🟢

    let company = result.unwrap();
    assert_eq!(company.name, "テスト株式会社"); // 【確認内容】: 企業名が正確に保存されることを確認 🟢
    assert_eq!(company.contact_person, "田中太郎"); // 【確認内容】: 担当者名が正確に保存されることを確認 🟢
    assert_eq!(company.contact_email, "tanaka@test.co.jp"); // 【確認内容】: 連絡先メールが正確に保存されることを確認 🟢
    assert_eq!(company.chat_link, Some("https://chat.test.co.jp".to_string())); // 【確認内容】: チャットリンクが正確に保存されることを確認 🟢
    assert!(company.id != uuid::Uuid::nil()); // 【確認内容】: UUID主キーが自動生成されることを確認 🟢
    assert!(!company.created_at.to_string().is_empty()); // 【確認内容】: created_atが自動設定されることを確認 🟢
    assert!(!company.updated_at.to_string().is_empty()); // 【確認内容】: updated_atが自動設定されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_企業情報のメールアドレス形式バリデーション() {
    // 【テスト目的】: 入力データの品質管理機能確認
    // 【テスト内容】: 不正なメールアドレス形式での企業作成の失敗確認
    // 【期待される動作】: バリデーションエラーが発生し、データベース保存が拒否される
    // 🟢 信頼性レベル: 既存users.rsのemailバリデーション実装に基づく

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 不正なメールアドレス形式を含む企業データ
    // 【初期条件設定】: メールアドレス形式チェックが有効な状態
    let invalid_company_data = ActiveModel {
        name: ActiveValue::set("テスト株式会社".to_string()),
        contact_person: ActiveValue::set("田中太郎".to_string()),
        contact_email: ActiveValue::set("invalid-email-format".to_string()), // 不正な形式
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };

    // 【実際の処理実行】: 不正データでの企業作成試行
    // 【処理内容】: バリデーション付きActiveModel保存処理
    let result = invalid_company_data.insert(&boot.app_context.db).await;

    // 【結果検証】: バリデーションエラーが適切に発生することを確認
    // 【期待値確認】: ModelError::ValidationErrorが返されることを確認
    assert!(result.is_err(), "不正なメールアドレスでの企業作成が成功してしまいました"); // 【確認内容】: バリデーションエラーが発生することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_企業名最大長境界値() {
    // 【テスト目的】: データベーススキーマ制約の確認
    // 【テスト内容】: VARCHAR(255)制約の境界での動作確認
    // 【期待される動作】: 255文字で成功、256文字で失敗
    // 🟢 信頼性レベル: database-schema.sqlのVARCHAR制約に基づく

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 255文字ちょうどの企業名
    // 【初期条件設定】: VARCHAR(255)制約が有効な状態
    let name_255_chars = "a".repeat(255);
    let company_data_255 = ActiveModel {
        name: ActiveValue::set(name_255_chars),
        contact_person: ActiveValue::set("田中太郎".to_string()),
        contact_email: ActiveValue::set("tanaka@test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };

    // 【実際の処理実行】: 255文字の企業名での作成試行
    let result_255 = company_data_255.insert(&boot.app_context.db).await;

    // 【結果検証】: 255文字では正常に保存されることを確認
    assert!(result_255.is_ok(), "255文字の企業名での作成が失敗しました"); // 【確認内容】: 255文字の境界値で正常保存されることを確認 🟢

    // 【テストデータ準備】: 256文字の企業名（制限超過）
    let name_256_chars = "a".repeat(256);
    let company_data_256 = ActiveModel {
        name: ActiveValue::set(name_256_chars),
        contact_person: ActiveValue::set("田中太郎".to_string()),
        contact_email: ActiveValue::set("tanaka2@test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };

    // 【実際の処理実行】: 256文字の企業名での作成試行
    let result_256 = company_data_256.insert(&boot.app_context.db).await;

    // 【結果検証】: 256文字では制約違反エラーが発生することを確認
    assert!(result_256.is_err(), "256文字の企業名での作成が成功してしまいました"); // 【確認内容】: 256文字で制約違反エラーが発生することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_受講者存在時企業削除制約違反エラー() {
    // 【テスト目的】: 企業削除制約ビジネスロジックの確認
    // 【テスト内容】: 受講者が存在する企業の削除試行
    // 【期待される動作】: 制約違反エラー発生と削除処理失敗
    // 🔴 信頼性レベル: 未実装メソッド呼び出しのため失敗予定（Redフェーズ対象）

    // 【テスト前準備】: 各テスト実行前にテスト環境を初期化し、一貫したテスト条件を保証
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 削除制約テスト用の企業データ作成
    // 【外部キー準備】: 受講者テーブルのcompany_id外部キー制約を満たすため
    let company_data = ActiveModel {
        name: ActiveValue::set("制約テスト株式会社".to_string()),
        contact_person: ActiveValue::set("田中太郎".to_string()),
        contact_email: ActiveValue::set("tanaka@constraint-test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【受講者事前作成】: 企業削除を制約する受講者データ作成
    // 【制約条件設定】: company_idで企業と受講者を関連付け、削除制約を発生させる
    let student_data = training_management::models::students::ActiveModel {
        name: ActiveValue::set("受講者太郎".to_string()),
        email: ActiveValue::set("student@constraint-test.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("開発部".to_string()),
        ..Default::default()
    };
    student_data.insert(&boot.app_context.db).await
        .expect("Failed to create test student");

    // 【実際の処理実行】: 受講者が存在する企業の削除試行
    // 【処理内容】: delete_with_constraints()メソッドによる制約チェック付き削除
    // 【実行タイミング】: 関連データ存在時の削除制約ビジネスロジック実行
    // 🔴 未実装メソッド: このメソッドは存在しないため現在は失敗する
    let result = training_management::models::companies::Model::delete_with_constraints(
        &boot.app_context.db, 
        company.id
    ).await;

    // 【結果検証】: 制約違反エラーが適切に発生することを確認
    // 【期待値確認】: ビジネスルールに基づく削除拒否エラーが返されることを確認
    // 【品質保証】: データ整合性保護とビジネスルール遵守の保証
    assert!(result.is_err(), "受講者存在企業の削除が成功してしまいました"); // 【確認内容】: 企業削除制約違反エラーが発生することを確認 🔴

    // 【エラー内容検証】: エラーメッセージの具体的内容確認
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("受講者が存在する"), "期待されるエラーメッセージが含まれていません: {}", error_msg); // 【確認内容】: 適切な制約違反エラーメッセージが返されることを確認 🔴

    // 【データ整合性確認】: 削除が失敗した後も企業データが残存していることを確認
    // 【整合性保証】: 制約違反時にデータが保護されることの確認
    let company_still_exists = training_management::models::companies::Model::find_by_id(&boot.app_context.db, company.id).await;
    assert!(company_still_exists.is_ok() && company_still_exists.unwrap().is_some(), 
           "制約違反後に企業データが残存していません"); // 【確認内容】: 削除失敗後の企業データ残存確認 🔴
}

#[tokio::test]
#[serial]
async fn test_非管理者権限による企業作成拒否() {
    // 【テスト目的】: RBAC権限制御の確認
    // 【テスト内容】: Trainer/Instructor権限での企業作成試行
    // 【期待される動作】: 権限不足エラー発生と操作拒否
    // 🔴 信頼性レベル: RBAC統合機能未実装のため失敗予定（Redフェーズ対象）

    // 【テスト前準備】: 各テスト実行前にテスト環境を初期化し、一貫したテスト条件を保証
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【認証コンテキスト準備】: Trainer権限のユーザー認証情報作成
    // 【権限設定】: Admin権限が必要な操作をより低い権限で試行する条件設定
    let auth_context = training_management::models::rbac::AuthContext {
        user_id: 1,
        user_role: training_management::models::rbac::UserRole::Trainer,
        session_id: "test-session-trainer".to_string(),
    };

    // 【テストデータ準備】: 正常な企業データ（権限チェックに焦点）
    // 【初期条件設定】: データ自体は有効だが権限が不足している条件
    let company_data = ActiveModel {
        name: ActiveValue::set("権限テスト株式会社".to_string()),
        contact_person: ActiveValue::set("佐藤花子".to_string()),
        contact_email: ActiveValue::set("sato@rbac-test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };

    // 【実際の処理実行】: Trainer権限での企業作成試行
    // 【処理内容】: create_with_rbac()メソッドによる権限チェック付き作成
    // 【実行タイミング】: RBAC権限制御が働く企業作成処理実行
    // 🔴 未実装メソッド: このメソッドは存在しないため現在は失敗する
    let result = training_management::models::companies::Model::create_with_rbac(
        &boot.app_context.db,
        &auth_context,
        company_data
    ).await;

    // 【結果検証】: 権限不足エラーが適切に発生することを確認
    // 【期待値確認】: RBAC制御による操作拒否エラーが返されることを確認
    // 【品質保証】: セキュリティ要件の充足とアクセス制御の正確性
    assert!(result.is_err(), "Trainer権限での企業作成が成功してしまいました"); // 【確認内容】: RBAC権限不足エラーが発生することを確認 🔴

    // 【エラー内容検証】: 権限不足を示すエラーメッセージの確認
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Admin権限が必要") || error_msg.contains("権限が不足"), 
           "期待される権限エラーメッセージが含まれていません: {}", error_msg); // 【確認内容】: 適切な権限不足エラーメッセージが返されることを確認 🔴
}

#[tokio::test]
#[serial]
async fn test_受講者なし企業の正常削除() {
    // 【テスト目的】: 制約なし企業の削除機能確認
    // 【テスト内容】: 受講者が存在しない企業の削除処理
    // 【期待される動作】: データベースから企業レコードの安全な削除
    // 🔴 信頼性レベル: delete_with_constraints()メソッド未実装のため失敗予定

    // 【テスト前準備】: 各テスト実行前にテスト環境を初期化し、一貫したテスト条件を保証
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 削除対象となる企業データ作成
    // 【制約なし条件】: 受講者・プロジェクト・研修が紐付いていない独立企業
    let company_data = ActiveModel {
        name: ActiveValue::set("削除対象株式会社".to_string()),
        contact_person: ActiveValue::set("山田次郎".to_string()),
        contact_email: ActiveValue::set("yamada@delete-test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company for deletion");

    // 【実際の処理実行】: 制約なし企業の削除試行
    // 【処理内容】: delete_with_constraints()メソッドによる制約チェック付き削除
    // 【実行タイミング】: 関連データ非存在時の正常削除処理実行
    // 🔴 未実装メソッド: このメソッドは存在しないため現在は失敗する
    let result = training_management::models::companies::Model::delete_with_constraints(
        &boot.app_context.db,
        company.id
    ).await;

    // 【結果検証】: 削除処理が正常完了することを確認
    // 【期待値確認】: 制約なし企業の削除が成功することを確認
    // 【品質保証】: データ削除機能の正常動作確認
    assert!(result.is_ok(), "制約なし企業の削除が失敗しました: {:?}", result.err()); // 【確認内容】: 制約なし企業削除処理が正常完了することを確認 🔴

    // 【削除確認検証】: 削除後に企業データが存在しないことを確認
    // 【完全削除保証】: データベースから企業レコードが確実に削除されたことを確認
    let company_not_exists = training_management::models::companies::Model::find_by_id(&boot.app_context.db, company.id).await;
    assert!(company_not_exists.is_ok() && company_not_exists.unwrap().is_none(),
           "削除後に企業データが残存しています"); // 【確認内容】: 削除後の企業データ完全削除確認 🔴
}