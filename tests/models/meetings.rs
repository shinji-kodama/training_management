use loco_rs::testing::request::boot_test;
use sea_orm::ActiveModelTrait;
use serial_test::serial;
use training_management::app::App;
use chrono::Datelike;

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
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(1)),
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
    // 定例会時刻が設定されている（未来の日時）
    assert!(result.scheduled_at > chrono::Utc::now().fixed_offset());
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
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(1)),
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

#[tokio::test]
#[serial]
async fn test_隔週繰り返し定例会設定機能() {
    // 【テスト目的】: 隔週繰り返しスケジュール設定と日付計算の動作確認
    // 【テスト内容】: biweekly繰り返し設定での2週間間隔確認と長期スケジュール表示
    // 【期待される動作】: 正確な2週間間隔計算、長期スケジュール表示機能
    // 🟡 信頼性レベル: 隔週計算ロジックは新規実装のため推測含む
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 隔週定例会テスト用の基本エンティティ構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("隔週テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("隔週担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("biweekly@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "隔週テスト管理者".to_string(),
        email: "biweekly.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("隔週テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("隔週定例会用研修".to_string()),
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
        title: sea_orm::ActiveValue::Set("隔週テストプロジェクト".to_string()),
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
    
    // 【隔週定例会作成】: 金曜日開始の隔週定例会（6ヶ月間）
    let start_friday = chrono::NaiveDateTime::parse_from_str("2024-12-06 15:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_utc()
        .fixed_offset();
    
    let biweekly_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("隔週進捗報告会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(start_friday),
        recurrence_type: sea_orm::ActiveValue::Set("biweekly".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 6).unwrap())),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("隔週定例会テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let result = biweekly_meeting.insert(&boot.app_context.db).await.unwrap();
    
    // 【基本設定検証】: 隔週設定が正常に保存される
    assert_eq!(result.recurrence_type, "biweekly");
    assert_eq!(result.recurrence_end_date, Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 6).unwrap()));
    assert_eq!(result.scheduled_at.weekday(), chrono::Weekday::Fri);
    
    // 【スケジュール計算機能テスト】: 次回開催日計算機能（新規実装必要）
    // このテストは現在失敗するはず - Red Phase
    let next_occurrence = training_management::models::meetings::Model::calculate_next_occurrence(
        &result.scheduled_at,
        &result.recurrence_type,
        &result.recurrence_end_date
    ).await;
    
    // 【期待される結果】: 2週間後の同曜日（2024-12-20 金曜日）
    assert!(next_occurrence.is_ok());
    let next_date = next_occurrence.unwrap().unwrap();
    assert_eq!(next_date.weekday(), chrono::Weekday::Fri);
    assert_eq!(
        (next_date.date_naive() - start_friday.date_naive()).num_days(),
        14 // 2週間 = 14日
    );
}

#[tokio::test]
#[serial]
async fn test_研修講師任意参加設定機能() {
    // 【テスト目的】: instructor_idによる任意参加者設定と権限制御の動作確認
    // 【テスト内容】: 研修講師の任意参加設定とOptional外部キー設定
    // 【期待される動作】: Optional外部キー設定、instructor権限確認
    // 🟢 信頼性レベル: 既存instructor_id Optional設計を完全活用
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 任意参加テスト用の基本エンティティ構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("講師参加テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("講師担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("instructor@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "講師テスト管理者".to_string(),
        email: "instructor.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    // 【instructor権限ユーザー作成】: 任意参加可能なinstructorユーザー
    let instructor = training_management::models::users::RegisterParams {
        name: "任意参加講師".to_string(),
        email: "optional.instructor@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_instructor = training_management::models::users::Model::create_with_password(&boot.app_context.db, &instructor)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("講師参加テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("任意参加テスト用研修".to_string()),
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
        title: sea_orm::ActiveValue::Set("講師参加テストプロジェクト".to_string()),
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
    
    // 【任意参加設定定例会作成】: instructor_idがSomeで設定された定例会
    let meeting_with_instructor = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("講師任意参加定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(1)),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(Some(created_instructor.id)), // 任意参加instructor設定
        notes: sea_orm::ActiveValue::Set(Some("講師任意参加テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let result = meeting_with_instructor.insert(&boot.app_context.db).await.unwrap();
    
    // 【Optional外部キー確認】: instructor_idがSomeで正常に保存される
    assert_eq!(result.instructor_id, Some(created_instructor.id));
    
    // 【任意参加なし定例会作成】: instructor_idがNoneで設定された定例会
    let meeting_without_instructor = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("講師不参加定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(2)),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None), // 任意参加なし
        notes: sea_orm::ActiveValue::Set(Some("講師不参加テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let result2 = meeting_without_instructor.insert(&boot.app_context.db).await.unwrap();
    
    // 【Optional外部キー確認】: instructor_idがNoneで正常に保存される
    assert!(result2.instructor_id.is_none());
    
    // 【参加状況確認機能テスト】: instructor参加状況の確認（新規実装必要）
    // このテストは現在失敗するはず - Red Phase
    let participation_status = training_management::models::meetings::Model::check_instructor_participation(
        &boot.app_context.db,
        project.id,
        created_instructor.id
    ).await;
    
    // 【期待される結果】: 該当プロジェクトでのinstructor参加状況
    assert!(participation_status.is_ok());
    let status = participation_status.unwrap();
    assert_eq!(status.total_meetings, 2);
    assert_eq!(status.participating_meetings, 1);
}

#[tokio::test]
#[serial]
async fn test_markdown記録保存機能() {
    // 【テスト目的】: 定例会記録のMarkdown形式保存とXSS防御機能動作確認
    // 【テスト内容】: 長文記録のMarkdown形式での適切な保存とセキュリティ機能
    // 【期待される動作】: 文字制限内での記録保存、XSS防御、Markdown形式保持
    // 🟢 信頼性レベル: 既存TEXT制限とセキュリティパターン活用
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: Markdown記録テスト用の基本エンティティ構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("記録テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("記録担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("notes@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "記録テスト管理者".to_string(),
        email: "notes.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("記録テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("Markdown記録テスト用研修".to_string()),
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
        title: sea_orm::ActiveValue::Set("記録テストプロジェクト".to_string()),
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
    
    // 【Markdown記録データ作成】: 実際の定例会で記録される標準的なMarkdown内容
    let markdown_notes = "# 定例会記録\n\n## 進捗報告\n- 機能A: 70%完了\n- 機能B: 開始予定\n\n## 課題\n- リソース調整が必要";
    
    let meeting_with_notes = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("記録付き定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(1)),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some(markdown_notes.to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let result = meeting_with_notes.insert(&boot.app_context.db).await.unwrap();
    
    // 【基本記録保存確認】: Markdown内容が正確に保存される
    assert!(result.notes.is_some());
    let saved_notes = result.notes.unwrap();
    assert!(saved_notes.contains("# 定例会記録"));
    assert!(saved_notes.contains("## 進捗報告"));
    assert!(saved_notes.contains("- 機能A: 70%完了"));
    
    // 【XSSサニタイゼーション機能テスト】: 危険なスクリプトを含む記録の処理（新規実装必要）
    // このテストは現在失敗するはず - Red Phase
    let dangerous_notes = "# 会議記録\n<script>alert('XSS')</script>\n## 内容\n<img src=x onerror=alert(1)>";
    
    let sanitized_notes = training_management::models::meetings::Model::sanitize_markdown_notes(dangerous_notes).await;
    
    // 【期待される結果】: XSSスクリプトが除去されてMarkdown構造のみ保持
    assert!(sanitized_notes.is_ok());
    let clean_notes = sanitized_notes.unwrap();
    assert!(clean_notes.contains("# 会議記録"));
    assert!(!clean_notes.contains("<script>"));
    assert!(!clean_notes.contains("onerror"));
    
    // 【文字数制限機能テスト】: 制限を超える長文記録の処理（新規実装必要）
    let long_notes = "A".repeat(10001); // 制限超過（仮に10000文字制限）
    
    let validation_result = training_management::models::meetings::Model::validate_notes_length(&long_notes).await;
    
    // 【期待される結果】: 文字数制限超過エラー
    assert!(validation_result.is_err());
    let error_msg = validation_result.unwrap_err().to_string();
    assert!(error_msg.contains("文字") || error_msg.contains("制限"));
}

#[tokio::test]
#[serial]
async fn test_過去日時指定エラー() {
    // 【テスト目的】: 過去日時での定例会作成拒否機能の動作確認
    // 【テスト内容】: 現在時刻より前の日時での定例会予約試行によるエラー処理
    // 【期待される動作】: 論理的整合性確保、実用性保持、適切なエラーメッセージ
    // 🟢 信頼性レベル: 時刻比較ロジックは標準的なパターンを活用
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 過去日時テスト用の基本エンティティ構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("日時テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("日時担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("datetime@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "日時テスト管理者".to_string(),
        email: "datetime.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("日時テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("過去日時テスト用研修".to_string()),
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
        title: sea_orm::ActiveValue::Set("日時テストプロジェクト".to_string()),
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
    
    // 【過去日時定例会作成】: 明らかに過去の日時で定例会作成を試行
    let past_datetime = chrono::NaiveDateTime::parse_from_str("2023-01-01 10:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_utc()
        .fixed_offset();
    
    let past_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("過去日時定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(past_datetime),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("過去日時テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    // 【バリデーションエラー確認】: 過去日時での作成がバリデーションで拒否される
    let result = past_meeting.insert(&boot.app_context.db).await;
    assert!(result.is_err());
    
    // 【エラー内容検証】: エラーメッセージが適切な日本語で表示される
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("過去") || error_message.contains("日時") || error_message.contains("未来"));
    
    // 【境界値テスト】: 現在時刻ちょうどと1秒後の動作確認
    let now = chrono::Utc::now().fixed_offset();
    let future_1_sec = now + chrono::Duration::seconds(1);
    
    // 現在時刻ちょうどでの作成（新規バリデーション実装必要）
    // このテストは現在失敗するはず - Red Phase
    let now_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("現在時刻定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(now),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("現在時刻テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let now_result = now_meeting.insert(&boot.app_context.db).await;
    // 現在時刻での作成は仕様により成功またはエラー
    // 実装時に仕様確定が必要
    
    // 1秒後での作成（正常ケース）
    let future_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("未来日時定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(future_1_sec),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("未来日時テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let future_result = future_meeting.insert(&boot.app_context.db).await.unwrap();
    assert_eq!(future_result.title, "未来日時定例会");
}

#[tokio::test]
#[serial]
async fn test_繰り返し終了日境界値() {
    // 【テスト目的】: 繰り返し終了日の境界値での制約チェック機能動作確認
    // 【テスト内容】: 開始日との相対的な境界値での制約確認と論理的整合性
    // 【期待される動作】: 日付境界での論理的整合性チェック、不正な日付関係の拒否
    // 🟡 信頼性レベル: 同日終了の扱いは仕様確認が必要
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 日付境界値テスト用の基本エンティティ構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("境界値テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("境界担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("boundary@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "境界値テスト管理者".to_string(),
        email: "boundary.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("境界値テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("日付境界値テスト用研修".to_string()),
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
        title: sea_orm::ActiveValue::Set("境界値テストプロジェクト".to_string()),
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
    
    // 【基準日設定】: 2024年12月15日を開始日として使用
    let start_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
    let start_datetime = chrono::NaiveDateTime::new(start_date, chrono::NaiveTime::from_hms_opt(14, 0, 0).unwrap())
        .and_utc()
        .fixed_offset();
    
    // 【終了日が開始日より前の場合テスト】: 論理的不整合
    let end_date_before = chrono::NaiveDate::from_ymd_opt(2024, 12, 14).unwrap(); // 開始日より1日前
    
    let meeting_before = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("終了日が過去定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(start_datetime),
        recurrence_type: sea_orm::ActiveValue::Set("weekly".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(Some(end_date_before)), // 論理的不整合
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("終了日が過去テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let result_before = meeting_before.insert(&boot.app_context.db).await;
    assert!(result_before.is_err());
    let error_message = result_before.unwrap_err().to_string();
    assert!(error_message.contains("終了") || error_message.contains("開始") || error_message.contains("日付"));
    
    // 【終了日が開始日と同日の場合テスト】: 仕様により成功またはエラー（新規バリデーション実装必要）
    // このテストは現在失敗するはず - Red Phase
    let end_date_same = chrono::NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(); // 開始日と同日
    
    let meeting_same = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("終了日が同日定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(start_datetime),
        recurrence_type: sea_orm::ActiveValue::Set("weekly".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(Some(end_date_same)),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("終了日が同日テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let same_day_validation = training_management::models::meetings::Model::validate_recurrence_dates(
        &start_datetime.date_naive(),
        &end_date_same
    ).await;
    
    // 仕様によって成功またはエラー - 実装時に確定
    
    // 【終了日が開始日より後の場合テスト】: 正常ケース
    let end_date_after = chrono::NaiveDate::from_ymd_opt(2024, 12, 16).unwrap(); // 開始日より1日後
    
    let meeting_after = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("終了日が未来定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(start_datetime),
        recurrence_type: sea_orm::ActiveValue::Set("weekly".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(Some(end_date_after)), // 正常日付関係
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("終了日が未来テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let result_after = meeting_after.insert(&boot.app_context.db).await.unwrap();
    assert_eq!(result_after.recurrence_type, "weekly");
    assert_eq!(result_after.recurrence_end_date, Some(end_date_after));
}

#[tokio::test]
#[serial]
async fn test_同時刻重複定例会エラー() {
    // 【テスト目的】: スケジュール競合チェック機能の動作確認
    // 【テスト内容】: 同一時刻での複数定例会・面談重複回避機能
    // 【期待される動作】: リソース競合防止、スケジュール整合性、代替案提示
    // 🟡 信頼性レベル: 競合チェック機能は新規実装のため推測含む
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 重複チェックテスト用の基本エンティティ構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("重複チェック会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("重複担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("conflict@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "重複チェック管理者".to_string(),
        email: "conflict.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("重複チェック研修".to_string()),
        description: sea_orm::ActiveValue::Set("スケジュール重複チェック用研修".to_string()),
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
        title: sea_orm::ActiveValue::Set("重複チェックプロジェクト".to_string()),
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
    
    // 【基準時刻設定】: 2024年12月15日14:00を同時刻テストの基準として使用
    let target_datetime = chrono::NaiveDateTime::parse_from_str("2024-12-15 14:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_utc()
        .fixed_offset();
    
    // 【最初の定例会作成】: 特定時刻に最初の定例会を作成
    let first_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("最初の定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(target_datetime),
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("最初の定例会テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    let first_result = first_meeting.insert(&boot.app_context.db).await.unwrap();
    assert_eq!(first_result.scheduled_at, target_datetime);
    
    // 【同時刻重複定例会作成】: 既存定例会と同一時刻での作成試行
    let conflicting_meeting = training_management::models::meetings::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        title: sea_orm::ActiveValue::Set("重複定例会".to_string()),
        scheduled_at: sea_orm::ActiveValue::Set(target_datetime), // 同一時刻
        recurrence_type: sea_orm::ActiveValue::Set("none".to_string()),
        recurrence_end_date: sea_orm::ActiveValue::Set(None),
        instructor_id: sea_orm::ActiveValue::Set(None),
        notes: sea_orm::ActiveValue::Set(Some("重複定例会テスト".to_string())),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    };
    
    // 【重複チェック機能テスト】: 同時刻重複検出機能（新規実装必要）
    // このテストは現在失敗するはず - Red Phase
    let conflict_check = training_management::models::meetings::Model::check_schedule_conflicts(
        &boot.app_context.db,
        &target_datetime,
        project.id,
        None // 新規作成のためmeeting_idはNone
    ).await;
    
    // 【期待される結果】: 競合が検出されエラーが返される
    assert!(conflict_check.is_ok());
    let conflicts = conflict_check.unwrap();
    assert!(conflicts.has_conflicts);
    assert_eq!(conflicts.conflicting_meetings.len(), 1);
    assert_eq!(conflicts.conflicting_meetings[0].id, first_result.id);
    
    // 【代替時刻提案機能テスト】: 空いている時間帯の提案（新規実装必要）
    let suggested_times = training_management::models::meetings::Model::suggest_alternative_times(
        &boot.app_context.db,
        &target_datetime,
        project.id,
        3 // 3個の代替時刻を提案
    ).await;
    
    // 【期待される結果】: 利用可能な代替時刻が提案される
    assert!(suggested_times.is_ok());
    let suggestions = suggested_times.unwrap();
    assert_eq!(suggestions.len(), 3);
    for suggestion in suggestions {
        assert_ne!(suggestion, target_datetime); // 競合時刻とは異なる
    }
}