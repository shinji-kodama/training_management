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

#[tokio::test]
#[serial]
async fn test_受講者企業間移管機能正常動作() {
    // 【テスト目的】: 管理者権限による受講者の企業間移管処理の動作確認
    // 【テスト内容】: 受講者の所属企業変更とデータ整合性保持の確認
    // 【期待される動作】: 企業ID変更後も一意制約と外部キー制約が正常に動作する
    // 🟡 信頼性レベル: 要件定義R-203-006の企業移管機能に基づく高い確率で正しい仕様

    // 【テスト前準備】: 企業間移管テストに必要な複数企業と受講者データの準備
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【移管元企業作成】: 受講者移管テスト用の移管元企業データ作成
    let source_company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("移管元株式会社".to_string()),
        contact_person: ActiveValue::set("移管元担当者".to_string()),
        contact_email: ActiveValue::set("source@transfer-test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let source_company = source_company_data.insert(&boot.app_context.db).await
        .expect("Failed to create source company");

    // 【移管先企業作成】: 受講者移管テスト用の移管先企業データ作成
    let target_company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("移管先株式会社".to_string()),
        contact_person: ActiveValue::set("移管先担当者".to_string()),
        contact_email: ActiveValue::set("target@transfer-test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let target_company = target_company_data.insert(&boot.app_context.db).await
        .expect("Failed to create target company");

    // 【移管対象受講者作成】: 移管元企業に所属する受講者データ作成
    // 【初期条件設定】: 移管前は移管元企業に所属している状態
    let student_data = ActiveModel {
        name: ActiveValue::set("移管対象受講者".to_string()),
        email: ActiveValue::set("transfer@student.co.jp".to_string()),
        company_id: ActiveValue::set(source_company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("移管前部署".to_string()),
        ..Default::default()
    };
    let student = student_data.insert(&boot.app_context.db).await
        .expect("Failed to create student for transfer");

    // 【実際の処理実行】: Student::transfer_to_company()メソッドによる企業間移管処理
    // 【処理内容】: 受講者の企業ID変更と関連データ整合性確保
    // 【実行タイミング】: 管理者権限での企業移管操作実行
    let transfer_result = training_management::models::students::Model::transfer_to_company(
        &boot.app_context.db, 
        student.id, 
        target_company.id
    ).await;

    // 【結果検証】: 移管処理の成功と移管後のデータ整合性確認
    // 【期待値確認】: 企業ID変更と一意制約の維持確認
    // 【品質保証】: 企業間移管での外部キー制約とビジネスルール整合性確認
    assert!(transfer_result.is_ok(), "受講者企業移管が失敗しました: {:?}", transfer_result.err()); // 【確認内容】: 企業移管処理が正常完了することを確認 🟡

    let transferred_student = transfer_result.unwrap();
    assert_eq!(transferred_student.id, student.id); // 【確認内容】: 受講者IDが変更されないことを確認 🟡
    assert_eq!(transferred_student.company_id, target_company.id); // 【確認内容】: 企業IDが移管先に正確に変更されることを確認 🟡
    assert_eq!(transferred_student.email, student.email); // 【確認内容】: メールアドレスが変更されないことを確認 🟡
    assert_eq!(transferred_student.name, student.name); // 【確認内容】: 受講者名が変更されないことを確認 🟡
}

#[tokio::test]
#[serial]
async fn test_進行中研修参加受講者削除制約違反エラー() {
    // 【テスト目的】: 削除制約ビジネスルールの動作確認
    // 【テスト内容】: 進行中の研修に参加する受講者の削除試行による制約違反確認
    // 【期待される動作】: ビジネスルールにより削除が拒否され適切なエラーが返却される
    // 🟡 信頼性レベル: 要件定義R-203-004の削除制約とTASK-202削除制約パターンに基づく

    // 【テスト前準備】: 削除制約テストに必要な企業・受講者・プロジェクトデータの準備
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 削除制約テスト用の企業データ作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("削除制約テスト株式会社".to_string()),
        contact_person: ActiveValue::set("削除制約担当者".to_string()),
        contact_email: ActiveValue::set("delete@constraint-test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【削除対象受講者作成】: 削除制約チェック対象となる受講者データ作成
    // 【初期条件設定】: 削除制約の確認対象となる受講者レコード
    let student_data = ActiveModel {
        name: ActiveValue::set("削除制約対象受講者".to_string()),
        email: ActiveValue::set("constraint@test-student.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("開発部".to_string()),
        ..Default::default()
    };
    let student = student_data.insert(&boot.app_context.db).await
        .expect("Failed to create student for constraint test");

    // 【関連プロジェクト作成】: 受講者が参加中の研修プロジェクトを作成
    // 【制約条件設定】: 削除を制限する関連データの準備
    // 注意: これは実際のプロジェクト作成処理の呼び出しを想定（まだ未実装）
    // TODO: プロジェクト作成機能実装後に実際のコードに更新
    
    // 【実際の処理実行】: Student::delete_with_constraints()メソッドによる制約チェック付き削除試行
    // 【処理内容】: 関連データ存在チェックと削除可否判定処理
    // 【実行タイミング】: ビジネスルールによる削除制約チェック実行
    let delete_result = training_management::models::students::Model::delete_with_constraints(
        &boot.app_context.db, 
        student.id
    ).await;

    // 【結果検証】: 削除制約違反エラーが適切に発生することを確認
    // 【期待値確認】: ビジネスルールにより削除が拒否されることを確認
    // 【品質保証】: 関連データ保護とビジネス継続性の確保
    assert!(delete_result.is_err(), "進行中研修参加受講者の削除が成功してしまいました"); // 【確認内容】: 削除制約違反エラーが発生することを確認 🟡
    
    // 【エラー内容確認】: 適切なエラーメッセージが返却されることを確認
    // 【セキュリティ確認】: ビジネスルール違反時の適切なエラー処理
    let error = delete_result.unwrap_err();
    // 【エラー詳細検証】: 削除制約違反の具体的エラーメッセージを確認
    assert!(error.to_string().contains("進行中の研修"), "期待されるエラーメッセージが含まれていません");
    // TODO: 具体的なエラー種別の確認（ModelErrorの種類など）を実装後に追加
}

#[tokio::test]
#[serial]
async fn test_受講者バリデーションエラー処理() {
    // 【テスト目的】: 受講者作成時のバリデーション機能動作確認
    // 【テスト内容】: 不正な入力値でのバリデーションエラー発生確認
    // 【期待される動作】: 各バリデーション制約で適切なエラーが発生する
    // 🟢 信頼性レベル: 既存Validator実装とvalidatorクレートに基づく確実なテスト

    // 【テスト前準備】: バリデーションテスト用の企業データ準備
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: バリデーションテスト用の企業データ作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("バリデーションテスト株式会社".to_string()),
        contact_person: ActiveValue::set("バリデーション担当者".to_string()),
        contact_email: ActiveValue::set("validation@test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【空文字名前のテスト】: 受講者名が空文字でのバリデーションエラー確認
    // 【制約条件設定】: 名前フィールドの必須制約とlengthバリデーション
    let empty_name_student = ActiveModel {
        name: ActiveValue::set("".to_string()), // 空文字（バリデーション違反）
        email: ActiveValue::set("empty-name@test.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("開発部".to_string()),
        ..Default::default()
    };

    // 【実際の処理実行】: 空文字名前での受講者作成試行
    // 【処理内容】: バリデーションエラーが期待される作成処理
    let empty_name_result = empty_name_student.insert(&boot.app_context.db).await;

    // 【結果検証】: 空文字名前でのバリデーションエラー確認
    // 【期待値確認】: lengthバリデーションによるエラー発生
    assert!(empty_name_result.is_err(), "空文字名前での受講者作成が成功してしまいました"); // 【確認内容】: 名前必須バリデーションエラーが発生することを確認 🟢

    // 【不正メール形式のテスト】: 無効なメールアドレス形式でのバリデーションエラー確認
    // 【制約条件設定】: メールフィールドのemail形式バリデーション
    let invalid_email_student = ActiveModel {
        name: ActiveValue::set("不正メールテスト".to_string()),
        email: ActiveValue::set("invalid-email-format".to_string()), // 不正メール形式（バリデーション違反）
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("student".to_string()),
        organization: ActiveValue::set("開発部".to_string()),
        ..Default::default()
    };

    // 【実際の処理実行】: 不正メール形式での受講者作成試行
    // 【処理内容】: emailバリデーションエラーが期待される作成処理
    let invalid_email_result = invalid_email_student.insert(&boot.app_context.db).await;

    // 【結果検証】: 不正メール形式でのバリデーションエラー確認
    // 【期待値確認】: emailバリデーションによるエラー発生
    assert!(invalid_email_result.is_err(), "不正メール形式での受講者作成が成功してしまいました"); // 【確認内容】: メール形式バリデーションエラーが発生することを確認 🟢

    // 【不正役割タイプのテスト】: 許可されていない役割タイプでのバリデーションエラー確認
    // 【制約条件設定】: role_typeフィールドのカスタムバリデーション
    let invalid_role_student = ActiveModel {
        name: ActiveValue::set("不正役割テスト".to_string()),
        email: ActiveValue::set("invalid-role@test.co.jp".to_string()),
        company_id: ActiveValue::set(company.id),
        role_type: ActiveValue::set("invalid_role".to_string()), // 不正役割タイプ（バリデーション違反）
        organization: ActiveValue::set("開発部".to_string()),
        ..Default::default()
    };

    // 【実際の処理実行】: 不正役割タイプでの受講者作成試行
    // 【処理内容】: カスタムバリデーションエラーが期待される作成処理
    let invalid_role_result = invalid_role_student.insert(&boot.app_context.db).await;

    // 【結果検証】: 不正役割タイプでのバリデーションエラー確認
    // 【期待値確認】: カスタムバリデーションによるエラー発生
    assert!(invalid_role_result.is_err(), "不正役割タイプでの受講者作成が成功してしまいました"); // 【確認内容】: 役割タイプバリデーションエラーが発生することを確認 🟢
}

#[tokio::test]
#[serial]
async fn test_受講者高度検索機能動作() {
    // 【テスト目的】: 受講者の高度検索機能群の動作確認
    // 【テスト内容】: 複合条件検索、役割タイプ別検索、ページネーション機能
    // 【期待される動作】: 各検索条件で正確なフィルタリングとソートが実行される
    // 🟡 信頼性レベル: 要件定義R-203-005の検索機能要件に基づく

    // 【テスト前準備】: 高度検索テスト用の企業と複数受講者データの準備
    // 【環境初期化】: データベーステーブルが空の状態から開始
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【企業事前作成】: 高度検索テスト用の企業データ作成
    let company_data = training_management::models::companies::ActiveModel {
        name: ActiveValue::set("高度検索テスト株式会社".to_string()),
        contact_person: ActiveValue::set("検索担当者".to_string()),
        contact_email: ActiveValue::set("search@advanced-test.co.jp".to_string()),
        chat_link: ActiveValue::set(None),
        ..Default::default()
    };
    let company = company_data.insert(&boot.app_context.db).await
        .expect("Failed to create test company");

    // 【複数受講者作成】: 検索テスト用の多様な受講者データ作成
    // 【初期条件設定】: 異なる役割タイプと部署を持つ受講者群の準備
    let students_data = vec![
        ("Aさん", "a@advanced-test.co.jp", "student", "開発部"),
        ("Bさん", "b@advanced-test.co.jp", "company_admin", "管理部"),
        ("Cさん", "c@advanced-test.co.jp", "student", "営業部"),
        ("Dさん", "d@advanced-test.co.jp", "student", "開発部"),
        ("Eさん", "e@advanced-test.co.jp", "company_admin", "管理部"),
    ];

    for (name, email, role, org) in students_data {
        let student_data = ActiveModel {
            name: ActiveValue::set(name.to_string()),
            email: ActiveValue::set(email.to_string()),
            company_id: ActiveValue::set(company.id),
            role_type: ActiveValue::set(role.to_string()),
            organization: ActiveValue::set(org.to_string()),
            ..Default::default()
        };
        student_data.insert(&boot.app_context.db).await
            .expect("Failed to create test students");
    }

    // 【実際の処理実行】: Student::search_with_filters()メソッドによる高度検索処理
    // 【処理内容】: 複合条件検索とフィルタリング機能の実行
    // 【実行タイミング】: 検索条件による受講者一覧取得
    let search_result = training_management::models::students::Model::search_with_filters(
        &boot.app_context.db,
        Some(company.id),           // 企業ID
        Some("student".to_string()), // 役割タイプフィルタ
        None,                       // 名前フィルタなし
        Some("開発部".to_string())    // 組織フィルタ
    ).await;

    // 【結果検証】: 複合条件検索結果の正確性確認
    // 【期待値確認】: 指定条件に合致する受講者のみ取得されることを確認
    // 【品質保証】: フィルタリング精度とデータスコープ制限の確認
    assert!(search_result.is_ok(), "高度検索が失敗しました: {:?}", search_result.err()); // 【確認内容】: 高度検索処理が正常完了することを確認 🟡

    let filtered_students = search_result.unwrap();
    assert_eq!(filtered_students.len(), 2); // 【確認内容】: student役割かつ開発部の受講者2名が取得されることを確認 🟡
    assert!(filtered_students.iter().all(|s| s.role_type == "student")); // 【確認内容】: 全員がstudent役割であることを確認 🟡
    assert!(filtered_students.iter().all(|s| s.organization == "開発部")); // 【確認内容】: 全員が開発部所属であることを確認 🟡
}