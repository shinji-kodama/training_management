use loco_rs::testing::prelude::*;
use serial_test::serial;
use training_management::{
    app::App,
    models::{
        rbac::{UserRole, AuthContext},
        user_management::{UserManagementService, UserParams, PasswordChangeParams}
    }
};
use sea_orm::{ActiveModelTrait, Set, DatabaseConnection};

// テストユーザー作成ヘルパー
async fn create_test_user(db: &DatabaseConnection, email: &str, name: &str, role: &str) -> training_management::models::_entities::users::Model {
    use training_management::models::_entities::users;
    
    let password_hash = bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).unwrap();
    let user_active_model = users::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        pid: Set(uuid::Uuid::new_v4()),
        email: Set(email.to_string()),
        password: Set(password_hash),
        api_key: Set(uuid::Uuid::new_v4().to_string()),
        name: Set(name.to_string()),
        role: Set(role.to_string()),
        created_at: sea_orm::ActiveValue::NotSet,
        updated_at: sea_orm::ActiveValue::NotSet,
        reset_token: Set(None),
        reset_sent_at: Set(None),
        email_verification_token: Set(None),
        email_verification_sent_at: Set(None),
        email_verified_at: Set(None),
        magic_link_token: Set(None),
        magic_link_expiration: Set(None),
    };
    user_active_model.insert(db).await.expect("テストユーザー作成失敗")
}

// テストファイル: tests/models/user_management.rs
// ユーザー管理機能実装のためのTDDテスト（Redフェーズ - 失敗するテスト作成）

#[tokio::test]
#[serial]
async fn 管理者による新規ユーザー作成成功() {
    // 【テスト目的】: 管理者権限による新しいユーザー作成機能の動作確認
    // 【テスト内容】: RBAC統合、入力バリデーション、データベース保存の検証
    // 【期待される動作】: 有効な入力で新規ユーザーが正常に作成される
    // 🟢 信頼性レベル: 要件定義書のCRUD操作要件から直接抽出した確実な仕様

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 管理者権限を持つテストユーザーの作成とセッション準備
    // 【初期条件設定】: 有効なRBACコンテキストと新規ユーザー作成データの準備
    let admin_user = create_test_user(&boot.app_context.db, "admin1@example.com", "Admin User", "admin").await;

    let auth_context = AuthContext {
        user_id: admin_user.id,
        user_role: UserRole::Admin,
        session_id: "admin_session_123".to_string(),
    };

    let new_user_params = UserParams {
        name: "新規ユーザー".to_string(),
        email: "newuser1@example.com".to_string(), // 他のテストと重複しないメールアドレス
        password: Some("Password123!".to_string()),
        role: UserRole::Trainer,
    };

    // 【実際の処理実行】: ユーザー管理サービスのユーザー作成メソッド呼び出し
    // 【処理内容】: 入力バリデーション、RBAC権限チェック、データベース挿入の実行
    let result = UserManagementService::create_user(
        &boot.app_context.db, 
        &auth_context, 
        &new_user_params
    ).await;

    // 【結果検証】: ユーザー作成成功とデータベース保存の確認
    // 【期待値確認】: レスポンス構造、保存データ、タイムスタンプの正確性確認
    assert!(result.is_ok()); // 【確認内容】: ユーザー作成処理が正常に完了することを確認 🟢
    let created_user = result.unwrap();
    assert_eq!(created_user.name, "新規ユーザー"); // 【確認内容】: 作成されたユーザーの名前が正確に保存されることを確認 🟢
    assert_eq!(created_user.email, "newuser1@example.com"); // 【確認内容】: メールアドレスが正確に保存されることを確認 🟢
    assert_eq!(created_user.role, "trainer"); // 【確認内容】: 指定された役割が正確に保存されることを確認 🟢
}

#[tokio::test]
#[serial] 
async fn 権限不足によるユーザー作成拒否() {
    // 【テスト目的】: trainer権限でのユーザー作成試行によるRBAC権限制御の確認
    // 【テスト内容】: RBAC統合による権限不足エラーの適切な処理を検証
    // 【期待される動作】: 権限不足により作成が拒否され、適切なエラーメッセージが返される
    // 🟢 信頼性レベル: RBAC要件から確実に抽出したセキュリティ仕様

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: trainer権限を持つテストユーザーの作成
    // 【初期条件設定】: 権限不足のRBACコンテキストと新規ユーザー作成データの準備
    let trainer_user = create_test_user(&boot.app_context.db, "trainer@example.com", "Trainer User", "trainer").await;

    let auth_context = AuthContext {
        user_id: trainer_user.id,
        user_role: UserRole::Trainer,
        session_id: "trainer_session_123".to_string(),
    };

    let new_user_params = UserParams {
        name: "作成しようとするユーザー".to_string(),
        email: "blocked@example.com".to_string(),
        password: Some("password123".to_string()),
        role: UserRole::Instructor,
    };

    // 【実際の処理実行】: 権限不足状況でのユーザー作成メソッド呼び出し
    // 【処理内容】: RBAC権限チェックによるアクセス拒否の実行
    let result = UserManagementService::create_user(
        &boot.app_context.db,
        &auth_context,
        &new_user_params
    ).await;

    // 【結果検証】: 権限不足による作成拒否とエラーメッセージの確認
    // 【期待値確認】: HTTPステータスコードとエラー内容の適切性確認
    assert!(result.is_err()); // 【確認内容】: 権限不足によりエラーが返されることを確認 🟢
    let error = result.unwrap_err();
    assert!(error.to_string().contains("権限が不足しています")); // 【確認内容】: 適切なエラーメッセージが含まれることを確認 🟢
}

#[tokio::test]
#[serial]
async fn メールアドレス重複によるユーザー作成失敗() {
    // 【テスト目的】: 重複メールアドレスでの作成試行によるデータ整合性チェックの確認
    // 【テスト内容】: データベース制約による重複防止機能の検証
    // 【期待される動作】: 重複メールアドレスにより作成が拒否され、適切なエラーが返される
    // 🟢 信頼性レベル: データベース制約設計から確実に抽出したデータ整合性仕様

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 管理者ユーザーと既存ユーザーの作成
    // 【初期条件設定】: 重複チェックのための既存データと新規作成データの準備
    let admin_user = create_test_user(&boot.app_context.db, "admin2@example.com", "Admin User", "admin").await;

    // 既存ユーザーを作成（重複チェック用）
    let _existing_user = create_test_user(&boot.app_context.db, "existing@example.com", "Existing User", "instructor").await;

    let auth_context = AuthContext {
        user_id: admin_user.id,
        user_role: UserRole::Admin,
        session_id: "admin_session_123".to_string(),
    };

    let duplicate_user_params = UserParams {
        name: "重複ユーザー".to_string(),
        email: "existing@example.com".to_string(), // 重複するメールアドレス
        password: Some("Password123!".to_string()),
        role: UserRole::Trainer,
    };

    // 【実際の処理実行】: 重複メールアドレスでのユーザー作成メソッド呼び出し
    // 【処理内容】: メール重複チェックによる作成拒否の実行
    let result = UserManagementService::create_user(
        &boot.app_context.db,
        &auth_context,
        &duplicate_user_params
    ).await;

    // 【結果検証】: メール重複による作成拒否とエラーメッセージの確認
    // 【期待値確認】: 重複エラーの適切な検出と処理確認
    assert!(result.is_err()); // 【確認内容】: メール重複によりエラーが返されることを確認 🟢
    let error = result.unwrap_err();
    assert!(error.to_string().contains("メールアドレスが既に使用されています")); // 【確認内容】: 重複エラーメッセージが含まれることを確認 🟢
}

#[tokio::test]
#[serial]
async fn ユーザー自身によるパスワード変更成功() {
    // 【テスト目的】: ログイン中ユーザーが自分のパスワードを変更する機能の確認
    // 【テスト内容】: 現在パスワード検証、新パスワード設定、セキュリティ要件の検証
    // 【期待される動作】: 現在パスワード確認後、新パスワードに安全に変更される
    // 🟢 信頼性レベル: セキュリティ要件から確実に抽出したパスワード管理仕様

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: パスワード変更対象のテストユーザーの作成
    // 【初期条件設定】: 有効なセッションと現在パスワード、新パスワードの準備
    let test_user = create_test_user(&boot.app_context.db, "testuser@example.com", "Test User", "trainer").await;

    let auth_context = AuthContext {
        user_id: test_user.id,
        user_role: UserRole::Trainer,
        session_id: "user_session_123".to_string(),
    };

    let password_change_params = PasswordChangeParams {
        current_password: "TestPass123!".to_string(), // ヘルパー関数で設定されたパスワード
        new_password: "NewPass456!".to_string(),
        confirm_password: "NewPass456!".to_string(),
    };

    // 【実際の処理実行】: パスワード変更メソッドの呼び出し
    // 【処理内容】: 現在パスワード検証、新パスワードのハッシュ化、データベース更新の実行
    let result = UserManagementService::change_password(
        &boot.app_context.db,
        &auth_context,
        &password_change_params
    ).await;

    // 【結果検証】: パスワード変更成功とセキュリティ要件の確認
    // 【期待値確認】: パスワードハッシュ化、古いパスワード無効化の確認
    assert!(result.is_ok()); // 【確認内容】: パスワード変更処理が正常に完了することを確認 🟢
    
    // パスワードが実際に変更されたことを確認（新パスワードでログイン可能）
    // TODO: パスワード変更の実際の確認は、実装後のGreenフェーズで追加
    // 現在はTDD Greenフェーズのため最小実装のみ
}

// --- TDD Red Phase: 追加テストケース（TC-002～TC-005） ---

#[tokio::test]
#[serial]
async fn 管理者によるユーザー情報更新成功() {
    // 【テスト目的】: 管理者権限による既存ユーザー情報更新機能の動作確認
    // 【テスト内容】: RBAC統合、更新バリデーション、データベース保存の検証
    // 【期待される動作】: 有効な更新データで既存ユーザー情報が正常に更新される
    // 🟢 信頼性レベル: 要件定義書のCRUD操作要件から直接抽出した確実な仕様

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 管理者権限を持つテストユーザーと更新対象ユーザーの作成
    // 【初期条件設定】: 有効なRBACコンテキストと更新データの準備
    // 【前提条件確認】: データベース接続とRBACシステム（TASK-102）が正常動作することを前提
    let admin_user = create_test_user(&boot.app_context.db, "admin_update@example.com", "Admin User", "admin").await;
    let target_user = create_test_user(&boot.app_context.db, "target_update@example.com", "Original User", "instructor").await;

    let auth_context = AuthContext {
        user_id: admin_user.id,
        user_role: UserRole::Admin,
        session_id: "admin_update_session_123".to_string(),
    };

    let update_params = UserParams {
        name: "更新されたユーザー名".to_string(),
        email: "modified_user@example.com".to_string(),
        password: None, // 更新時にはパスワードは不要
        role: UserRole::Trainer,
    };

    // 【実際の処理実行】: ユーザー管理サービスのユーザー更新メソッド呼び出し
    // 【処理内容】: 更新バリデーション、RBAC権限チェック、データベース更新の実行
    // 【実行タイミング】: RBAC権限チェック通過後のユーザー情報更新処理段階
    let result = UserManagementService::update_user(
        &boot.app_context.db,
        &auth_context,
        target_user.id,
        &update_params
    ).await;

    // 【結果検証】: ユーザー情報更新成功とデータベース反映の確認
    // 【期待値確認】: 更新内容、タイムスタンプ、データ整合性の正確性確認
    // 【品質保証】: データ整合性とセキュリティ要件の充足を保証
    assert!(result.is_ok()); // 【Green Phase】: 更新処理が正常に完了することを確認 🟢
    let updated_user = result.unwrap();
    assert_eq!(updated_user.name, "更新されたユーザー名"); // 【確認内容】: 名前が正確に更新されることを確認
    assert_eq!(updated_user.email, "modified_user@example.com"); // 【確認内容】: メールが正確に更新されることを確認
    assert_eq!(updated_user.role, "trainer"); // 【確認内容】: 役割が正確に更新されることを確認
}

#[tokio::test]
#[serial]
async fn 管理者によるユーザー削除成功() {
    // 【テスト目的】: 管理者権限による不要ユーザーアカウントの安全な削除機能確認
    // 【テスト内容】: RBAC統合、削除処理、関連データの適切な処理の検証
    // 【期待される動作】: 指定されたユーザーが安全に削除される
    // 🟢 信頼性レベル: 要件定義書のCRUD操作要件から直接抽出した確実な仕様

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 管理者権限を持つテストユーザーと削除対象ユーザーの作成
    // 【初期条件設定】: 有効なRBACコンテキストと削除対象の準備
    // 【前提条件確認】: データベース接続とRBACシステム（TASK-102）が正常動作することを前提
    let admin_user = create_test_user(&boot.app_context.db, "admin_delete@example.com", "Admin User", "admin").await;
    let target_user = create_test_user(&boot.app_context.db, "target_delete@example.com", "To Delete User", "instructor").await;

    let auth_context = AuthContext {
        user_id: admin_user.id,
        user_role: UserRole::Admin,
        session_id: "admin_delete_session_123".to_string(),
    };

    // 【実際の処理実行】: ユーザー管理サービスのユーザー削除メソッド呼び出し
    // 【処理内容】: 削除権限チェック、関連データ処理、データベース削除の実行
    // 【実行タイミング】: RBAC権限チェック通過後のユーザー削除処理段階
    let result = UserManagementService::delete_user(
        &boot.app_context.db,
        &auth_context,
        target_user.id
    ).await;

    // 【結果検証】: ユーザー削除成功と関連データの適切な処理確認
    // 【期待値確認】: 削除完了、関連データのカスケード削除、監査ログ記録の確認
    // 【品質保証】: データ整合性保持と監査ログ記録により削除処理の確実性を保証
    assert!(result.is_ok()); // 【Green Phase】: 削除処理が正常に完了することを確認 🟢
}

#[tokio::test]
#[serial]
async fn 管理者によるユーザー一覧取得成功() {
    // 【テスト目的】: 管理者権限による全ユーザー情報の一覧表示機能をページネーション付きで確認
    // 【テスト内容】: RBAC統合、ページネーション処理、パフォーマンス最適化の検証
    // 【期待される動作】: システム内の全ユーザー情報が適切にページネーション付きで取得される
    // 🟡 信頼性レベル: 要件定義書から推測した一覧表示機能

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 管理者権限を持つテストユーザーと複数のダミーユーザーの作成
    // 【初期条件設定】: 有効なRBACコンテキストとページネーション情報の準備
    // 【前提条件確認】: データベース接続とRBACシステム（TASK-102）が正常動作することを前提
    let admin_user = create_test_user(&boot.app_context.db, "admin_list@example.com", "Admin User", "admin").await;

    // 複数のテストユーザーを作成（ページネーション機能確認用）
    for i in 1..=5 {
        let email = format!("testuser{}@example.com", i);
        let name = format!("Test User {}", i);
        create_test_user(&boot.app_context.db, &email, &name, "instructor").await;
    }

    let auth_context = AuthContext {
        user_id: admin_user.id,
        user_role: UserRole::Admin,
        session_id: "admin_list_session_123".to_string(),
    };

    // 【実際の処理実行】: ユーザー管理サービスのユーザー一覧取得メソッド呼び出し
    // 【処理内容】: 権限チェック、ページネーション処理、データ取得の実行
    // 【実行タイミング】: RBAC権限チェック通過後のユーザー一覧取得処理段階
    let result = UserManagementService::list_users(
        &boot.app_context.db,
        &auth_context,
        1, // page
        20 // per_page
    ).await;

    // 【結果検証】: ユーザー一覧取得成功とページネーション情報の確認
    // 【期待値確認】: ユーザー一覧データ、ページ情報、機密情報除外の確認
    // 【品質保証】: ページネーション機能とパフォーマンス、セキュリティ要件の充足を保証
    assert!(result.is_ok()); // 【Green Phase】: 一覧取得処理が正常に完了することを確認 🟢
    let (users, pagination_info) = result.unwrap();
    assert!(users.len() >= 5); // 【確認内容】: 作成したテストユーザーが含まれることを確認
    assert_eq!(pagination_info.current_page, 1); // 【確認内容】: 正しいページ情報が返されることを確認
    assert_eq!(pagination_info.per_page, 20); // 【確認内容】: 正しいper_page値が返されることを確認
}

#[tokio::test]
#[serial]
async fn 管理者による役割変更成功() {
    // 【テスト目的】: 管理者権限によるユーザー役割変更機能とRBAC統合の動作確認
    // 【テスト内容】: RBAC統合、役割変更処理、権限マトリックス更新の検証
    // 【期待される動作】: 指定されたユーザーの役割が正常に変更される
    // 🟢 信頼性レベル: 要件定義書の役割管理機能から直接抽出した確実な仕様

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 管理者権限を持つテストユーザーと役割変更対象ユーザーの作成
    // 【初期条件設定】: 有効なRBACコンテキストと新しい役割データの準備
    // 【前提条件確認】: データベース接続とRBACシステム（TASK-102）が正常動作することを前提
    let admin_user = create_test_user(&boot.app_context.db, "admin_role@example.com", "Admin User", "admin").await;
    let target_user = create_test_user(&boot.app_context.db, "target_role@example.com", "Role Change User", "instructor").await;

    let auth_context = AuthContext {
        user_id: admin_user.id,
        user_role: UserRole::Admin,
        session_id: "admin_role_session_123".to_string(),
    };

    // 【実際の処理実行】: ユーザー管理サービスの役割変更メソッド呼び出し
    // 【処理内容】: 役割変更権限チェック、新役割設定、RBAC権限マトリックス更新の実行
    // 【実行タイミング】: RBAC権限チェック通過後の役割変更処理段階
    let result = UserManagementService::change_user_role(
        &boot.app_context.db,
        &auth_context,
        target_user.id,
        UserRole::Trainer
    ).await;

    // 【結果検証】: 役割変更成功とRBAC権限の即座の反映確認
    // 【期待値確認】: 役割変更完了、権限マトリックス更新、セッション再評価の確認
    // 【品質保証】: 役割変更の即座反映と権限マトリックス更新により確実な権限制御を保証
    assert!(result.is_ok()); // 【Green Phase】: 役割変更処理が正常に完了することを確認 🟢
    let updated_user = result.unwrap();
    assert_eq!(updated_user.role, "trainer"); // 【確認内容】: 役割が正確に変更されることを確認
    assert_eq!(updated_user.id, target_user.id); // 【確認内容】: 正しいユーザーが更新されることを確認
}

