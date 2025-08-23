use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App,
    models::materials::ActiveModel,
};

#[tokio::test]
#[serial]
async fn test_教材情報の正常作成() {
    // 【テスト目的】: Materialsエンティティの基本的な作成機能の動作確認
    // 【テスト内容】: 正常な教材データでの作成処理とデータベース保存
    // 【期待される動作】: 有効な教材データが正常にデータベースに保存される
    // 🟢 信頼性レベル: database-schema.sqlとtestcases.mdの定義に基づく確実なテストケース

    // 【テスト前準備】: 各テスト実行前にテスト環境を初期化し、一貫したテスト条件を保証
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 実際の教材登録で使用される標準的な教材情報
    // 【初期条件設定】: 教材テーブルの制約とインデックスが正常に設定済み
    let material_data = ActiveModel {
        title: ActiveValue::set("Rust基礎入門".to_string()),
        url: ActiveValue::set("https://example.com/rust-basics".to_string()),
        domain: ActiveValue::set("example.com".to_string()),
        description: ActiveValue::set("Rust言語の基礎的な文法と概念を学ぶコース".to_string()),
        recommendation_level: ActiveValue::set(4),
        created_by: ActiveValue::set(1), // 管理者ユーザーID
        ..Default::default()
    };

    // 【実際の処理実行】: Material::create()メソッドによる教材データ作成
    // 【処理内容】: ActiveModelを使用したSeaORM経由でのデータベース保存
    // 【実行タイミング】: トランザクション内での教材レコード作成実行
    let result = material_data.insert(&boot.app_context.db).await;

    // 【結果検証】: 作成された教材データの各フィールド値とタイムスタンプ確認
    // 【期待値確認】: UUID主キー生成、created_at/updated_at自動設定の検証
    // 【品質保証】: データベース制約とビジネスルールの整合性確認
    assert!(result.is_ok(), "教材作成が失敗しました: {:?}", result.err()); // 【確認内容】: 教材作成処理が正常完了することを確認 🟢

    let material = result.unwrap();
    assert_eq!(material.title, "Rust基礎入門"); // 【確認内容】: 教材タイトルが正確に保存されることを確認 🟢
    assert_eq!(material.url, "https://example.com/rust-basics"); // 【確認内容】: 教材URLが正確に保存されることを確認 🟢
    assert_eq!(material.description, "Rust言語の基礎的な文法と概念を学ぶコース"); // 【確認内容】: 教材説明が正確に保存されることを確認 🟢
    assert_eq!(material.recommendation_level, 4); // 【確認内容】: 推奨レベルが正確に保存されることを確認 🟢
    assert_eq!(material.domain, "example.com"); // 【確認内容】: ドメインが正確に保存されることを確認 🟢
    assert!(material.id != uuid::Uuid::nil()); // 【確認内容】: UUID主キーが自動生成されることを確認 🟢
    assert!(!material.created_at.to_string().is_empty()); // 【確認内容】: created_atが自動設定されることを確認 🟢
    assert!(!material.updated_at.to_string().is_empty()); // 【確認内容】: updated_atが自動設定されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_教材のURL形式バリデーション() {
    // 【テスト目的】: URL形式チェック機能の動作確認
    // 【テスト内容】: 不正なURL形式での教材作成の失敗確認
    // 【期待される動作】: バリデーションエラーが発生し、データベース保存が拒否される
    // 🟢 信頼性レベル: バリデーション実装パターンに基づく

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 不正なURL形式を含む教材データ
    // 【初期条件設定】: URL形式チェックが有効な状態
    let invalid_material_data = ActiveModel {
        title: ActiveValue::set("無効URL教材".to_string()),
        url: ActiveValue::set("invalid-url-format".to_string()), // 不正な形式
        domain: ActiveValue::set("invalid-domain".to_string()),
        description: ActiveValue::set("不正なURLを持つ教材".to_string()),
        recommendation_level: ActiveValue::set(3),
        created_by: ActiveValue::set(1), // 管理者ユーザーID
        ..Default::default()
    };

    // 【実際の処理実行】: 不正データでの教材作成試行
    // 【処理内容】: バリデーション付きActiveModel保存処理
    let result = invalid_material_data.insert(&boot.app_context.db).await;

    // 【結果検証】: バリデーションエラーが適切に発生することを確認
    // 【期待値確認】: ModelError::ValidationErrorが返されることを確認
    assert!(result.is_err(), "不正なURLでの教材作成が成功してしまいました"); // 【確認内容】: バリデーションエラーが発生することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_推奨レベル境界値() {
    // 【テスト目的】: 推奨レベル(1-5)のCHECK制約境界値確認
    // 【テスト内容】: 有効範囲（1,5）では成功、無効範囲（0,6）では失敗
    // 【期待される動作】: 1,5で成功、0,6でCHECK制約違反エラー
    // 🟢 信頼性レベル: database-schema.sqlのCHECK制約定義に基づく

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 推奨レベル1（最小有効値）
    // 【初期条件設定】: CHECK制約が有効な状態
    let material_level_1 = ActiveModel {
        title: ActiveValue::set("レベル1教材".to_string()),
        url: ActiveValue::set("https://example.com/level1".to_string()),
        domain: ActiveValue::set("example.com".to_string()),
        description: ActiveValue::set("推奨レベル1の教材".to_string()),
        recommendation_level: ActiveValue::set(1),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };

    // 【実際の処理実行】: 推奨レベル1での作成試行
    let result_1 = material_level_1.insert(&boot.app_context.db).await;

    // 【結果検証】: 推奨レベル1では正常に保存されることを確認
    assert!(result_1.is_ok(), "推奨レベル1での作成が失敗しました"); // 【確認内容】: 推奨レベル1で正常保存されることを確認 🟢

    // 【テストデータ準備】: 推奨レベル5（最大有効値）
    let material_level_5 = ActiveModel {
        title: ActiveValue::set("レベル5教材".to_string()),
        url: ActiveValue::set("https://example.com/level5".to_string()),
        domain: ActiveValue::set("example.com".to_string()),
        description: ActiveValue::set("推奨レベル5の教材".to_string()),
        recommendation_level: ActiveValue::set(5),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };

    // 【実際の処理実行】: 推奨レベル5での作成試行
    let result_5 = material_level_5.insert(&boot.app_context.db).await;

    // 【結果検証】: 推奨レベル5では正常に保存されることを確認
    assert!(result_5.is_ok(), "推奨レベル5での作成が失敗しました"); // 【確認内容】: 推奨レベル5で正常保存されることを確認 🟢

    // 【テストデータ準備】: 推奨レベル0（無効値）
    let material_level_0 = ActiveModel {
        title: ActiveValue::set("レベル0教材".to_string()),
        url: ActiveValue::set("https://example.com/level0".to_string()),
        domain: ActiveValue::set("example.com".to_string()),
        description: ActiveValue::set("推奨レベル0の教材（無効）".to_string()),
        recommendation_level: ActiveValue::set(0),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };

    // 【実際の処理実行】: 推奨レベル0での作成試行
    let result_0 = material_level_0.insert(&boot.app_context.db).await;

    // 【結果検証】: 推奨レベル0ではCHECK制約違反エラーが発生することを確認
    assert!(result_0.is_err(), "推奨レベル0での作成が成功してしまいました"); // 【確認内容】: 推奨レベル0でCHECK制約違反エラーが発生することを確認 🟢

    // 【テストデータ準備】: 推奨レベル6（無効値）
    let material_level_6 = ActiveModel {
        title: ActiveValue::set("レベル6教材".to_string()),
        url: ActiveValue::set("https://example.com/level6".to_string()),
        domain: ActiveValue::set("example.com".to_string()),
        description: ActiveValue::set("推奨レベル6の教材（無効）".to_string()),
        recommendation_level: ActiveValue::set(6),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };

    // 【実際の処理実行】: 推奨レベル6での作成試行
    let result_6 = material_level_6.insert(&boot.app_context.db).await;

    // 【結果検証】: 推奨レベル6ではCHECK制約違反エラーが発生することを確認
    assert!(result_6.is_err(), "推奨レベル6での作成が成功してしまいました"); // 【確認内容】: 推奨レベル6でCHECK制約違反エラーが発生することを確認 🟢
}