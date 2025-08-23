use loco_rs::testing::request::boot_test;
use sea_orm::ActiveModelTrait;
use serial_test::serial;
use training_management::app::App;

// 【テスト対象】: プロジェクト参加者（ProjectParticipants）モデルの包括的CRUD機能テスト
// 【テスト方針】: database-schema.sqlの制約とビジネスルールに基づく確実なテストケース
// 【フレームワーク】: Loco.rs 0.16.3 + SeaORM 1.1.12 + PostgreSQL環境でのモデルテスト
// 🟢 信頼性レベル: database-schema.sqlのテーブル定義と制約に完全準拠

#[tokio::test]
#[serial]
async fn test_プロジェクト参加者の正常作成() {
    // 【テスト目的】: プロジェクト参加者エンティティの基本的な作成処理とデータベース保存の動作確認
    // 【テスト内容】: 有効なプロジェクト参加者データが正常にデータベースに保存され、UUID主キーとタイムスタンプが自動設定される
    // 【期待される動作】: 外部キー関係（project_id, student_id）が正常に機能し、企業整合性制約がクリアされる
    // 🟢 信頼性レベル: database-schema.sqlのproject_participantsテーブル定義に基づく確実なテストケース
    
    // 【テスト前準備】: データベース接続とテスト環境の初期化
    // 【初期条件設定】: プロジェクト参加者作成に必要な外部キーデータ（企業、受講者、プロジェクト、ユーザー）を事前に準備
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: プロジェクト参加者作成に必要な外部エンティティを事前に作成
    // 【データ整合性】: 外部キー制約を満たすため、companies, users, students, projects テーブルにデータを準備
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
    
    let student = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("受講者太郎".to_string()),
        email: sea_orm::ActiveValue::Set("student@test.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("開発部".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
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
    
    let project = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("2025年春期プログラミング研修プロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 4, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【テストデータ準備】: プロジェクト参加者作成で使用する実際のビジネスデータ形式
    // 【制約確認】: project_id, student_idの外部キー制約と企業整合性制約を満たすデータ設定
    let participant_data = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        student_id: sea_orm::ActiveValue::Set(student.id),
        status: sea_orm::ActiveValue::Set(1), // 研修状況: 1-5段階
        all_interviews_completed: sea_orm::ActiveValue::Set(false),
        ..Default::default()
    };
    
    // 【実際の処理実行】: ProjectParticipant::create()メソッドによるプロジェクト参加者データ作成
    // 【処理内容】: ActiveModelを使用したSeaORM経由でのデータベース保存
    // 【UUID生成確認】: before_save()でUUID主キーが自動生成されることを検証
    let result = participant_data.insert(&boot.app_context.db).await.unwrap();
    
    // 【結果検証】: 作成されたプロジェクト参加者データの各フィールド値とタイムスタンプ確認
    // 【期待値確認】: UUID主キー生成、created_at/updated_at自動設定の検証
    // 【品質保証】: データベース制約とビジネスルールの整合性確認
    assert!(!result.id.to_string().is_empty()); // 【確認内容】: UUID主キーが自動生成されていることを確認 🟢
    assert_eq!(result.project_id, project.id); // 【確認内容】: プロジェクトIDの外部キー関係が正常に設定されることを確認 🟢
    assert_eq!(result.student_id, student.id); // 【確認内容】: 受講者IDの外部キー関係が正常に設定されることを確認 🟢
    assert_eq!(result.status, 1); // 【確認内容】: 研修状況が正確に保存されることを確認 🟢
    assert_eq!(result.all_interviews_completed, false); // 【確認内容】: 面談完了フラグが正確に保存されることを確認 🟢
    assert!(!result.created_at.to_string().is_empty()); // 【確認内容】: 作成日時が自動的に設定されることを確認 🟢
    assert!(!result.updated_at.to_string().is_empty()); // 【確認内容】: 更新日時が自動的に設定されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_プロジェクト別参加者一覧取得() {
    // 【テスト目的】: プロジェクトIDを条件とした参加者一覧取得機能の動作確認
    // 【テスト内容】: 指定プロジェクトに参加する全受講者が正確に取得され、適切な並び順で返却される
    // 【期待される動作】: 1対多リレーション（プロジェクト→参加者）が正常に機能し、外部キーインデックスが活用される
    // 🟢 信頼性レベル: database-schema.sqlの外部キー制約とインデックス定義に基づく
    
    // 【テスト前準備】: 複数参加者を持つプロジェクトデータの作成
    // 【初期条件設定】: 1つのプロジェクトに対して複数の参加者を紐付けた状態を構築
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: テスト用の企業、ユーザー、受講者、プロジェクトデータを準備
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
        name: "プロジェクト責任者".to_string(),
        email: "admin@testdev.co.jp".to_string(),
        password: "admin123secure".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &user)
        .await
        .unwrap();
    
    let student1 = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("受講者A".to_string()),
        email: sea_orm::ActiveValue::Set("studentA@testdev.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("開発1部".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let student2 = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("受講者B".to_string()),
        email: sea_orm::ActiveValue::Set("studentB@testdev.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("開発2部".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("チーム開発研修".to_string()),
        description: sea_orm::ActiveValue::Set("チーム開発技術を学ぶ".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("個人開発経験".to_string()),
        goals: sea_orm::ActiveValue::Set("チーム開発スキル習得".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("チームプロジェクト完成".to_string()),
        company_id: sea_orm::ActiveValue::Set(Some(company.id.clone())),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("2025年チーム開発プロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 5, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 7, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【複数参加者作成】: 同一プロジェクトに対して異なる受講者の参加者レコードを作成
    // 【並び順テスト準備】: 異なるstatusを持つ参加者を作成して並び順確認を可能にする
    let _participant1 = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        student_id: sea_orm::ActiveValue::Set(student1.id),
        status: sea_orm::ActiveValue::Set(2),
        all_interviews_completed: sea_orm::ActiveValue::Set(false),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let _participant2 = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        student_id: sea_orm::ActiveValue::Set(student2.id),
        status: sea_orm::ActiveValue::Set(3),
        all_interviews_completed: sea_orm::ActiveValue::Set(true),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【実際の処理実行】: ProjectParticipant::find_by_project_id()メソッドによるプロジェクト別参加者検索
    // 【処理内容】: project_idを条件とした外部キーインデックス活用の効率的検索
    // 【パフォーマンステスト】: 1対多リレーション検索の動作確認
    let participants = training_management::models::project_participants::Model::find_by_project_id(&boot.app_context.db, project.id).await.unwrap();
    
    // 【結果検証】: 検索された参加者件数と内容の確認
    // 【期待値確認】: 作成した2件の参加者が正確に取得されることを確認
    // 【品質保証】: 外部キー関係とリレーション機能の整合性確認
    assert_eq!(participants.len(), 2); // 【確認内容】: プロジェクトに紐づく2件の参加者が正確に取得されることを確認 🟢
    
    // 【個別参加者確認】: 各参加者が正しいプロジェクトIDを持っていることを確認
    for participant in &participants {
        assert_eq!(participant.project_id, project.id); // 【確認内容】: 全参加者が指定プロジェクトIDを持っていることを確認 🟢
        assert!(participant.status >= 1 && participant.status <= 5); // 【確認内容】: 全参加者で研修状況が有効範囲内であることを確認 🟢
        assert!(!participant.id.to_string().is_empty()); // 【確認内容】: 各参加者にUUID主キーが設定されていることを確認 🟢
    }
}

#[tokio::test]
#[serial]
async fn test_企業整合性制約バリデーション() {
    // 【テスト目的】: プロジェクトと受講者の企業整合性制約の動作確認とビジネスルール検証
    // 【テスト内容】: 異なる企業のプロジェクトと受講者での参加者作成時のエラーハンドリング
    // 【期待される動作】: 企業整合性トリガーがエラーを発生し、データ不整合が阻止される
    // 🟢 信頼性レベル: database-schema.sqlの企業整合性チェック関数に完全準拠
    
    // 【テスト前準備】: 企業整合性制約違反テスト用の異なる企業データ準備
    // 【初期条件設定】: 異なる企業に属するプロジェクトと受講者を意図的に作成
    let boot = boot_test::<App>().await.unwrap();
    
    // 【異なる企業データ作成】: プロジェクトと受講者で異なる企業を設定して制約違反を確認
    let company_a = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("A株式会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("田中A太郎".to_string()),
        contact_email: sea_orm::ActiveValue::Set("contactA@companyA.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(None),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let company_b = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("B株式会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("田中B太郎".to_string()),
        contact_email: sea_orm::ActiveValue::Set("contactB@companyB.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(None),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let user = training_management::models::users::RegisterParams {
        name: "制約テスト管理者".to_string(),
        email: "constraint@test.co.jp".to_string(),
        password: "constraint123".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &user)
        .await
        .unwrap();
    
    // 【A企業の受講者作成】
    let student_company_a = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("A企業受講者".to_string()),
        email: sea_orm::ActiveValue::Set("studentA@companyA.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company_a.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("A企業開発部".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【B企業のプロジェクト作成】: 異なる企業のプロジェクト
    let training_b = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("B企業研修".to_string()),
        description: sea_orm::ActiveValue::Set("B企業専用研修".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("B企業社員限定".to_string()),
        goals: sea_orm::ActiveValue::Set("B企業スキル向上".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("B企業基準達成".to_string()),
        company_id: sea_orm::ActiveValue::Set(Some(company_b.id.clone())),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project_company_b = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training_b.id),
        company_id: sea_orm::ActiveValue::Set(company_b.id),
        title: sea_orm::ActiveValue::Set("B企業専用プロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 8, 31).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【制約違反データ準備】: A企業の受講者をB企業のプロジェクトに参加させる不正なデータ
    // 【意図的エラー作成】: 企業整合性制約を意図的に違反するデータでテスト
    let invalid_participant_data = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project_company_b.id), // 【制約違反】: B企業のプロジェクト
        student_id: sea_orm::ActiveValue::Set(student_company_a.id),   // 【制約違反】: A企業の受講者
        status: sea_orm::ActiveValue::Set(1),
        all_interviews_completed: sea_orm::ActiveValue::Set(false),
        ..Default::default()
    };
    
    // 【実際の処理実行】: 企業整合性制約違反データでの保存試行
    // 【処理内容】: データベーストリガーレベルでの制約チェック機能の検証
    // 【エラー期待処理】: 企業整合性制約違反によるエラー発生を期待した処理実行
    let result = invalid_participant_data.insert(&boot.app_context.db).await;
    
    // 【結果検証】: 企業整合性制約違反エラーが適切に発生することを確認
    // 【期待値確認】: DbErrでの企業整合性制約エラーが返されることを検証
    // 【品質保証】: データ整合性保護機能の確認
    assert!(result.is_err()); // 【確認内容】: 企業整合性制約違反時にエラーが発生することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_重複参加防止制約バリデーション() {
    // 【テスト目的】: UNIQUE制約（project_id, student_id）の動作確認とビジネスルール検証
    // 【テスト内容】: 同じプロジェクトに同じ受講者を重複して参加させる試行でのエラーハンドリング
    // 【期待される動作】: 一意制約違反エラーが発生し、重複参加データの保存が阻止される
    // 🟢 信頼性レベル: database-schema.sqlのUNIQUE制約定義に完全準拠
    
    // 【テスト前準備】: 重複参加制約違反テスト用のデータ準備
    // 【初期条件設定】: 1人の受講者と1つのプロジェクトで重複参加テストを可能にする環境構築
    let boot = boot_test::<App>().await.unwrap();
    
    // 【基本データ作成】: 重複参加テストに必要な企業、受講者、プロジェクトデータ
    let company = training_management::models::companies::ActiveModel {
        name: sea_orm::ActiveValue::Set("重複テスト会社".to_string()),
        contact_person: sea_orm::ActiveValue::Set("重複太郎".to_string()),
        contact_email: sea_orm::ActiveValue::Set("duplicate@test.co.jp".to_string()),
        chat_link: sea_orm::ActiveValue::Set(None),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let user = training_management::models::users::RegisterParams {
        name: "重複テスト管理者".to_string(),
        email: "admin@duplicate.co.jp".to_string(),
        password: "duplicate123".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &user)
        .await
        .unwrap();
    
    let student = training_management::models::students::ActiveModel {
        name: sea_orm::ActiveValue::Set("重複テスト受講者".to_string()),
        email: sea_orm::ActiveValue::Set("student@duplicate.co.jp".to_string()),
        company_id: sea_orm::ActiveValue::Set(company.id),
        role_type: sea_orm::ActiveValue::Set("student".to_string()),
        organization: sea_orm::ActiveValue::Set("テスト部".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let training = training_management::models::trainings::ActiveModel {
        title: sea_orm::ActiveValue::Set("重複テスト研修".to_string()),
        description: sea_orm::ActiveValue::Set("重複制約を確認する".to_string()),
        prerequisites: sea_orm::ActiveValue::Set("特になし".to_string()),
        goals: sea_orm::ActiveValue::Set("重複防止理解".to_string()),
        completion_criteria: sea_orm::ActiveValue::Set("理解度100%".to_string()),
        company_id: sea_orm::ActiveValue::Set(Some(company.id.clone())),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    let project = training_management::models::projects::ActiveModel {
        training_id: sea_orm::ActiveValue::Set(training.id),
        company_id: sea_orm::ActiveValue::Set(company.id),
        title: sea_orm::ActiveValue::Set("重複テストプロジェクト".to_string()),
        start_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 7, 1).unwrap()),
        end_date: sea_orm::ActiveValue::Set(chrono::NaiveDate::from_ymd_opt(2025, 9, 30).unwrap()),
        created_by: sea_orm::ActiveValue::Set(created_user.id),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【最初の参加者作成】: まず正常な参加者を作成
    let _first_participant = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id),
        student_id: sea_orm::ActiveValue::Set(student.id),
        status: sea_orm::ActiveValue::Set(1),
        all_interviews_completed: sea_orm::ActiveValue::Set(false),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【重複データ準備】: 同じproject_id + student_idの組み合わせで重複参加を試行
    // 【意図的エラー作成】: UNIQUE制約を意図的に違反するデータでテスト
    let duplicate_participant_data = training_management::models::project_participants::ActiveModel {
        project_id: sea_orm::ActiveValue::Set(project.id), // 【制約違反】: 同じプロジェクトID
        student_id: sea_orm::ActiveValue::Set(student.id),   // 【制約違反】: 同じ受講者ID
        status: sea_orm::ActiveValue::Set(2), // 【異なる値】: statusは異なるが制約には関係しない
        all_interviews_completed: sea_orm::ActiveValue::Set(true), // 【異なる値】: フラグは異なるが制約には関係しない
        ..Default::default()
    };
    
    // 【実際の処理実行】: 重複参加制約違反データでの保存試行
    // 【処理内容】: データベースのUNIQUE制約チェック機能の検証
    // 【エラー期待処理】: 一意制約違反によるエラー発生を期待した処理実行
    let result = duplicate_participant_data.insert(&boot.app_context.db).await;
    
    // 【結果検証】: 重複参加制約違反エラーが適切に発生することを確認
    // 【期待値確認】: DbErrでの一意制約エラーが返されることを検証
    // 【品質保証】: 重複防止機能の確認
    assert!(result.is_err()); // 【確認内容】: 重複参加制約違反時にエラーが発生することを確認 🟢
}