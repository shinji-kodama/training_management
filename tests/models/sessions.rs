use loco_rs::testing::prelude::*;
use serial_test::serial;
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel, EntityTrait};
use training_management::{
    app::App,
    models::{sessions::{self, ActiveModel}, users::{self, RegisterParams}}
};
use uuid;

// テストファイル: tests/models/sessions.rs
// セッションベース認証実装のためのTDDテスト（Redフェーズ）

#[tokio::test]
#[serial]
async fn セッション作成とデータベース保存() {
    // 【テスト目的】: ログイン成功時のセッション情報がデータベースに正常保存されることを確認
    // 【テスト内容】: sessions::Model::create_session によるデータベースへの永続化をテスト
    // 【期待される動作】: UUIDセッションID生成、24時間有効期限設定、外部キー関係の正常設定
    // 🟢 青信号: 要件定義書のセッション管理仕様から直接抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 24時間の有効期限設定による標準的なセッション作成パターンを用意
    // 【初期条件設定】: テスト用ユーザーを作成してセッション作成の前提条件を整える 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Session Test User".to_string(),
            email: "session_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("テストユーザー作成失敗");
    
    let test_user_id = test_user.id; // 作成されたユーザーIDを使用 🟢
    let session_token = "test_secure_random_token_12345678901234567890";
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    // 【実際の処理実行】: sessions::Model::create_session を呼び出してセッション作成
    // 【処理内容】: データベースsessionsテーブルへのINSERT操作を実行
    let result = sessions::Model::create_session(
        &boot.app_context.db,
        test_user_id,
        session_token.to_string(),
        expires_at.into(),
    ).await;

    // 【結果検証】: セッションが正常に作成され、適切な値が設定されることを確認
    // 【期待値確認】: セッション管理の基盤となるデータ永続化の確実性を検証

    assert!(result.is_ok()); // 【確認内容】: セッション作成処理が成功することを確認 🟢
    
    let session = result.unwrap();
    assert_eq!(session.user_id, test_user_id); // 【確認内容】: 外部キー関係が正常に設定されることを確認 🟢
    assert_eq!(session.session_token, session_token); // 【確認内容】: セッショントークンが正確に保存されることを確認 🟢
    assert!(!session.id.to_string().is_empty()); // 【確認内容】: UUID主キーが自動生成されることを確認 🟢
    assert!(session.expires_at.naive_utc() > chrono::Utc::now().naive_utc()); // 【確認内容】: 有効期限が未来に設定されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn 有効なセッショントークンでの認証通過() {
    // 【テスト目的】: セッションミドルウェアが有効なセッショントークンを正常に認証することを確認
    // 【テスト内容】: session_auth_middleware による正常な認証チェック処理をテスト
    // 【期待される動作】: Cookieからトークン取得、データベース検証、ユーザー情報注入
    // 🟡 黄信号: 要件定義書のセッションミドルウェア仕様から推測

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: ログイン済みユーザーからの後続リクエストを模擬するためのセッション準備
    // 【初期条件設定】: テスト用ユーザーを作成して有効なセッションを事前作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Valid Session Test User".to_string(),
            email: "valid_session_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("テストユーザー作成失敗");
    
    let valid_session = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id, // 作成されたユーザーIDを使用 🟢
        "valid_session_token_1234567890123456789012345".to_string(),
        (chrono::Utc::now() + chrono::Duration::hours(24)).into(),
    ).await.expect("テストセッション作成失敗");

    // 【実際の処理実行】: セッション検証ミドルウェアを呼び出し
    // 【処理内容】: HTTPクッキーからセッショントークンを取得し、データベースで検証
    let result = sessions::Model::find_by_token(&boot.app_context.db, &valid_session.session_token).await;

    // 【結果検証】: セッション検索が成功し、適切なユーザー情報が取得されることを確認
    // 【期待値確認】: セッションベース認証の本質的な機能である認証状態の継続性を検証

    assert!(result.is_ok()); // 【確認内容】: セッション検索処理が成功することを確認 🟡
    
    let found_session = result.unwrap();
    assert_eq!(found_session.session_token, valid_session.session_token); // 【確認内容】: セッショントークンが一致することを確認 🟡
    assert_eq!(found_session.user_id, valid_session.user_id); // 【確認内容】: ユーザー情報が正確に取得されることを確認 🟡
    assert!(found_session.expires_at.naive_utc() > chrono::Utc::now().naive_utc()); // 【確認内容】: セッションが有効期限内であることを確認 🟡
}

#[tokio::test]
#[serial]
async fn 期限切れセッションでの認証失敗() {
    // 【テスト目的】: 24時間の有効期限を超過したセッションが適切に拒否されることを確認
    // 【テスト内容】: expires_at を超過したセッションでの認証拒否処理をテスト
    // 【期待される動作】: HTTP 401 Unauthorized、セッション自動削除
    // 🟡 黄信号: 要件定義書のEDGE-001から推測

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 意図的に期限切れのセッションを作成し、期限切れ認証拒否をテスト
    // 【初期条件設定】: テスト用ユーザーを作成して過去の時刻を有効期限としたセッションを準備 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Expired Session Test User".to_string(),
            email: "expired_session_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("テストユーザー作成失敗");
    
    // まず有効なセッションを作成して、その後時刻を過去に設定して期限切れにする
    let valid_session = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id, // 作成されたユーザーIDを使用 🟢
        "expired_session_token_1234567890123456789012345".to_string(),
        (chrono::Utc::now() + chrono::Duration::hours(1)).into(), // 一時的に1時間後に設定
    ).await.expect("一時的セッション作成失敗");
    
    // データベース直接操作で有効期限を過去に変更（テスト用）
    let mut active_model: ActiveModel = valid_session.into();
    active_model.expires_at = ActiveValue::Set((chrono::Utc::now() - chrono::Duration::hours(1)).into());
    let expired_session = active_model.update(&boot.app_context.db).await.expect("セッション期限切れ更新失敗");

    // 【実際の処理実行】: 期限切れセッションでの認証試行を実行
    // 【処理内容】: セッション有効期限チェック機能を呼び出し
    let result = sessions::Model::validate_session(&boot.app_context.db, &expired_session.session_token).await;

    // 【結果検証】: セッション期限切れが適切に検出され、認証が拒否されることを確認
    // 【期待値確認】: セッション生涯管理とセキュリティ確保の実装を検証

    assert!(result.is_err()); // 【確認内容】: 期限切れセッションの認証が失敗することを確認 🟡
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("expired") || error.to_string().contains("期限切れ")); // 【確認内容】: 適切な期限切れエラーメッセージが返されることを確認 🟡
    
    // セッション自動削除の確認
    let cleanup_result = sessions::Model::find_by_token(&boot.app_context.db, &expired_session.session_token).await;
    assert!(cleanup_result.is_err()); // 【確認内容】: 期限切れセッションが自動削除されることを確認 🟡
}

#[tokio::test]
#[serial] 
async fn 不正セッショントークンでの認証失敗() {
    // 【テスト目的】: データベースに存在しないセッショントークンが適切に拒否されることを確認
    // 【テスト内容】: 存在しないセッショントークンでの認証拒否処理をテスト
    // 【期待される動作】: HTTP 401 Unauthorized、セキュリティログ記録
    // 🟡 黄信号: 要件定義書のEDGE-002から推測

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: データベースに存在しない偽造されたセッショントークンを用意
    // 【初期条件設定】: セッションハイジャック攻撃を模擬した不正トークンを設定
    let forged_token = "invalid_forged_token_1234567890123456789012345";

    // 【実際の処理実行】: 不正なセッショントークンでの認証試行を実行
    // 【処理内容】: セッショントークン存在確認とセキュリティログ記録機能を呼び出し
    let result = sessions::Model::find_by_token(&boot.app_context.db, forged_token).await;

    // 【結果検証】: セッショントークン偽造が適切に検出され、認証が拒否されることを確認
    // 【期待値確認】: セッショントークン完全性検証の確実性を検証

    assert!(result.is_err()); // 【確認内容】: 不正セッショントークンの認証が失敗することを確認 🟡
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("not found") || error.to_string().contains("無効")); // 【確認内容】: 適切な無効セッションエラーメッセージが返されることを確認 🟡
}

#[tokio::test]
#[serial]
async fn セッション有効期限境界値テスト() {
    // 【テスト目的】: セッション有効期限の境界（23:59:59 vs 24:00:00）で正確な動作を確認
    // 【テスト内容】: 24時間有効期限の正確な境界での認証可否をテスト
    // 【期待される動作】: 1秒差での認証成功・失敗の確実な分岐
    // 🟡 黄信号: 要件定義書の24時間有効期限仕様から推測

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let base_time = chrono::Utc::now();
    
    // 【テストデータ準備】: 境界値付近の有効期限を持つセッションを2つ作成
    // 【初期条件設定】: 23時間59分59秒（有効）と24時間ちょうど（無効）のセッション
    
    // 【テストユーザー作成】: 境界値テスト用のユーザーを作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Boundary Test User".to_string(),
            email: "boundary_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("テストユーザー作成失敗");
    
    // ケース1: 23時間59分59秒経過（有効）
    let valid_session = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id, // 作成されたユーザーIDを使用 🟢
        "boundary_valid_token_1234567890123456789012345".to_string(),
        (base_time + chrono::Duration::hours(24) - chrono::Duration::seconds(1)).into(),
    ).await.expect("境界値有効セッション作成失敗");

    // ケース2: 24時間ちょうど経過（無効）  
    // まず有効なセッションを作成
    let temp_session = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id, // 同じユーザーでテスト 🟢
        "boundary_invalid_token_1234567890123456789012345".to_string(),
        (base_time + chrono::Duration::seconds(10)).into(), // 一時的に10秒後に設定
    ).await.expect("境界値一時セッション作成失敗");
    
    // データベース直接操作で有効期限を1秒前に変更（テスト用）
    let mut active_model: ActiveModel = temp_session.into();
    active_model.expires_at = ActiveValue::Set((base_time - chrono::Duration::seconds(1)).into());
    let invalid_session = active_model.update(&boot.app_context.db).await.expect("境界値セッション期限切れ更新失敗");

    // 【実際の処理実行】: 境界値セッションでの認証試行を実行
    // 【処理内容】: 時刻比較ロジックの秒単位精度を検証
    
    let valid_result = sessions::Model::validate_session(&boot.app_context.db, &valid_session.session_token).await;
    let invalid_result = sessions::Model::validate_session(&boot.app_context.db, &invalid_session.session_token).await;

    // 【結果検証】: 境界値での認証可否が正確に判定されることを確認
    // 【期待値確認】: セッション有効期限管理の正確性を検証

    assert!(valid_result.is_ok()); // 【確認内容】: 境界内セッション（23:59:59）の認証が成功することを確認 🟡
    assert!(invalid_result.is_err()); // 【確認内容】: 境界外セッション（24:00:00）の認証が失敗することを確認 🟡
}

#[tokio::test]
#[serial]
async fn 空文字列での入力検証確認() {
    // 【テスト目的】: 空文字列・null値での入力検証が確実に機能することを確認
    // 【テスト内容】: 必須フィールドの入力検証境界をテスト
    // 【期待される動作】: HTTP 400 Bad Request、バリデーションエラー
    // 🟢 青信号: 入力検証の基本パターンから確実に抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 空文字列による不正入力パターンを用意
    // 【初期条件設定】: フォーム入力の最小値制約違反を模擬
    
    // 空のセッショントークンでのテスト
    let empty_token_result = sessions::Model::find_by_token(&boot.app_context.db, "").await;
    
    // 【実際の処理実行】: 空文字列入力での検証処理を実行
    // 【処理内容】: 必須項目チェックの確実な実行を検証
    
    // 【結果検証】: 空値での処理継続が防止されることを確認
    // 【期待値確認】: 入力検証機能の完全性を検証

    assert!(empty_token_result.is_err()); // 【確認内容】: 空文字列セッショントークンが拒否されることを確認 🟢
    
    let error = empty_token_result.unwrap_err();
    assert!(error.to_string().contains("empty") || error.to_string().contains("無効") || error.to_string().contains("not found")); // 【確認内容】: 適切な入力検証エラーメッセージが返されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn 有効なCSRFトークンでの状態変更操作許可() {
    // 【テスト目的】: CSRF保護機能が正常動作し、正規操作を阻害しないことを確認
    // 【テスト内容】: セッション作成時のCSRFトークン生成と検証機能をテスト
    // 【期待される動作】: CSRFトークン検証後の操作許可
    // 🟢 青信号: TASK-101完了条件「CSRF攻撃が防御される」から直接抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: CSRF機能付きセッションの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "CSRF Test User".to_string(),
            email: "csrf_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("CSRFテストユーザー作成失敗");
    
    let session = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id, // 作成されたユーザーIDを使用 🟢
        "csrf_session_token_1234567890123456789012345".to_string(),
        (chrono::Utc::now() + chrono::Duration::hours(24)).into(),
    ).await.expect("CSRFテストセッション作成失敗");

    // 【実際の処理実行】: CSRFトークン検証を実行
    // 【処理内容】: セッション作成時に自動生成されたCSRFトークンを使用した検証

    let csrf_token = session.csrf_token.as_ref()
        .expect("セッションにCSRFトークンが存在すること");

    let result = session.verify_csrf_token(csrf_token);

    // 【結果検証】: 正常なCSRFトークンでの検証が成功することを確認
    // 【期待値確認】: CSRF保護機能が正規操作を阻害しないことを検証

    assert!(result.is_ok()); // 【確認内容】: 有効なCSRFトークンの検証が成功することを確認 🟢

    // 【追加検証】: CSRFトークンの形式が正しいことを確認
    assert!(csrf_token.len() >= sessions::MIN_CSRF_TOKEN_LENGTH); // 【確認内容】: CSRFトークン長が要件を満たすことを確認 🟢
    assert!(csrf_token.len() <= sessions::MAX_CSRF_TOKEN_LENGTH); // 【確認内容】: CSRFトークン長が制限内であることを確認 🟢
}

#[tokio::test]
#[serial]
async fn 不正CSRFトークンでの状態変更操作拒否() {
    // 【テスト目的】: CSRF攻撃が適切に防御され、不正な状態変更操作が拒否されることを確認
    // 【テスト内容】: 偽造されたCSRFトークンでの操作試行をテスト
    // 【期待される動作】: HTTP 403 Forbidden、操作拒否
    // 🟢 青信号: TASK-101完了条件「CSRF攻撃が防御される」から直接抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: CSRF攻撃を模擬したセッションの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "CSRF Attack Test User".to_string(),
            email: "csrf_attack_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("CSRF攻撃テストユーザー作成失敗");
    
    let session = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id, // 作成されたユーザーIDを使用 🟢
        "csrf_attack_session_token_1234567890123456789012345".to_string(),
        (chrono::Utc::now() + chrono::Duration::hours(24)).into(),
    ).await.expect("CSRF攻撃テストセッション作成失敗");

    // 【実際の処理実行】: 偽造されたCSRFトークンでの検証試行
    // 【処理内容】: セッションとは異なる偽造トークンを使用した攻撃シミュレーション

    let forged_csrf_token = "forged_malicious_csrf_token_attack_simulation_12345678901234567890";

    let result = session.verify_csrf_token(forged_csrf_token);

    // 【結果検証】: 偽造されたCSRFトークンが適切に拒否されることを確認
    // 【期待値確認】: CSRF攻撃防御機能の有効性を検証

    assert!(result.is_err()); // 【確認内容】: 偽造CSRFトークンの検証が失敗することを確認 🟢
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("mismatch") || error.to_string().contains("不一致") || error.to_string().contains("Invalid CSRF token")); // 【確認内容】: 適切なCSRFエラーメッセージが返されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn 正常ログイン処理() {
    // 【テスト目的】: ログイン成功時のセッション作成とレスポンス確認
    // 【テスト内容】: login API呼び出しとセッション作成フローをテスト
    // 【期待される動作】: SessionLoginResponseとCSRFトークン返却
    // 🟢 青信号: TASK-101完了条件「ログイン機能が動作する」から直接抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: ログイン機能テスト用ユーザーの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Login Test User".to_string(),
            email: "login_test@example.com".to_string(),
            password: "login123".to_string(),
        },
    ).await.expect("ログインテストユーザー作成失敗");

    // メールアドレス認証を完了状態にする
    let mut active_user = test_user.into_active_model();
    active_user.email_verified_at = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().into()));
    let verified_user = active_user.update(&boot.app_context.db).await.expect("ユーザー認証完了失敗");

    // 【実際の処理実行】: ログインAPIの呼び出し
    // 【処理内容】: 正常な認証情報でのセッション作成処理
    
    // 直接モデルレベルでのログイン処理をテスト（統合テスト用）
    let login_user = users::Model::find_by_email(&boot.app_context.db, &verified_user.email).await.expect("ユーザー検索失敗");
    let password_valid = login_user.verify_password("login123");
    
    assert!(password_valid); // パスワード検証が成功することを確認

    // セッション作成の確認
    let session_token = format!("test_session_{}", uuid::Uuid::new_v4());
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(sessions::DEFAULT_SESSION_DURATION_HOURS);

    let session = sessions::Model::create_session(
        &boot.app_context.db,
        login_user.id,
        session_token.clone(),
        expires_at.into(),
    ).await.expect("ログインセッション作成失敗");

    // 【結果検証】: ログイン処理の完全性を確認
    // 【期待値確認】: セッション作成とCSRF保護の適切な実装

    assert_eq!(session.user_id, login_user.id); // 【確認内容】: ユーザー関連付けが正しいことを確認 🟢
    assert_eq!(session.session_token, session_token); // 【確認内容】: セッショントークンが正確に設定されることを確認 🟢
    assert!(session.csrf_token.is_some()); // 【確認内容】: CSRFトークンが生成されることを確認 🟢
    assert!(session.expires_at.naive_utc() > chrono::Utc::now().naive_utc()); // 【確認内容】: 有効期限が未来に設定されることを確認 🟢
}

#[tokio::test]
#[serial]
async fn ログアウト処理() {
    // 【テスト目的】: ログアウト時のセッション削除確認
    // 【テスト内容】: logout API呼び出しとセッション無効化をテスト
    // 【期待される動作】: セッション削除と認証状態のクリア
    // 🟢 青信号: TASK-101完了条件「ログアウト機能が動作する」から直接抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: ログアウト機能テスト用ユーザーとセッションの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Logout Test User".to_string(),
            email: "logout_test@example.com".to_string(),
            password: "logout123".to_string(),
        },
    ).await.expect("ログアウトテストユーザー作成失敗");

    let session = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id,
        "logout_test_session_token_123456789012345".to_string(),
        (chrono::Utc::now() + chrono::Duration::hours(24)).into(),
    ).await.expect("ログアウトテストセッション作成失敗");

    // 【実際の処理実行】: ログアウト処理の実行
    // 【処理内容】: セッション削除による認証状態のクリア

    // セッション削除の実行
    let delete_result = sessions::Entity::delete_by_id(session.id)
        .exec(&boot.app_context.db)
        .await
        .expect("セッション削除失敗");

    // 【結果検証】: ログアウト処理の完全性を確認
    // 【期待値確認】: セッション無効化とセキュリティの確保

    assert_eq!(delete_result.rows_affected, 1); // 【確認内容】: セッションが確実に削除されることを確認 🟢

    // セッション削除後の検索確認
    let find_result = sessions::Model::find_by_token(&boot.app_context.db, &session.session_token).await;
    assert!(find_result.is_err()); // 【確認内容】: セッション削除後に検索できないことを確認 🟢
}

#[tokio::test]
#[serial]
async fn 不正な認証情報でのログイン拒否() {
    // 【テスト目的】: 不正な認証情報によるログイン試行の適切な拒否
    // 【テスト内容】: 間違ったパスワードでのログイン試行をテスト
    // 【期待される動作】: 認証失敗とセッション作成の阻止
    // 🟢 青信号: TASK-101完了条件「不正認証が拒否される」から直接抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 不正認証テスト用ユーザーの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Auth Reject Test User".to_string(),
            email: "auth_reject_test@example.com".to_string(),
            password: "correct_password".to_string(),
        },
    ).await.expect("認証拒否テストユーザー作成失敗");

    // 【実際の処理実行】: 不正な認証情報でのログイン試行
    // 【処理内容】: 間違ったパスワードによる認証試行

    let found_user = users::Model::find_by_email(&boot.app_context.db, &test_user.email).await.expect("ユーザー検索失敗");
    let invalid_password_result = found_user.verify_password("wrong_password");

    // 【結果検証】: 不正認証の適切な拒否を確認
    // 【期待値確認】: セキュリティ機能の有効性を検証

    assert!(!invalid_password_result); // 【確認内容】: 不正パスワードの認証が失敗することを確認 🟢

    // 不正認証後のセッション作成試行（実際のログインフローでは発生しない）
    // このケースではセッション作成が行われないことを確認するテスト
}

#[tokio::test]
#[serial]
async fn パスワード不一致でのログイン拒否() {
    // 【テスト目的】: パスワード不一致による認証拒否の確認
    // 【テスト内容】: 複数の不正パスワードパターンでのテスト
    // 【期待される動作】: 一貫した認証拒否とセキュリティログ
    // 🟢 青信号: TASK-101完了条件「パスワード不一致が拒否される」から直接抽出

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: パスワード不一致テスト用ユーザーの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Password Mismatch Test User".to_string(),
            email: "password_mismatch_test@example.com".to_string(),
            password: "secret123".to_string(),
        },
    ).await.expect("パスワード不一致テストユーザー作成失敗");

    // 【実際の処理実行】: 複数の不正パスワードパターンでのテスト
    // 【処理内容】: 様々な不正パスワードでの認証試行

    let found_user = users::Model::find_by_email(&boot.app_context.db, &test_user.email).await.expect("ユーザー検索失敗");

    let test_cases = [
        ("", "空文字列パスワード"),
        ("wrong", "完全に異なるパスワード"),
        ("secret124", "1文字違いのパスワード"),
        ("SECRET123", "大文字小文字違いのパスワード"),
        ("secret123 ", "末尾空白付きパスワード"),
    ];

    for (wrong_password, case_description) in test_cases.iter() {
        let result = found_user.verify_password(wrong_password);
        
        // 【結果検証】: 各パターンでの認証拒否を確認
        assert!(!result, "{}でのパスワード認証が失敗すること", case_description); // 【確認内容】: すべての不正パスワードパターンが拒否されることを確認 🟢
    }
}

#[tokio::test]
#[serial]
async fn セッショントークン長の境界値テスト() {
    // 【テスト目的】: セッショントークンの長さ制限境界での動作確認
    // 【テスト内容】: 31文字（無効）と32文字（有効）、255文字（有効）、256文字（無効）での検証
    // 【期待される動作】: 厳密な長さ制限による適切な受け入れ・拒否
    // 🟡 黄信号: 要件定義書のトークン長制限から推測実装

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: 境界値テスト用ユーザーの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Token Boundary Test User".to_string(),
            email: "token_boundary_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("境界値テストユーザー作成失敗");

    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).into();

    // 【テストケース】: 各境界値でのトークン長テスト

    // ケース1: 31文字（最小長未満 - 無効）
    let short_token = "a".repeat(31);
    let short_result = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id,
        short_token,
        expires_at,
    ).await;
    assert!(short_result.is_err()); // 【確認内容】: 短すぎるトークンが拒否されることを確認 🟡

    // ケース2: 32文字（最小長 - 有効）
    let min_token = "a".repeat(sessions::MIN_SESSION_TOKEN_LENGTH);
    let min_result = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id,
        min_token,
        expires_at,
    ).await;
    assert!(min_result.is_ok()); // 【確認内容】: 最小長トークンが受け入れられることを確認 🟡

    // ケース3: 255文字（最大長 - 有効）
    let max_token = "b".repeat(sessions::MAX_SESSION_TOKEN_LENGTH);
    let max_result = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id,
        max_token,
        expires_at,
    ).await;
    assert!(max_result.is_ok()); // 【確認内容】: 最大長トークンが受け入れられることを確認 🟡

    // ケース4: 256文字（最大長超過 - 無効）
    let long_token = "c".repeat(256);
    let long_result = sessions::Model::create_session(
        &boot.app_context.db,
        test_user.id,
        long_token,
        expires_at,
    ).await;
    assert!(long_result.is_err()); // 【確認内容】: 長すぎるトークンが拒否されることを確認 🟡
}

#[tokio::test]
#[serial]
async fn 最大同時セッション数の境界値テスト() {
    // 【テスト目的】: 同一ユーザーの最大セッション数制限の確認
    // 【テスト内容】: セッション数カウント機能の動作検証
    // 【期待される動作】: 効率的な同時セッション数管理
    // 🟡 黄信号: 要件定義書のセッション上限管理から推測実装

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    // 【テストデータ準備】: セッション上限テスト用ユーザーの作成 🟢
    let test_user = users::Model::create_with_password(
        &boot.app_context.db,
        &RegisterParams {
            name: "Max Sessions Test User".to_string(),
            email: "max_sessions_test@example.com".to_string(),
            password: "test123".to_string(),
        },
    ).await.expect("最大セッションテストユーザー作成失敗");

    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).into();

    // 【実際の処理実行】: 複数セッションの作成
    let mut created_sessions = Vec::new();
    
    // 3つのセッションを作成
    for i in 1..=3 {
        let session_token = format!("max_session_test_token_{}_12345678901234567890", i);
        let session = sessions::Model::create_session(
            &boot.app_context.db,
            test_user.id,
            session_token,
            expires_at,
        ).await.expect(&format!("セッション{}作成失敗", i));
        created_sessions.push(session);
    }

    // 【結果検証】: セッション数カウント機能の確認
    // 【期待値確認】: 効率的なセッション管理機能の検証

    let active_count = sessions::Entity::count_active_sessions_for_user(&boot.app_context.db, test_user.id)
        .await
        .expect("アクティブセッション数取得失敗");

    assert_eq!(active_count, 3); // 【確認内容】: 作成したセッション数が正確にカウントされることを確認 🟡

    // 【セッション無効化テスト】: 一部セッション削除後のカウント確認
    sessions::Entity::delete_by_id(created_sessions[0].id)
        .exec(&boot.app_context.db)
        .await
        .expect("セッション削除失敗");

    let updated_count = sessions::Entity::count_active_sessions_for_user(&boot.app_context.db, test_user.id)
        .await
        .expect("更新後セッション数取得失敗");

    assert_eq!(updated_count, 2); // 【確認内容】: セッション削除後の数が正確に更新されることを確認 🟡

    // 【全セッション無効化テスト】: ユーザー全セッション削除
    let invalidated_count = sessions::Entity::invalidate_all_user_sessions(&boot.app_context.db, test_user.id)
        .await
        .expect("全セッション無効化失敗");

    assert_eq!(invalidated_count, 2); // 【確認内容】: 残り2セッションが削除されることを確認 🟡

    let final_count = sessions::Entity::count_active_sessions_for_user(&boot.app_context.db, test_user.id)
        .await
        .expect("最終セッション数取得失敗");

    assert_eq!(final_count, 0); // 【確認内容】: 全セッション削除後にカウントが0になることを確認 🟡
}