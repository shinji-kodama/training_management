use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App,
    models::students::ActiveModel,
};

#[tokio::test]
#[serial]
async fn test_受講者情報の正常作成() {
    // 【テスト目的】: 受講者エンティティの基本的な作成機能の動作確認
    // 【テスト内容】: 正常な受講者データでの作成処理とデータベース保存
    // 【期待される動作】: 有効な受講者データが正常にデータベースに保存される
    // 🟢 信頼性レベル: database-schema.sqlとtestcases.mdの定義に基づく確実なテストケース

    // 【テスト前準備】: 各テスト実行前にテスト環境を初期化し、一貫したテスト条件を保証
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 受講者作成に必要な関連企業データを準備
    // 【外部キー準備】: 受講者テーブルのcompany_id外部キー制約を満たすため
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("テスト株式会社".to_string()),
        contact_person: ActiveValue::set("田中太郎".to_string()),
        contact_email: ActiveValue::set("tanaka@testcompany.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【テストデータ準備】: 実際の受講者登録で使用される標準的な受講者情報
    // 【初期条件設定】: 受講者テーブルの制約とインデックスが正常に設定済み
    let student_data = ActiveModel {
        name: ActiveValue::set("山田花子".to_string()),
        email: ActiveValue::set("yamada@testcompany.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("開発部".to_string()),
        ..Default::default()
    };

    // 【実際の処理実行】: Student::create()メソッドによる受講者データ作成
    // 【処理内容】: ActiveModelを使用したSeaORM経由でのデータベース保存
    // 【実行タイミング】: トランザクション内での受講者レコード作成実行
    let result = student_data.insert(&boot.app_context.db).await;

    // 【結果検証】: 作成された受講者データの各フィールド値とタイムスタンプ確認
    // 【期待値確認】: UUID主キー生成、created_at/updated_at自動設定の検証
    // 【品質保証】: データベース制約とビジネスルールの整合性確認
    assert!(result.is_ok(), "受講者作成が失敗しました: {:?}", result.err()); // 【確認内容】: 受講者作成処理が正常完了することを確認 🟢

    let student = result.unwrap();
    assert_eq!(student.name, "山田花子"); // 【確認内容】: 受講者名が正確に保存されることを確認 🟢
    assert_eq!(student.email, "yamada@testcompany.co.jp"); // 【確認内容】: メールアドレスが正確に保存されることを確認 🟢
    assert_eq!(student.company_id, company.id); // 【確認内容】: 企業IDの外部キー関係が正確に保存されることを確認 🟢
    assert_eq!(student.role_type, "student"); // 【確認内容】: 役割タイプが正確に保存されることを確認 🟢
    assert_eq!(student.organization, "開発部"); // 【確認内容】: 所属組織が正確に保存されることを確認 🟢
    assert!(student.id != uuid::Uuid::nil()); // 【確認内容】: UUID主キーが自動生成されることを確認 🟢
    assert!(!student.created_at.to_string().is_empty()); // 【確認内容】: created_atが自動設定されることを確認 🟢
    assert!(!student.updated_at.to_string().is_empty()); // 【確認内容】: updated_atが自動設定されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_受講者企業リレーション検索() {
    // 【テスト目的】: 受講者と企業間のリレーション機能確認
    // 【テスト内容】: 特定企業に所属する受講者の検索機能
    // 【期待される動作】: 企業IDを指定して所属受講者を正常に取得できる
    // 🟢 信頼性レベル: database-schema.sqlの外部キー関係定義に基づく

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: リレーション検索テスト用の企業データ作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("リレーションテスト株式会社".to_string()),
        contact_person: ActiveValue::set("佐藤次郎".to_string()),
        contact_email: ActiveValue::set("sato@relationtest.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【複数受講者作成】: 企業に所属する複数受講者データの準備
    // 【リレーション検証】: 同一企業に複数受講者が所属するケースのテスト
    let student1_data = ActiveModel {
        name: ActiveValue::set("受講者1".to_string()),
        email: ActiveValue::set("student1@relationtest.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("営業部".to_string()),
        ..Default::default()
    };

    let student2_data = ActiveModel {
        name: ActiveValue::set("受講者2".to_string()),
        email: ActiveValue::set("student2@relationtest.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("company_admin".to_string()),
        organization: ActiveValue::set("管理部".to_string()),
        ..Default::default()
    };

    // 【受講者データ保存】: テスト用受講者データのデータベース保存
    student1_data.insert(&boot.app_context.db).await
        .expect("Failed to create student1");
    student2_data.insert(&boot.app_context.db).await
        .expect("Failed to create student2");

    // 【実際の処理実行】: Student::find_by_company_id()メソッドによる企業所属受講者検索
    // 【処理内容】: 企業IDを条件とした受講者一覧取得処理
    let students_result = training_management::models::students::Model::find_by_company_id(&boot.app_context.db, company.id).await;

    // 【結果検証】: 取得された受講者リストの内容確認
    // 【期待値確認】: 指定企業に所属する全受講者が取得されることを確認
    assert!(students_result.is_ok(), "企業所属受講者の検索が失敗しました: {:?}", students_result.err()); // 【確認内容】: 企業所属受講者検索処理が正常完了することを確認 🟢

    let students = students_result.unwrap();
    assert_eq!(students.len(), 2); // 【確認内容】: 作成した受講者2人が正常に取得されることを確認 🟢
    assert!(students.iter().all(|s| s.company_id == company.id)); // 【確認内容】: 取得された全受講者が指定企業に所属することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_同一企業内メール重複エラー() {
    // 【テスト目的】: データベーススキーマの一意制約確認
    // 【テスト内容】: 同一企業内での受講者メールアドレス重複の制約確認
    // 【期待される動作】: 同一企業内での重複メールアドレスで制約違反エラーが発生
    // 🟢 信頼性レベル: database-schema.sqlのUNIQUE(email, company_id)制約に基づく

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 一意制約テスト用の企業データ作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("一意制約テスト株式会社".to_string()),
        contact_person: ActiveValue::set("田中三郎".to_string()),
        contact_email: ActiveValue::set("tanaka@uniquetest.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【最初の受講者作成】: 一意制約テストのベースとなる受講者データ作成
    // 【初期条件設定】: 重複チェック対象となるメールアドレスの事前登録
    let first_student_data = ActiveModel {
        name: ActiveValue::set("最初の受講者".to_string()),
        email: ActiveValue::set("duplicate@uniquetest.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("開発部".to_string()),
        ..Default::default()
    };

    let first_result = first_student_data.insert(&boot.app_context.db).await;
    assert!(first_result.is_ok(), "最初の受講者作成が失敗しました");

    // 【重複メール受講者作成試行】: 同一企業内で同じメールアドレスでの受講者作成試行
    // 【制約違反条件】: UNIQUE(email, company_id)制約に違反する条件でのデータ作成
    let duplicate_student_data = ActiveModel {
        name: ActiveValue::set("重複メールの受講者".to_string()),
        email: ActiveValue::set("duplicate@uniquetest.co.jp".to_string()), // 同じメールアドレス
        company_id: ActiveValue::set(company.id), // 同じ企業ID
        role_type: ActiveValue::set("company_admin".to_string()),
        organization: ActiveValue::set("管理部".to_string()),
        ..Default::default()
    };

    // 【実際の処理実行】: 重複データでの受講者作成試行
    // 【処理内容】: 一意制約違反が期待される受講者作成処理
    let duplicate_result = duplicate_student_data.insert(&boot.app_context.db).await;

    // 【結果検証】: 一意制約違反エラーが適切に発生することを確認
    // 【期待値確認】: データベース制約により作成が拒否されることを確認
    assert!(duplicate_result.is_err(), "同一企業内での重複メールアドレスでの受講者作成が成功してしまいました"); // 【確認内容】: 一意制約違反エラーが発生することを確認 🟢
}