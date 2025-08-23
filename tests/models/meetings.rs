use loco_rs::testing::request::boot_test;
use sea_orm::ActiveModelTrait;
use serial_test::serial;
use training_management::app::App;

// 【テスト対象】: 定例会（Meetings）モデルの包括的CRUD機能テスト
// 【テスト方針】: database-schema.sqlの制約とビジネスルールに基づく確実なテストケース
// 【フレームワーク】: Loco.rs 0.16.3 + SeaORM 1.1.12 + PostgreSQL環境でのモデルテスト
// 🟢 信頼性レベル: database-schema.sqlのmeetingsテーブル定義と制約に完全準拠

#[tokio::test]
#[serial]
async fn test_定例会の正常作成() {
    // 【テスト目的】: 定例会エンティティの基本的な作成処理とデータベース保存の動作確認
    // 【テスト内容】: 有効な定例会データが正常にデータベースに保存され、UUID主キーとタイムスタンプが自動設定される
    // 【期待される動作】: 外部キー関係（project_id, created_by, instructor_id）が正常に機能し、繰り返し設定制約がクリアされる
    // 🟢 信頼性レベル: database-schema.sqlのmeetingsテーブル定義に基づく確実なテストケース
    
    // 【テスト前準備】: データベース接続とテスト環境の初期化
    // 【初期条件設定】: 定例会作成に必要な外部キーデータ（企業、ユーザー、研修、プロジェクト）を事前に準備
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: 定例会作成に必要な外部エンティティを事前に作成
    // 【データ整合性】: 外部キー制約を満たすため、companies, users, trainings, projects テーブルにデータを準備
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("定例会テスト株式会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("定例会担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("meeting@test.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(Some("https://chat.meeting.co.jp".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【プロジェクト作成者ユーザー作成】: プロジェクト作成用のユーザー
    let project_manager = training_management::models::users::RegisterParams {
        name: "プロジェクト管理者".to_string(),
        email: "manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    // 【定例会講師ユーザー作成】: 定例会を実施するinstructorユーザーを作成（instructor_id外部キー用）
    let instructor = training_management::models::users::RegisterParams {
        name: "定例会講師".to_string(),
        email: "instructor@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_instructor = training_management::models::users::Model::create_with_password(&boot.app_context.db, &instructor)
        .await
        .unwrap();
    
    // 【研修コース作成】: プロジェクト作成に必要な研修コース
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("定例会用研修コース".to_string()),
        description: sea_orm::ActiveValue::Set("定例会テスト用の研修".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("基礎知識".to_string()),
        goals: sea_orm::ActiveValue::Set("スキル向上".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("課題完了".to_string()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【プロジェクト作成】: 定例会が関連するプロジェクト
    let project = training_management::models::projects::ActiveModel {
        title: sea_orm::ActiveValue::Set("定例会対象プロジェクト".to_string()),
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【定例会データ作成】: 正常な定例会データを準備（繰り返し設定なし）
    let meeting_data = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("週次定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None), // noneの場合はNULL可
        instructor_id: sea_orm::ActiveValue::Set(Some(created_instructor.id)),
        notes: sea_orm::ActiveValue::Set(Some("# 定例会記録\\n\\n## 今回のアジェンダ\\n- 進捗報告\\n- 課題共有".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    // 【定例会作成実行】: データベースへ定例会データを保存
    let result = meeting_data.insert(&boot.app_context.db).await.unwrap();
    
    // 【結果検証】: 作成された定例会データの妥当性確認
    // UUID主キー自動生成の確認（空文字列ではないUUIDが設定される）
    assert!(!result.id.to_string().is_empty());
    
    // 外部キー関係の正常保存確認
    assert_eq!(result.project_id, project.id);
    assert_eq!(result.created_by, created_manager.id);
    assert_eq!(result.instructor_id, Some(created_instructor.id));
    
    // 繰り返し設定の正常動作確認（noneは有効な値）
    assert_eq!(result.recurrence_type, "none");
    assert!(result.recurrence_end_date.is_none());
    
    // Markdownノート保存の確認
    assert!(result.notes.is_some());
    assert!(result.notes.unwrap().contains("定例会記録"));
    
    // タイトル保存の確認
    assert_eq!(result.title, "週次定例会");
    
    // タイムスタンプ自動設定確認
    // created_at と updated_at は chrono::DateTime<FixedOffset> 型で常に値を持つ
    
    // 【ビジネスロジック検証】: 定例会データがビジネス要件を満たしているか確認
    // 定例会時刻が設定されている
    assert!(result.scheduled_at > chrono::Utc::now().fixed_offset() - chrono::Duration::seconds(10));
}

#[tokio::test]
#[serial]
async fn test_プロジェクト別定例会一覧取得() {
    // 【テスト目的】: プロジェクトに紐付く定例会一覧の検索機能動作確認
    // 【テスト内容】: 特定のプロジェクトに関連する定例会を正確に抽出できる
    // 【期待される動作】: 1対多リレーション（プロジェクト→定例会）が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlの外部キー関係に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 複数定例会を持つプロジェクトのテストデータセット構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("定例会検索テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("検索担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("search@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "検索テスト管理者".to_string(),
        email: "search.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let instructor = training_management::models::users::RegisterParams {
        name: "検索テスト講師".to_string(),
        email: "search.instructor@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_instructor = training_management::models::users::Model::create_with_password(&boot.app_context.db, &instructor)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("検索テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("定例会検索用研修".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("基礎知識".to_string()),
        goals: sea_orm::ActiveValue::Set("スキル向上".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("課題完了".to_string()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project = training_management::models::projects::ActiveModel {
        title: sea_orm::ActiveValue::Set("検索テストプロジェクト".to_string()),
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【複数定例会作成】: 同一プロジェクトに対する複数の定例会を作成
    // 1回目の定例会（通常の定例会）
    let meeting1 = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("第1回定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::days(1)),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(Some(created_instructor.id)),
        notes: sea_orm::ActiveValue::Set(Some("1回目定例会".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 2回目の定例会（毎週繰り返し）
    let meeting2 = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("週次定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::days(7)),
        recurrence_type: sea_orm::ActiveValue::Set("weekly".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())),
        instructor_id: sea_orm::ActiveValue::Set(Some(created_instructor.id)),
        notes: sea_orm::ActiveValue::Set(Some("週次定例会".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【検索機能テスト実行】: プロジェクト別定例会一覧取得機能をテスト
    let meetings = training_management::models::meetings::Model::find_by_project_id(&boot.app_context.db, project.id).await.unwrap();
    
    // 【検索結果検証】: 検索結果の妥当性確認
    // 正しい数の定例会が取得される（2件）
    assert_eq!(meetings.len(), 2);
    
    // すべての定例会が正しいプロジェクトに紐付いている
    for meeting in &meetings {
        assert_eq!(meeting.project_id, project.id);
    }
    
    // 作成した定例会IDが含まれている
    let meeting_ids: Vec<uuid::Uuid> = meetings.iter().map(|m| m.id).collect();
    assert!(meeting_ids.contains(&meeting1.id));
    assert!(meeting_ids.contains(&meeting2.id));
}

#[tokio::test]
#[serial]
async fn test_繰り返し設定制約バリデーション() {
    // 【テスト目的】: 定例会の繰り返し設定制約チェック機能動作確認
    // 【テスト内容】: 繰り返し設定が'weekly'または'biweekly'の場合、終了日が必須となる制約の動作確認
    // 【期待される動作】: CHECK制約による繰り返し設定と終了日の整合性チェックが正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlのCHECK制約定義に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 定例会作成に必要な基本エンティティの構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("制約テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("制約担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("constraint@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "制約テスト管理者".to_string(),
        email: "constraint.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("制約テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("繰り返し制約テスト用研修".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("基礎知識".to_string()),
        goals: sea_orm::ActiveValue::Set("スキル向上".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("課題完了".to_string()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project = training_management::models::projects::ActiveModel {
        title: sea_orm::ActiveValue::Set("制約テストプロジェクト".to_string()),
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【制約違反テスト】: 繰り返し設定が'weekly'で終了日がNULLの定例会作成
    let invalid_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("制約違反定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        recurrence_type: sea_orm::ActiveValue::Set("weekly".to_string()), // 繰り返し設定あり
        recurrence_end_date: sea_orm::ActiveValue::Set(None), // しかし終了日がNULL（制約違反）
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("制約違反テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    // 【制約違反確認】: CHECK制約によりデータベースエラーが発生することを確認
    let result = invalid_meeting.insert(&boot.app_context.db).await;
    assert!(result.is_err());
    
    // 【エラー内容検証】: エラーメッセージがCHECK制約違反を示している
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("check") || error_message.contains("constraint") || error_message.contains("recurrence"));
    
    // 【正常設定確認】: 有効な繰り返し設定では正常に作成される
    let valid_weekly_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("正常週次定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(1)),
        recurrence_type: sea_orm::ActiveValue::Set("weekly".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())), // 終了日設定
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("正常週次定例会".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let weekly_result = valid_weekly_meeting.insert(&boot.app_context.db).await.unwrap();
    assert_eq!(weekly_result.recurrence_type, "weekly");
    assert!(weekly_result.recurrence_end_date.is_some());
    
    // 【隔週設定テスト】: biweekly設定でも正常に作成される
    let valid_biweekly_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("正常隔週定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(2)),
        recurrence_type: sea_orm::ActiveValue::Set("biweekly".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("正常隔週定例会".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let biweekly_result = valid_biweekly_meeting.insert(&boot.app_context.db).await.unwrap();
    assert_eq!(biweekly_result.recurrence_type, "biweekly");
    assert!(biweekly_result.recurrence_end_date.is_some());
}

#[tokio::test]
#[serial]
async fn test_繰り返し種別制約バリデーション() {
    // 【テスト目的】: 定例会の繰り返し種別値の制約チェック機能動作確認
    // 【テスト内容】: 許可されていない繰り返し種別値での定例会作成が適切に拒否される
    // 【期待される動作】: CHECK制約（'none', 'weekly', 'biweekly'）が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlのCHECK制約定義に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 定例会作成に必要な基本エンティティの構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("種別テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("種別担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("type@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "種別テスト管理者".to_string(),
        email: "type.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("種別テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("繰り返し種別テスト用研修".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("基礎知識".to_string()),
        goals: sea_orm::ActiveValue::Set("スキル向上".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("課題完了".to_string()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project = training_management::models::projects::ActiveModel {
        title: sea_orm::ActiveValue::Set("種別テストプロジェクト".to_string()),
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【無効繰り返し種別テスト】: 許可されていない繰り返し種別値での定例会作成
    let invalid_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("無効種別定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        recurrence_type: sea_orm::ActiveValue::Set("invalid_recurrence".to_string()), // 無効な繰り返し種別
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("無効種別テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    // 【制約違反確認】: CHECK制約によりデータベースエラーが発生することを確認
    let result = invalid_meeting.insert(&boot.app_context.db).await;
    assert!(result.is_err());
    
    // 【エラー内容検証】: エラーメッセージがCHECK制約違反を示している
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("check") || error_message.contains("constraint") || error_message.contains("recurrence_type"));
    
    // 【正常繰り返し種別確認】: 有効な繰り返し種別値では正常に作成される
    let valid_types = vec!["none", "weekly", "biweekly"];
    for (i, recurrence_type) in valid_types.iter().enumerate() {
        let end_date = if *recurrence_type == "none" { 
            None 
        } else { 
            Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()) 
        };
        
        let valid_meeting = training_management::models::meetings::ActiveModel {
            project_id: sea_orm::ActiveValue::Set(project.id),
            title: sea_orm::ActiveValue::Set(format!("{}種別定例会", recurrence_type)),
            scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(i as i64 + 1)),
            recurrence_type: sea_orm::ActiveValue::Set(recurrence_type.to_string()),
            recurrence_end_date: sea_orm::ActiveValue::Set(end_date),
            instructor_id: sea_orm::ActiveValue::Set(None),
            notes: sea_orm::ActiveValue::Set(Some(format!("{}種別テスト", recurrence_type))),
            created_by: sea_orm::ActiveValue::Set(created_manager.id),
            ..Default::default()
        };
        
        let result = valid_meeting.insert(&boot.app_context.db).await.unwrap();
        assert_eq!(result.recurrence_type, *recurrence_type);
    }
}

#[tokio::test]
#[serial]
async fn test_プロジェクト参照整合性制約() {
    // 【テスト目的】: プロジェクト参照整合性制約の動作確認
    // 【テスト内容】: 存在しないproject_idでの定例会作成が適切に拒否される
    // 【期待される動作】: 外部キー制約が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlの外部キー制約に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【定例会作成者作成】: 有効な定例会作成者を作成
    let project_manager = training_management::models::users::RegisterParams {
        name: "整合性テスト管理者".to_string(),
        email: "integrity.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    // 【無効な外部キーテスト】: 存在しないproject_idでの定例会作成
    let nonexistent_uuid = uuid::Uuid::new_v4(); // 存在しないUUID
    
    let invalid_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(nonexistent_uuid),
        title: sea_orm::ActiveValue::Set("整合性制約テスト".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("整合性制約テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    // 【外部キー制約違反確認】: データベースエラーが発生することを確認
    let result = invalid_meeting.insert(&boot.app_context.db).await;
    assert!(result.is_err());
    
    // 【エラー内容検証】: エラーメッセージが外部キー制約違反を示している
    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("foreign key") || 
        error_message.contains("references") || 
        error_message.contains("project") ||
        error_message.contains("violates")
    );
}