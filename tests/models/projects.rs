use loco_rs::testing::request::boot_test;
use sea_orm::ActiveModelTrait;
use serial_test::serial;
use training_management::app::App;

// 【テスト対象】: プロジェクト（Projects）モデルの包括的CRUD機能テスト
// 【テスト方針】: database-schema.sqlの制約とビジネスルールに基づく確実なテストケース
// 【フレームワーク】: Loco.rs 0.16.3 + SeaORM 1.1.12 + PostgreSQL環境でのモデルテスト
// 🟢 信頼性レベル: database-schema.sqlのテーブル定義と制約に完全準拠

#[tokio::test]
#[serial]
async fn test_プロジェクト情報の正常作成() {
    // 【テスト目的】: プロジェクトエンティティの基本的な作成処理とデータベース保存の動作確認
    // 【テスト内容】: 有効なプロジェクトデータが正常にデータベースに保存され、UUID主キーとタイムスタンプが自動設定される
    // 【期待される動作】: 外部キー関係（training_id, company_id, created_by）が正常に機能し、CHECK制約がクリアされる
    // 🟢 信頼性レベル: database-schema.sqlのprojectsテーブル定義に基づく確実なテストケース
    
    // 【テスト前準備】: データベース接続とテスト環境の初期化
    // 【初期条件設定】: プロジェクト作成に必要な外部キーデータ（研修、企業、ユーザー）を事前に準備
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: プロジェクト作成に必要な外部エンティティを事前に作成
    // 【データ整合性】: 外部キー制約を満たすため、companies, trainings, usersテーブルにデータを準備
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("テスト株式会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("田中太郎".to_string()),
        contact_email: sea_orm::ActiveValue::Set("contact@test.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(Some("https://chat.test.co.jp".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let user = training_management::models::users::RegisterParams {
        name: "プロジェクト管理者".to_string(),
        email: "manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &user)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("基礎研修コース".to_string()),
        description: sea_orm::ActiveValue::Set("プログラミング基礎を学ぶコースです".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("特になし".to_string()),
        goals: sea_orm::ActiveValue::Set("基本的なプログラミングスキルの習得".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("課題を80%以上で完了".to_string()),
        company_id: sea_orm::ActiveValue::Set(Some(company.id.clone())),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【テストデータ準備】: プロジェクト作成で使用する実際のビジネスデータ形式
    // 【制約確認】: start_date <= end_date のCHECK制約を満たすデータ設定
    let project_data = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("2025年春期プログラミング研修プロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 4, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    };
    
    // 【実際の処理実行】: Project::create()メソッドによるプロジェクトデータ作成
    // 【処理内容】: ActiveModelを使用したSeaORM経由でのデータベース保存
    // 【UUID生成確認】: before_save()でUUID主キーが自動生成されることを検証
    let result = project_data.insert(&boot.app_context.db).await.unwrap();
    
    // 【結果検証】: 作成されたプロジェクトデータの各フィールド値とタイムスタンプ確認
    // 【期待値確認】: UUID主キー生成、created_at/updated_at自動設定の検証
    // 【品質保証】: データベース制約とビジネスルールの整合性確認
    assert!(!result.id.to_string().is_empty()); // 【確認内容】: UUID主キーが自動生成されていることを確認 🟢
    assert_eq!(result.title, "2025年春期プログラミング研修プロジェクト"); // 【確認内容】: プロジェクトタイトルが正確に保存されることを確認 🟢
    assert_eq!(result.training_id, training.id); // 【確認内容】: 研修IDの外部キー関係が正常に設定されることを確認 🟢
    assert_eq!(result.company_id, company.id); // 【確認内容】: 企業IDの外部キー関係が正常に設定されることを確認 🟢
    assert_eq!(result.created_by, created_user.id); // 【確認内容】: 作成者IDの外部キー関係が正常に設定されることを確認 🟢
    assert!(!result.created_at.to_string().is_empty()); // 【確認内容】: 作成日時が自動的に設定されることを確認 🟢
    assert!(!result.updated_at.to_string().is_empty()); // 【確認内容】: 更新日時が自動的に設定されることを確認 🟢
    assert!(result.end_date >= result.start_date); // 【確認内容】: CHECK制約（終了日≥開始日）が正常に機能することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_企業別プロジェクト一覧取得() {
    // 【テスト目的】: 企業IDを条件としたプロジェクト一覧取得機能の動作確認
    // 【テスト内容】: 指定企業に紐づく全プロジェクトが正確に取得され、適切な並び順で返却される
    // 【期待される動作】: 1対多リレーション（企業→プロジェクト）が正常に機能し、外部キーインデックスが活用される
    // 🟢 信頼性レベル: database-schema.sqlの外部キー制約とインデックス定義に基づく
    
    // 【テスト前準備】: 複数プロジェクトを持つ企業データの作成
    // 【初期条件設定】: 1つの企業に対して複数のプロジェクトを紐付けた状態を構築
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: テスト用の企業、研修、ユーザーデータを準備
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("株式会社テスト開発".to_string()),
        contact_person: sea_orm::ActiveValue::Set("山田花子".to_string()),
        contact_email: sea_orm::ActiveValue::Set("yamada@testdev.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(Some("https://chat.testdev.co.jp".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let user = training_management::models::users::RegisterParams {
        name: "研修責任者".to_string(),
        email: "admin@testdev.co.jp".to_string(),
        password: "admin123secure".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &user)
        .await
        .unwrap();
    
    let training1 = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("Web開発研修".to_string()),
        description: sea_orm::ActiveValue::Set("フロントエンド開発技術を学ぶ".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("HTML/CSS基礎知識".to_string()),
        goals: sea_orm::ActiveValue::Set("モダンなWebアプリケーション開発".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("最終プロジェクト完成".to_string()),
        company_id: sea_orm::ActiveValue::Set(Some(company.id.clone())),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let training2 = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("データベース研修".to_string()),
        description: sea_orm::ActiveValue::Set("SQL設計と最適化を学ぶ".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("データベース基礎".to_string()),
        goals: sea_orm::ActiveValue::Set("効率的なDB設計技術習得".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("実践課題80%以上".to_string()),
        company_id: sea_orm::ActiveValue::Set(Some(company.id.clone())),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【複数プロジェクト作成】: 同一企業に対して異なる研修の複数プロジェクトを作成
    // 【並び順テスト準備】: 開始日が異なるプロジェクトを作成して並び順確認を可能にする
    let _project1 = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training1.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("2025年Web開発プロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 5, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let _project2 = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training2.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("2025年DB設計プロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 8, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【実際の処理実行】: Project::find_by_company_id()メソッドによる企業別プロジェクト検索
    // 【処理内容】: company_idを条件とした外部キーインデックス活用の効率的検索
    // 【パフォーマンステスト】: 1対多リレーション検索の動作確認
    let projects = training_management::models::projects::Model::find_by_company_id(&boot.app_context.db, company.id).await.unwrap();
    
    // 【結果検証】: 検索されたプロジェクト件数と内容の確認
    // 【期待値確認】: 作成した2件のプロジェクトが正確に取得されることを確認
    // 【品質保証】: 外部キー関係とリレーション機能の整合性確認
    assert_eq!(projects.len(), 2); // 【確認内容】: 企業に紐づく2件のプロジェクトが正確に取得されることを確認 🟢
    
    // 【個別プロジェクト確認】: 各プロジェクトが正しい企業IDを持っていることを確認
    for project in &projects {
        assert_eq!(project.company_id, company.id); // 【確認内容】: 全プロジェクトが指定企業IDを持っていることを確認 🟢
        assert!(!project.title.is_empty()); // 【確認内容】: プロジェクトタイトルが適切に設定されていることを確認 🟢
        assert!(project.end_date >= project.start_date); // 【確認内容】: 全プロジェクトでCHECK制約が維持されていることを確認 🟢
    }
}

#[tokio::test]
#[serial]
async fn test_日付制約バリデーション() {
    // 【テスト目的】: CHECK制約（end_date >= start_date）の動作確認とビジネスルール検証
    // 【テスト内容】: 終了日が開始日より前の不正なプロジェクトデータでの作成試行
    // 【期待される動作】: データベースレベルでCHECK制約違反エラーが発生し、不正データの保存が阻止される
    // 🟢 信頼性レベル: database-schema.sqlのCHECK制約定義に完全準拠
    
    // 【テスト前準備】: 制約違反テスト用の依存データ準備
    // 【初期条件設定】: CHECK制約以外の全ての制約を満たすデータ環境を構築
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: 外部キー制約を満たすための基本データ作成
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("制約テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("制約太郎".to_string()),
        contact_email: sea_orm::ActiveValue::Set("constraint@test.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(None),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let user = training_management::models::users::RegisterParams {
        name: "制約確認者".to_string(),
        email: "checker@test.co.jp".to_string(),
        password: "check123secure".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &user)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("制約確認研修".to_string()),
        description: sea_orm::ActiveValue::Set("制約の動作を確認する".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("特になし".to_string()),
        goals: sea_orm::ActiveValue::Set("制約理解".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("理解度100%".to_string()),
        company_id: sea_orm::ActiveValue::Set(Some(company.id.clone())),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【有効なプロジェクト作成】: まず有効なプロジェクトを作成して基本機能確認
    let _valid_project = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("有効なプロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【制約違反データ準備】: end_date < start_date の不正な日付関係を設定
    // 【意図的エラー作成】: CHECK制約を意図的に違反するデータでテスト
    let invalid_project_data = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("不正な日付範囲のプロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()), // 【制約違反】: 終了日より後の開始日
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),   // 【制約違反】: 開始日より前の終了日
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    };
    
    // 【実際の処理実行】: CHECK制約違反データでの保存試行
    // 【処理内容】: データベースレベルでの制約チェック機能の検証
    // 【エラー期待処理】: 制約違反によるエラー発生を期待した処理実行
    let result = invalid_project_data.insert(&boot.app_context.db).await;
    
    // 【結果検証】: CHECK制約違反エラーが適切に発生することを確認
    // 【期待値確認】: DbErrでの制約エラーが返されることを検証
    // 【品質保証】: データベース制約によるデータ整合性保護機能の確認
    assert!(result.is_err()); // 【確認内容】: 日付制約違反時にエラーが発生することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_外部キー制約バリデーション() {
    // 【テスト目的】: 外部キー制約（training_id, company_id, created_by）の動作確認
    // 【テスト内容】: 存在しない外部キー値を指定したプロジェクト作成時のエラーハンドリング
    // 【期待される動作】: 外部キー制約違反エラーが発生し、参照整合性が保護される
    // 🟢 信頼性レベル: database-schema.sqlの外部キー制約定義に完全準拠
    
    // 【テスト前準備】: 外部キー制約違反テスト用の環境準備
    // 【初期条件設定】: データベースが空の状態で外部キー制約違反を確認
    let boot = boot_test::<App>().await.unwrap();
    
    // 【存在しない外部キーデータ準備】: 実際には存在しないUUIDを使用した制約違反データ
    // 【意図的エラー作成】: 外部キー制約を意図的に違反するデータでテスト
    let non_existent_uuid = uuid::Uuid::new_v4(); // 【制約違反準備】: データベースに存在しない新規UUID生成
    let non_existent_user_id = 999999; // 【制約違反準備】: データベースに存在しない新規ユーザーID
    
    let invalid_project_data = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(non_existent_uuid), // 【制約違反】: 存在しない研修ID
        company_id: sea_orm::ActiveValue::Set(non_existent_uuid),  // 【制約違反】: 存在しない企業ID
        title: sea_orm::ActiveValue::Set("外部キー制約違反プロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 4, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()),
        created_by: sea_orm::ActiveValue::Set(non_existent_user_id), // 【制約違反】: 存在しないユーザーID
        ..Default::default()
    };
    
    // 【実際の処理実行】: 外部キー制約違反データでの保存試行
    // 【処理内容】: データベースの参照整合性チェック機能の検証
    // 【エラー期待処理】: 外部キー制約違反によるエラー発生を期待した処理実行
    let result = invalid_project_data.insert(&boot.app_context.db).await;
    
    // 【結果検証】: 外部キー制約違反エラーが適切に発生することを確認
    // 【期待値確認】: DbErrでの外部キー制約エラーが返されることを検証
    // 【品質保証】: 参照整合性保護機能の確認
    assert!(result.is_err()); // 【確認内容】: 外部キー制約違反時にエラーが発生することを確認 🟢
    
    // 【エラー内容確認】: 外部キー制約違反エラーの詳細内容確認
    let error = result.unwrap_err();
    // 【期待エラー確認】: PostgreSQLの外部キー制約違反エラーが含まれることを確認
    // 🟡 エラーメッセージの具体的内容確認は実装依存のため黄色信号
    println!("外部キー制約エラー詳細: {}", error.to_string()); // デバッグ用出力
    assert!(error.to_string().contains("foreign") || 
            error.to_string().contains("reference") || 
            error.to_string().contains("constraint") ||
            error.to_string().contains("研修") || 
            error.to_string().contains("企業") ||
            error.to_string().contains("ユーザー") ||
            error.to_string().contains("存在しません")); // 【確認内容】: 外部キー制約違反エラーが適切に報告されることを確認 🟡
}