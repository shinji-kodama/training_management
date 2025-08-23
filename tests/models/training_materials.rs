/**
 * 【テストファイル概要】: TrainingMaterialsモデル（研修教材紐付け）のTDD Redフェーズテストケース
 * 【テスト対象】: 研修教材紐付け情報の正常作成、研修別教材一覧取得、制約違反バリデーション
 * 【実装方針】: CompaniesとStudentsモデルのテストパターンを継承し、中間テーブル固有の要件を追加
 * 🟢 信頼性レベル: database-schema.sqlとTASK-004要件定義に基づく確実なテストケース
 */

use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App,
    models::training_materials::ActiveModel,
};

#[tokio::test]
#[serial]
async fn test_研修教材紐付け情報の正常作成() {
    // 【テスト目的】: 研修教材紐付けエンティティの基本的な作成機能の動作確認
    // 【テスト内容】: 正常な研修教材紐付けデータでの作成処理とデータベース保存
    // 【期待される動作】: 有効な研修教材紐付けデータが正常にデータベースに保存される
    // 🟢 信頼性レベル: database-schema.sqlとtask-004要件に基づく確実なテストケース

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 研修コース作成に必要な関連企業データを準備
    // 【外部キー準備】: 研修テーブルのcompany_id外部キー制約を満たすため
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("テスト研修企業".to_string()),
        contact_person: ActiveValue::set("研修担当者".to_string()),
        contact_email: ActiveValue::set("training@testcompany.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【研修コース事前作成】: 研修教材紐付けに必要な関連研修コースデータを準備
    // 【外部キー準備】: training_materials.training_id外部キー制約を満たすため
    let training_data = training_management::models::trainings::ActiveModel {
        title: ActiveValue::set("Rust入門研修".to_string()),
        description: ActiveValue::set("Rust言語の基礎から実践的な開発手法まで学ぶ包括的な研修コース".to_string()),
        prerequisites: ActiveValue::set("プログラミング経験1年以上".to_string()),
        goals: ActiveValue::set("Rust言語でのWebアプリケーション開発ができるようになる".to_string()),
        completion_criteria: ActiveValue::set("最終課題のWebアプリケーションを完成させる".to_string()),
        company_id: ActiveValue::set(Some(company.id)),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let training = training_data.insert(&boot.app_context.db).await
        .expect("Failed to create test training");

    // 【教材事前作成】: 研修教材紐付けに必要な関連教材データを準備
    // 【外部キー準備】: training_materials.material_id外部キー制約を満たすため
    let material_data = training_management::models::materials::ActiveModel {
        title: ActiveValue::set("Rust基礎文法".to_string()),
        url: ActiveValue::set("https://doc.rust-lang.org/book/".to_string()),
        domain: ActiveValue::set("doc.rust-lang.org".to_string()),
        description: ActiveValue::set("Rustプログラミング言語の公式ドキュメント".to_string()),
        recommendation_level: ActiveValue::set(5),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let material = material_data.insert(&boot.app_context.db).await
        .expect("Failed to create test material");

    // 【テストデータ準備】: 実際の研修教材紐付けで使用される標準的な紐付け情報
    // 【必須フィールド設定】: データベーススキーマの必須フィールドをすべて設定
    let training_material_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material.id),
        period_days: ActiveValue::set(7), // 1週間の学習期間
        order_index: ActiveValue::set(1), // 研修コース内での最初の教材
        ..Default::default()
    };

    // 【実際の処理実行】: TrainingMaterial::create()メソッドによる研修教材紐付けデータ作成
    // 【処理内容】: SeaORM ActiveModelのinsert()メソッドでデータベースに保存
    let result = training_material_data.insert(&boot.app_context.db).await;

    // 【結果検証】: 作成された研修教材紐付けデータの各フィールド値とタイムスタンプ確認
    // 【期待値確認】: 入力した各フィールドが正確に保存され、自動生成フィールドが設定されることを確認
    assert!(result.is_ok(), "研修教材紐付け作成が失敗しました: {:?}", result.err());

    let training_material = result.unwrap();
    assert_eq!(training_material.training_id, training.id); // 【確認内容】: 研修IDが正確に保存されている 🟢
    assert_eq!(training_material.material_id, material.id); // 【確認内容】: 教材IDが正確に保存されている 🟢
    assert_eq!(training_material.period_days, 7); // 【確認内容】: 学習期間（日数）が正確に保存されている 🟢
    assert_eq!(training_material.order_index, 1); // 【確認内容】: 教材順序が正確に保存されている 🟢
    assert!(training_material.id != uuid::Uuid::nil()); // 【確認内容】: UUID主キーが自動生成されている 🟢
    assert!(!training_material.created_at.to_string().is_empty()); // 【確認内容】: 作成日時が自動設定されている 🟢
}

#[tokio::test]
#[serial]
async fn test_研修別教材一覧取得() {
    // 【テスト目的】: 研修コースと教材間の多対多リレーション機能の動作確認
    // 【テスト内容】: 特定研修に紐づく教材一覧の取得機能とリレーション動作の確認
    // 【期待される動作】: 研修IDで教材紐付け情報を検索し、正しい教材リストが順序付きで取得される
    // 🟢 信頼性レベル: database-schema.sqlの外部キー関係定義に基づく確実なテストケース

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業データ準備】: リレーション検索のための企業データを事前作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("研修実施企業B".to_string()),
        contact_person: ActiveValue::set("企業担当者".to_string()),
        contact_email: ActiveValue::set("contact@companyb.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【研修コースデータ準備】: 教材紐付けのための研修コースを作成
    let training_data = training_management::models::trainings::ActiveModel {
        title: ActiveValue::set("フルスタック開発研修".to_string()),
        description: ActiveValue::set("フロントエンドからバックエンドまでの総合的な研修".to_string()),
        prerequisites: ActiveValue::set("HTML/CSS/JavaScript基礎知識".to_string()),
        goals: ActiveValue::set("フルスタックWebアプリケーションを開発できるようになる".to_string()),
        completion_criteria: ActiveValue::set("最終プロジェクトの完成と発表".to_string()),
        company_id: ActiveValue::set(Some(company.id)),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let training = training_data.insert(&boot.app_context.db).await
        .expect("Failed to create test training");

    // 【教材データ準備】: 研修に紐づく複数の教材を作成
    // 【順序テストデータ】: 異なる順序インデックスで複数教材を作成し、順序付き取得をテスト
    let material1_data = training_management::models::materials::ActiveModel {
        title: ActiveValue::set("HTML基礎".to_string()),
        url: ActiveValue::set("https://developer.mozilla.org/ja/docs/Web/HTML".to_string()),
        domain: ActiveValue::set("developer.mozilla.org".to_string()),
        description: ActiveValue::set("HTML要素とセマンティクスの基礎".to_string()),
        recommendation_level: ActiveValue::set(4),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let material1 = material1_data.insert(&boot.app_context.db).await
        .expect("Failed to create material1");

    let material2_data = training_management::models::materials::ActiveModel {
        title: ActiveValue::set("JavaScript基礎".to_string()),
        url: ActiveValue::set("https://javascript.info/".to_string()),
        domain: ActiveValue::set("javascript.info".to_string()),
        description: ActiveValue::set("モダンJavaScriptの基礎から応用まで".to_string()),
        recommendation_level: ActiveValue::set(5),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let material2 = material2_data.insert(&boot.app_context.db).await
        .expect("Failed to create material2");

    let material3_data = training_management::models::materials::ActiveModel {
        title: ActiveValue::set("React入門".to_string()),
        url: ActiveValue::set("https://ja.reactjs.org/tutorial/tutorial.html".to_string()),
        domain: ActiveValue::set("ja.reactjs.org".to_string()),
        description: ActiveValue::set("Reactコンポーネントとステート管理の基礎".to_string()),
        recommendation_level: ActiveValue::set(5),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let material3 = material3_data.insert(&boot.app_context.db).await
        .expect("Failed to create material3");

    // 【研修教材紐付けデータ準備】: 研修に複数教材を順序付きで紐付け
    // 【順序検証データ】: 意図的に順序を変えて作成し、order_index順でのソートをテスト
    let training_material1_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material1.id),
        period_days: ActiveValue::set(3), // HTML基礎：3日
        order_index: ActiveValue::set(1), // 最初の教材
        ..Default::default()
    };
    training_material1_data.insert(&boot.app_context.db).await
        .expect("Failed to create training_material1");

    let training_material2_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material3.id),
        period_days: ActiveValue::set(10), // React入門：10日
        order_index: ActiveValue::set(3), // 3番目の教材
        ..Default::default()
    };
    training_material2_data.insert(&boot.app_context.db).await
        .expect("Failed to create training_material2");

    let training_material3_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material2.id),
        period_days: ActiveValue::set(7), // JavaScript基礎：7日
        order_index: ActiveValue::set(2), // 2番目の教材
        ..Default::default()
    };
    training_material3_data.insert(&boot.app_context.db).await
        .expect("Failed to create training_material3");

    // 【実際の検索処理実行】: TrainingMaterial::find_by_training_id()メソッドによる研修別検索
    // 【リレーション機能確認】: 研修IDを条件とした教材紐付け一覧の順序付き取得の動作確認
    let training_materials_result = training_management::models::training_materials::Model::find_by_training_id(&boot.app_context.db, training.id).await;

    // 【結果検証】: 検索結果の件数、内容、順序の確認
    // 【期待値確認】: 作成した3件の研修教材紐付けが正しい順序で検索されることを確認
    assert!(training_materials_result.is_ok(), "研修別教材一覧取得が失敗しました: {:?}", training_materials_result.err());
    
    let training_materials = training_materials_result.unwrap();
    assert_eq!(training_materials.len(), 3); // 【確認内容】: 作成した3件の教材紐付けが検索されている 🟢
    
    // 【教材順序確認】: order_index順に教材が取得されていることを確認
    assert_eq!(training_materials[0].order_index, 1); // 【確認内容】: 最初がHTML基礎（order_index=1） 🟢
    assert_eq!(training_materials[1].order_index, 2); // 【確認内容】: 2番目がJavaScript基礎（order_index=2） 🟢
    assert_eq!(training_materials[2].order_index, 3); // 【確認内容】: 3番目がReact入門（order_index=3） 🟢
    
    // 【教材ID確認】: 正しい教材が順序通りに取得されていることを確認
    assert_eq!(training_materials[0].material_id, material1.id); // 【確認内容】: HTML基礎が最初の順序 🟢
    assert_eq!(training_materials[1].material_id, material2.id); // 【確認内容】: JavaScript基礎が2番目の順序 🟢
    assert_eq!(training_materials[2].material_id, material3.id); // 【確認内容】: React入門が3番目の順序 🟢

    // 【学習期間確認】: 各教材の学習期間が正しく設定されていることを確認
    assert_eq!(training_materials[0].period_days, 3); // 【確認内容】: HTML基礎の学習期間が3日 🟢
    assert_eq!(training_materials[1].period_days, 7); // 【確認内容】: JavaScript基礎の学習期間が7日 🟢
    assert_eq!(training_materials[2].period_days, 10); // 【確認内容】: React入門の学習期間が10日 🟢
}

#[tokio::test]
#[serial]
async fn test_制約違反バリデーション() {
    // 【テスト目的】: 研修教材紐付け作成時のユニーク制約バリデーション機能の動作確認
    // 【テスト内容】: 同一研修での教材重複と順序重複のバリデーションエラーが発生することを確認
    // 【期待される動作】: ユニーク制約違反の場合に適切なバリデーションエラーが発生する
    // 🟢 信頼性レベル: database-schema.sqlのUNIQUE制約に基づく確実なテストケース

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【基本データ準備】: 制約テストのための基本エンティティを作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("制約テスト企業".to_string()),
        contact_person: ActiveValue::set("テスト担当者".to_string()),
        contact_email: ActiveValue::set("test@constraint.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    let training_data = training_management::models::trainings::ActiveModel {
        title: ActiveValue::set("制約テスト研修".to_string()),
        description: ActiveValue::set("制約テスト用研修".to_string()),
        prerequisites: ActiveValue::set("なし".to_string()),
        goals: ActiveValue::set("制約テスト".to_string()),
        completion_criteria: ActiveValue::set("テスト完了".to_string()),
        company_id: ActiveValue::set(Some(company.id)),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let training = training_data.insert(&boot.app_context.db).await
        .expect("Failed to create test training");

    let material_data = training_management::models::materials::ActiveModel {
        title: ActiveValue::set("制約テスト教材".to_string()),
        url: ActiveValue::set("https://test.example.com/".to_string()),
        domain: ActiveValue::set("test.example.com".to_string()),
        description: ActiveValue::set("制約テスト用のダミー教材".to_string()),
        recommendation_level: ActiveValue::set(3),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let material = material_data.insert(&boot.app_context.db).await
        .expect("Failed to create test material");

    // 【正常な紐付け作成】: 制約テストのベースとなる正常な研修教材紐付けを作成
    let valid_training_material_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material.id),
        period_days: ActiveValue::set(5),
        order_index: ActiveValue::set(1),
        ..Default::default()
    };
    let _valid_result = valid_training_material_data.insert(&boot.app_context.db).await
        .expect("Failed to create valid training material");

    // 【重複教材制約テストデータ準備】: 同一研修・同一教材の重複紐付けを試行
    // 【エラー発生条件設定】: UNIQUE(training_id, material_id)制約違反を誘発
    let duplicate_material_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material.id), // 【制約違反】: 同一教材を再度紐付け
        period_days: ActiveValue::set(3),
        order_index: ActiveValue::set(2), // 順序は異なるが教材が重複
        ..Default::default()
    };

    // 【実際の制約違反処理実行】: 重複教材での紐付け作成試行
    // 【エラー発生確認】: UNIQUE制約違反でデータベースエラーが発生することを確認
    let duplicate_result = duplicate_material_data.insert(&boot.app_context.db).await;

    // 【制約違反結果検証】: 重複教材バリデーションエラーが適切に発生することを確認
    // 【期待値確認】: 同一研修で同一教材の重複紐付けが失敗することを確認
    assert!(duplicate_result.is_err(), "同一研修での教材重複が許可されてしまいました"); // 【確認内容】: UNIQUE(training_id, material_id)制約が機能している 🟢
    
    // 【エラー内容確認】: データベース制約エラーの内容が適切であることを確認
    let _error = duplicate_result.err().unwrap();
    // 【制約エラーメッセージ確認】: ユニーク制約違反のエラーメッセージが含まれていることを確認
    // 🟡 推測レベル: 具体的なエラーメッセージはSeaORM/PostgreSQL実装依存のため、エラー発生の確認に留める
}

#[tokio::test]
#[serial]
async fn test_順序制約バリデーション() {
    // 【テスト目的】: 研修内での教材順序重複制約バリデーション機能の動作確認
    // 【テスト内容】: 同一研修での教材順序重複のバリデーションエラーが発生することを確認
    // 【期待される動作】: 順序重複制約違反の場合に適切なバリデーションエラーが発生する
    // 🟢 信頼性レベル: database-schema.sqlのUNIQUE(training_id, order_index)制約に基づく確実なテストケース

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【基本データ準備】: 順序制約テストのための基本エンティティを作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("順序テスト企業".to_string()),
        contact_person: ActiveValue::set("順序担当者".to_string()),
        contact_email: ActiveValue::set("order@test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    let training_data = training_management::models::trainings::ActiveModel {
        title: ActiveValue::set("順序テスト研修".to_string()),
        description: ActiveValue::set("順序制約テスト用研修".to_string()),
        prerequisites: ActiveValue::set("なし".to_string()),
        goals: ActiveValue::set("順序制約テスト".to_string()),
        completion_criteria: ActiveValue::set("テスト完了".to_string()),
        company_id: ActiveValue::set(Some(company.id)),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let training = training_data.insert(&boot.app_context.db).await
        .expect("Failed to create test training");

    // 【異なる教材データ準備】: 順序制約テスト用に複数の異なる教材を作成
    let material1_data = training_management::models::materials::ActiveModel {
        title: ActiveValue::set("順序テスト教材1".to_string()),
        url: ActiveValue::set("https://test1.example.com/".to_string()),
        domain: ActiveValue::set("test1.example.com".to_string()),
        description: ActiveValue::set("順序制約テスト用教材1".to_string()),
        recommendation_level: ActiveValue::set(3),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let material1 = material1_data.insert(&boot.app_context.db).await
        .expect("Failed to create test material1");

    let material2_data = training_management::models::materials::ActiveModel {
        title: ActiveValue::set("順序テスト教材2".to_string()),
        url: ActiveValue::set("https://test2.example.com/".to_string()),
        domain: ActiveValue::set("test2.example.com".to_string()),
        description: ActiveValue::set("順序制約テスト用教材2".to_string()),
        recommendation_level: ActiveValue::set(4),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let material2 = material2_data.insert(&boot.app_context.db).await
        .expect("Failed to create test material2");

    // 【正常な順序付き紐付け作成】: 制約テストのベースとなる正常な順序付き紐付けを作成
    let valid_order_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material1.id),
        period_days: ActiveValue::set(5),
        order_index: ActiveValue::set(1), // 順序1として設定
        ..Default::default()
    };
    let _valid_result = valid_order_data.insert(&boot.app_context.db).await
        .expect("Failed to create valid order training material");

    // 【順序重複制約テストデータ準備】: 同一研修・同一順序の重複紐付けを試行
    // 【エラー発生条件設定】: UNIQUE(training_id, order_index)制約違反を誘発
    let duplicate_order_data = ActiveModel {
        training_id: ActiveValue::set(training.id),
        material_id: ActiveValue::set(material2.id), // 教材は異なるが順序が重複
        period_days: ActiveValue::set(3),
        order_index: ActiveValue::set(1), // 【制約違反】: 同一順序を再度設定
        ..Default::default()
    };

    // 【実際の順序制約違反処理実行】: 重複順序での紐付け作成試行
    // 【エラー発生確認】: UNIQUE(training_id, order_index)制約違反でデータベースエラーが発生することを確認
    let duplicate_order_result = duplicate_order_data.insert(&boot.app_context.db).await;

    // 【順序制約違反結果検証】: 重複順序バリデーションエラーが適切に発生することを確認
    // 【期待値確認】: 同一研修で同一順序の重複紐付けが失敗することを確認
    assert!(duplicate_order_result.is_err(), "同一研修での順序重複が許可されてしまいました"); // 【確認内容】: UNIQUE(training_id, order_index)制約が機能している 🟢
    
    // 【エラー内容確認】: データベース制約エラーの内容が適切であることを確認
    let _error = duplicate_order_result.err().unwrap();
    // 【順序制約エラーメッセージ確認】: 順序ユニーク制約違反のエラーメッセージが含まれていることを確認
    // 🟡 推測レベル: 具体的なエラーメッセージはSeaORM/PostgreSQL実装依存のため、エラー発生の確認に留める
}