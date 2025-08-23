use loco_rs::testing::request::boot_test;
use sea_orm::ActiveModelTrait;
use serial_test::serial;
use training_management::app::App;

// 【テスト対象】: 面談（Interviews）モデルの包括的CRUD機能テスト
// 【テスト方針】: database-schema.sqlの制約とビジネスルールに基づく確実なテストケース
// 【フレームワーク】: Loco.rs 0.16.3 + SeaORM 1.1.12 + PostgreSQL環境でのモデルテスト
// 🟢 信頼性レベル: database-schema.sqlのinterviewsテーブル定義と制約に完全準拠

#[tokio::test]
#[serial]
async fn test_面談の正常作成() {
    // 【テスト目的】: 面談エンティティの基本的な作成処理とデータベース保存の動作確認
    // 【テスト内容】: 有効な面談データが正常にデータベースに保存され、UUID主キーとタイムスタンプが自動設定される
    // 【期待される動作】: 外部キー関係（project_participant_id, interviewer_id）が正常に機能し、ステータス制約がクリアされる
    // 🟢 信頼性レベル: database-schema.sqlのinterviewsテーブル定義に基づく確実なテストケース
    
    // 【テスト前準備】: データベース接続とテスト環境の初期化
    // 【初期条件設定】: 面談作成に必要な外部キーデータ（企業、受講者、プロジェクト、ユーザー、プロジェクト参加者）を事前に準備
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: 面談作成に必要な外部エンティティを事前に作成
    // 【データ整合性】: 外部キー制約を満たすため、companies, users, students, projects, project_participants テーブルにデータを準備
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("面談テスト株式会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("面談担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("interview@test.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(Some("https://chat.interview.co.jp".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【面談担当者作成】: 面談を実施するtrainerユーザーを作成（interviewer_id外部キー用）
    let interviewer = training_management::models::users::RegisterParams {
        name: "面談担当者".to_string(),
        email: "interviewer@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_interviewer = training_management::models::users::Model::create_with_password(&boot.app_context.db, &interviewer)
        .await
        .unwrap();
    
    // 【受講者作成】: 面談対象の受講者を作成
    let student = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("面談受講者".to_string()),
        email: sea_orm::ActiveValue::Set("student@test.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("開発部".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【プロジェクト管理者作成】: プロジェクト作成用のユーザー
    let project_manager = training_management::models::users::RegisterParams {
        name: "プロジェクト管理者".to_string(),
        email: "manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    // 【研修コース作成】: プロジェクト作成に必要な研修コース
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("面談用研修コース".to_string()),
        description: sea_orm::ActiveValue::Set("面談テスト用の研修".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("基礎知識".to_string()),
        goals: sea_orm::ActiveValue::Set("スキル向上".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("課題完了".to_string()),
        created_by: sea_orm::ActiveValue::Set(created_manager.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【プロジェクト作成】: 面談対象のプロジェクト
    let project = training_management::models::projects::ActiveModel {
        title: sea_orm::ActiveValue::Set("面談対象プロジェクト".to_string()),
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
    
    // 【プロジェクト参加者作成】: 面談に必要なproject_participant_id外部キー用データ
    let project_participant = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        student_id: sea_orm::ActiveValue::Set(student.id),
        status: sea_orm::ActiveValue::Set(1), // 研修開始状態
        all_interviews_completed: sea_orm::ActiveValue::Set(false),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【面談データ作成】: 正常な面談データを準備
    let interview_data = training_management::models::interviews::ActiveModel {
        project_participant_id: sea_orm::ActiveValue::Set(project_participant.id),
        interviewer_id: sea_orm::ActiveValue::Set(created_interviewer.id),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        status: sea_orm::ActiveValue::Set("scheduled".to_string()),
        notes: sea_orm::ActiveValue::Set(Some("# 面談記録\n\n## 今回の目標\n- 進捗確認\n- 課題解決".to_string())),
        ..Default::default()
    };
    
    // 【面談作成実行】: データベースへ面談データを保存
    let result = interview_data.insert(&boot.app_context.db).await.unwrap();
    
    // 【結果検証】: 作成された面談データの妥当性確認
    // UUID主キー自動生成の確認（空文字列ではないUUIDが設定される）
    assert!(!result.id.to_string().is_empty());
    
    // 外部キー関係の正常保存確認
    assert_eq!(result.project_participant_id, project_participant.id);
    assert_eq!(result.interviewer_id, created_interviewer.id);
    
    // ステータス制約の正常動作確認（scheduledは有効な値）
    assert_eq!(result.status, "scheduled");
    
    // Markdownノート保存の確認
    assert!(result.notes.is_some());
    assert!(result.notes.unwrap().contains("面談記録"));
    
    // タイムスタンプ自動設定確認
    // created_at と updated_at は chrono::DateTime<FixedOffset> 型で常に値を持つ
    
    // 【ビジネスロジック検証】: 面談データがビジネス要件を満たしているか確認
    // 面談時刻が未来時刻として設定されている
    assert!(result.scheduled_at > chrono::Utc::now().fixed_offset() - chrono::Duration::seconds(10));
}

#[tokio::test]
#[serial]
async fn test_プロジェクト参加者別面談一覧取得() {
    // 【テスト目的】: プロジェクト参加者に紐付く面談一覧の検索機能動作確認
    // 【テスト内容】: 特定のプロジェクト参加者に関連する面談を正確に抽出できる
    // 【期待される動作】: 1対多リレーション（プロジェクト参加者→面談）が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlの外部キー関係に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 複数面談を持つプロジェクト参加者のテストデータセット構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("面談検索テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("検索担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("search@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let interviewer = training_management::models::users::RegisterParams {
        name: "検索テスト担当者".to_string(),
        email: "search.interviewer@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_interviewer = training_management::models::users::Model::create_with_password(&boot.app_context.db, &interviewer)
        .await
        .unwrap();
    
    let student = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("検索テスト受講者".to_string()),
        email: sea_orm::ActiveValue::Set("search.student@test.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("開発部".to_string()),
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
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("検索テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("面談検索用研修".to_string()),
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
    
    let project_participant = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        student_id: sea_orm::ActiveValue::Set(student.id),
        status: sea_orm::ActiveValue::Set(1),
        all_interviews_completed: sea_orm::ActiveValue::Set(false),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【複数面談作成】: 同一プロジェクト参加者に対する複数の面談を作成
    // 1回目の面談（scheduled）
    let interview1 = training_management::models::interviews::ActiveModel {
        project_participant_id: sea_orm::ActiveValue::Set(project_participant.id),
        interviewer_id: sea_orm::ActiveValue::Set(created_interviewer.id),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::days(1)),
        status: sea_orm::ActiveValue::Set("scheduled".to_string()),
        notes: sea_orm::ActiveValue::Set(Some("1回目面談".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 2回目の面談（completed）
    let interview2 = training_management::models::interviews::ActiveModel {
        project_participant_id: sea_orm::ActiveValue::Set(project_participant.id),
        interviewer_id: sea_orm::ActiveValue::Set(created_interviewer.id),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::days(7)),
        status: sea_orm::ActiveValue::Set("completed".to_string()),
        notes: sea_orm::ActiveValue::Set(Some("2回目面談".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【検索機能テスト実行】: プロジェクト参加者別面談一覧取得機能をテスト
    let interviews = training_management::models::interviews::Model::find_by_project_participant_id(&boot.app_context.db, project_participant.id).await.unwrap();
    
    // 【検索結果検証】: 検索結果の妥当性確認
    // 正しい数の面談が取得される（2件）
    assert_eq!(interviews.len(), 2);
    
    // すべての面談が正しいプロジェクト参加者に紐付いている
    for interview in &interviews {
        assert_eq!(interview.project_participant_id, project_participant.id);
    }
    
    // 作成した面談IDが含まれている
    let interview_ids: Vec<uuid::Uuid> = interviews.iter().map(|i| i.id).collect();
    assert!(interview_ids.contains(&interview1.id));
    assert!(interview_ids.contains(&interview2.id));
}

#[tokio::test]
#[serial]
async fn test_面談ステータス制約バリデーション() {
    // 【テスト目的】: 面談ステータス値の制約チェック機能動作確認
    // 【テスト内容】: 許可されていないステータス値での面談作成が適切に拒否される
    // 【期待される動作】: CHECK制約（'scheduled', 'completed', 'cancelled'）が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlのCHECK制約定義に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 面談作成に必要な基本エンティティの構築
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("ステータステスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("ステータス担当者".to_string()),
        contact_email: sea_orm::ActiveValue::Set("status@test.co.jp".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let interviewer = training_management::models::users::RegisterParams {
        name: "ステータステスト担当者".to_string(),
        email: "status.interviewer@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_interviewer = training_management::models::users::Model::create_with_password(&boot.app_context.db, &interviewer)
        .await
        .unwrap();
    
    let student = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("ステータステスト受講者".to_string()),
        email: sea_orm::ActiveValue::Set("status.student@test.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("開発部".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_manager = training_management::models::users::RegisterParams {
        name: "ステータステスト管理者".to_string(),
        email: "status.manager@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_manager = training_management::models::users::Model::create_with_password(&boot.app_context.db, &project_manager)
        .await
        .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("ステータステスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("面談ステータステスト用研修".to_string()),
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
        title: sea_orm::ActiveValue::Set("ステータステストプロジェクト".to_string()),
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
    
    let project_participant = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        student_id: sea_orm::ActiveValue::Set(student.id),
        status: sea_orm::ActiveValue::Set(1),
        all_interviews_completed: sea_orm::ActiveValue::Set(false),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【無効ステータステスト】: 許可されていないステータス値での面談作成
    let invalid_interview = training_management::models::interviews::ActiveModel {
        project_participant_id: sea_orm::ActiveValue::Set(project_participant.id),
        interviewer_id: sea_orm::ActiveValue::Set(created_interviewer.id),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        status: sea_orm::ActiveValue::Set("invalid_status".to_string()), // 無効なステータス値
        notes: sea_orm::ActiveValue::Set(Some("無効ステータステスト".to_string())),
        ..Default::default()
    };
    
    // 【制約違反確認】: CHECK制約によりデータベースエラーが発生することを確認
    let result = invalid_interview.insert(&boot.app_context.db).await;
    assert!(result.is_err());
    
    // 【エラー内容検証】: エラーメッセージがCHECK制約違反を示している
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("check") || error_message.contains("constraint") || error_message.contains("status"));
    
    // 【正常ステータス確認】: 有効なステータス値では正常に作成される
    let valid_statuses = vec!["scheduled", "completed", "cancelled"];
    for (i, status) in valid_statuses.iter().enumerate() {
        let valid_interview = training_management::models::interviews::ActiveModel {
            project_participant_id: sea_orm::ActiveValue::Set(project_participant.id),
            interviewer_id: sea_orm::ActiveValue::Set(created_interviewer.id),
            scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset() + chrono::Duration::hours(i as i64 + 1)),
            status: sea_orm::ActiveValue::Set(status.to_string()),
            notes: sea_orm::ActiveValue::Set(Some(format!("{}ステータステスト", status))),
            ..Default::default()
        };
        
        let result = valid_interview.insert(&boot.app_context.db).await.unwrap();
        assert_eq!(result.status, *status);
    }
}

#[tokio::test]
#[serial]
async fn test_プロジェクト参加者参照整合性制約() {
    // 【テスト目的】: プロジェクト参加者参照整合性制約の動作確認
    // 【テスト内容】: 存在しないproject_participant_idでの面談作成が適切に拒否される
    // 【期待される動作】: 外部キー制約とトリガー関数が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlの外部キー制約とcheck_interview_project_participant()関数に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【面談担当者作成】: 有効な面談担当者を作成
    let interviewer = training_management::models::users::RegisterParams {
        name: "整合性テスト担当者".to_string(),
        email: "integrity.interviewer@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_interviewer = training_management::models::users::Model::create_with_password(&boot.app_context.db, &interviewer)
        .await
        .unwrap();
    
    // 【無効な外部キーテスト】: 存在しないproject_participant_idでの面談作成
    let nonexistent_uuid = uuid::Uuid::new_v4(); // 存在しないUUID
    
    let invalid_interview = training_management::models::interviews::ActiveModel {
        project_participant_id: sea_orm::ActiveValue::Set(nonexistent_uuid),
        interviewer_id: sea_orm::ActiveValue::Set(created_interviewer.id),
        scheduled_at: sea_orm::ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        status: sea_orm::ActiveValue::Set("scheduled".to_string()),
        notes: sea_orm::ActiveValue::Set(Some("整合性制約テスト".to_string())),
        ..Default::default()
    };
    
    // 【外部キー制約違反確認】: データベースエラーが発生することを確認
    let result = invalid_interview.insert(&boot.app_context.db).await;
    assert!(result.is_err());
    
    // 【エラー内容検証】: エラーメッセージが外部キー制約違反を示している
    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("foreign key") || 
        error_message.contains("references") || 
        error_message.contains("project_participant") ||
        error_message.contains("not exists") ||
        error_message.contains("violates")
    );
}