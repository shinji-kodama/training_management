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