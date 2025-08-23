/**
 * 【テストファイル概要】: Trainingsモデル（研修コース）のTDD Redフェーズテストケース
 * 【テスト対象】: 研修コース情報の正常作成、企業別研修コース検索、必須フィールドバリデーション
 * 【実装方針】: CompaniesとStudentsモデルのテストパターンを継承し、研修コース固有の要件を追加
 * 🟢 信頼性レベル: database-schema.sqlとTASK-004要件定義に基づく確実なテストケース
 */

use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App,
    models::trainings::ActiveModel,
};

#[tokio::test]
#[serial]
async fn test_研修コース情報の正常作成() {
    // 【テスト目的】: 研修コースエンティティの基本的な作成機能の動作確認
    // 【テスト内容】: 正常な研修コースデータでの作成処理とデータベース保存
    // 【期待される動作】: 有効な研修コースデータが正常にデータベースに保存される
    // 🟢 信頼性レベル: database-schema.sqlとtask-004要件に基づく確実なテストケース

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 研修コース作成に必要な関連企業データを準備
    // 【外部キー準備】: 研修コーステーブルのcompany_id外部キー制約を満たすため
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("テスト研修企業".to_string()),
        contact_person: ActiveValue::set("研修担当者".to_string()),
        contact_email: ActiveValue::set("training@testcompany.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【テストデータ準備】: 実際の研修コース登録で使用される標準的な研修コース情報
    // 【必須フィールド設定】: データベーススキーマの必須フィールドをすべて設定
    let training_data = ActiveModel {
        title: ActiveValue::set("Rust入門研修".to_string()),
        description: ActiveValue::set("Rust言語の基礎から実践的な開発手法まで学ぶ包括的な研修コース".to_string()),
        prerequisites: ActiveValue::set("プログラミング経験1年以上、基本的なコンピュータサイエンスの知識".to_string()),
        goals: ActiveValue::set("Rust言語でのWebアプリケーション開発ができるようになる".to_string()),
        completion_criteria: ActiveValue::set("最終課題のWebアプリケーションを完成させる".to_string()),
        company_id: ActiveValue::set(Some(company.id)),
        created_by: ActiveValue::set(1), // 管理者ユーザーID
        ..Default::default()
    };

    // 【実際の処理実行】: Training::create()メソッドによる研修コースデータ作成
    // 【処理内容】: SeaORM ActiveModelのinsert()メソッドでデータベースに保存
    let result = training_data.insert(&boot.app_context.db).await;

    // 【結果検証】: 作成された研修コースデータの各フィールド値とタイムスタンプ確認
    // 【期待値確認】: 入力した各フィールドが正確に保存され、自動生成フィールドが設定されることを確認
    assert!(result.is_ok(), "研修コース作成が失敗しました: {:?}", result.err());

    let training = result.unwrap();
    assert_eq!(training.title, "Rust入門研修"); // 【確認内容】: 研修コースタイトルが正確に保存されている 🟢
    assert_eq!(training.description, "Rust言語の基礎から実践的な開発手法まで学ぶ包括的な研修コース"); // 【確認内容】: 研修コース説明が正確に保存されている 🟢
    assert_eq!(training.prerequisites, "プログラミング経験1年以上、基本的なコンピュータサイエンスの知識"); // 【確認内容】: 受講前提条件が正確に保存されている 🟢
    assert_eq!(training.goals, "Rust言語でのWebアプリケーション開発ができるようになる"); // 【確認内容】: 研修ゴールが正確に保存されている 🟢
    assert_eq!(training.completion_criteria, "最終課題のWebアプリケーションを完成させる"); // 【確認内容】: 完了条件が正確に保存されている 🟢
    assert_eq!(training.company_id, Some(company.id)); // 【確認内容】: 企業IDの外部キー関係が正確に設定されている 🟢
    assert_eq!(training.created_by, 1); // 【確認内容】: 作成者IDが正確に保存されている 🟢
    assert!(training.id != uuid::Uuid::nil()); // 【確認内容】: UUID主キーが自動生成されている 🟢
    assert!(!training.created_at.to_string().is_empty()); // 【確認内容】: 作成日時が自動設定されている 🟢
    assert!(!training.updated_at.to_string().is_empty()); // 【確認内容】: 更新日時が自動設定されている 🟢
}

#[tokio::test]
#[serial]
async fn test_企業別研修コース検索() {
    // 【テスト目的】: 企業と研修コース間の1対多リレーション機能の動作確認
    // 【テスト内容】: 特定企業に紐づく研修コースの検索機能とリレーション動作の確認
    // 【期待される動作】: 企業IDで研修コースを検索し、正しい研修コースリストが取得される
    // 🟢 信頼性レベル: database-schema.sqlの外部キー関係定義に基づく確実なテストケース

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業データ準備】: リレーション検索のための企業データを事前作成
    // 【外部キー関係構築】: 研修コースが企業に正しく紐づけられることを確認するため
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("研修実施企業A".to_string()),
        contact_person: ActiveValue::set("企業担当者".to_string()),
        contact_email: ActiveValue::set("contact@companya.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【研修コースデータ準備】: 企業に紐づく研修コースを複数作成
    // 【リレーション検証データ】: 同一企業に複数の研修コースが紐づくケースを想定
    let training1_data = ActiveModel {
        title: ActiveValue::set("基礎研修".to_string()),
        description: ActiveValue::set("基礎的なスキルを身につける研修".to_string()),
        prerequisites: ActiveValue::set("特になし".to_string()),
        goals: ActiveValue::set("基礎スキルの習得".to_string()),
        completion_criteria: ActiveValue::set("基礎テストに合格する".to_string()),
        company_id: ActiveValue::set(Some(company.id)),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    
    let training2_data = ActiveModel {
        title: ActiveValue::set("応用研修".to_string()),
        description: ActiveValue::set("応用的なスキルを身につける研修".to_string()),
        prerequisites: ActiveValue::set("基礎研修修了".to_string()),
        goals: ActiveValue::set("応用スキルの習得".to_string()),
        completion_criteria: ActiveValue::set("応用課題を完成させる".to_string()),
        company_id: ActiveValue::set(Some(company.id)),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };

    // 【研修コース作成実行】: テスト用の研修コースをデータベースに保存
    training1_data.insert(&boot.app_context.db).await
        .expect("Failed to create training1");
    training2_data.insert(&boot.app_context.db).await
        .expect("Failed to create training2");

    // 【実際の検索処理実行】: Training::find_by_company_id()メソッドによる企業別検索
    // 【リレーション機能確認】: 企業IDを条件とした研修コース検索の動作確認
    let trainings_result = training_management::models::trainings::Model::find_by_company_id(&boot.app_context.db, company.id).await;

    // 【結果検証】: 検索結果の件数と内容の確認
    // 【期待値確認】: 作成した2件の研修コースが正しく検索されることを確認
    assert!(trainings_result.is_ok(), "企業別研修コース検索が失敗しました: {:?}", trainings_result.err());
    
    let trainings = trainings_result.unwrap();
    assert_eq!(trainings.len(), 2); // 【確認内容】: 作成した2件の研修コースが検索されている 🟢
    
    // 【研修コースタイトル確認】: 作成した研修コースのタイトルが正しく取得されている
    let titles: Vec<String> = trainings.iter().map(|t| t.title.clone()).collect();
    assert!(titles.contains(&"基礎研修".to_string())); // 【確認内容】: 基礎研修が検索結果に含まれている 🟢
    assert!(titles.contains(&"応用研修".to_string())); // 【確認内容】: 応用研修が検索結果に含まれている 🟢
}

#[tokio::test]
#[serial]
async fn test_必須フィールドバリデーション() {
    // 【テスト目的】: 研修コース作成時の必須フィールドバリデーション機能の動作確認
    // 【テスト内容】: 必須フィールドが空の場合にバリデーションエラーが発生することを確認
    // 【期待される動作】: 必須フィールドが未入力の場合に適切なバリデーションエラーが発生する
    // 🟢 信頼性レベル: database-schema.sqlのNOT NULL制約に基づく確実なテストケース

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【バリデーションテストデータ準備】: 必須フィールドの一部を空にした不正なデータ
    // 【エラー発生条件設定】: titleフィールドを空文字列にしてバリデーションエラーを誘発
    let invalid_training_data = ActiveModel {
        title: ActiveValue::set("".to_string()), // 【バリデーション対象】: 空のタイトル
        description: ActiveValue::set("テスト用の研修説明".to_string()),
        prerequisites: ActiveValue::set("テスト前提条件".to_string()),
        goals: ActiveValue::set("テストゴール".to_string()),
        completion_criteria: ActiveValue::set("テスト完了条件".to_string()),
        company_id: ActiveValue::set(None),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };

    // 【実際のバリデーション処理実行】: Training::create()メソッドによるバリデーション実行
    // 【エラー発生確認】: 不正なデータでの作成試行でバリデーションエラーが発生することを確認
    let result = invalid_training_data.insert(&boot.app_context.db).await;

    // 【結果検証】: バリデーションエラーが適切に発生することを確認
    // 【期待値確認】: 必須フィールドが空の場合に作成が失敗することを確認
    assert!(result.is_err(), "必須フィールドが空でも研修コース作成が成功してしまいました"); // 【確認内容】: 不正なデータでの作成が適切に失敗している 🟢
    
    // 【エラー内容確認】: バリデーションエラーの内容が適切であることを確認
    let _error = result.err().unwrap();
    // 【バリデーションメッセージ確認】: タイトルが空の場合のエラーメッセージが含まれていることを確認
    // 🟡 推測レベル: 具体的なエラーメッセージは実装依存のため、エラー発生の確認に留める
}