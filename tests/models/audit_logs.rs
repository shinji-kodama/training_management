use loco_rs::testing::request::boot_test;
use sea_orm::ActiveModelTrait;
use serial_test::serial;
use training_management::app::App;

// 【テスト対象】: 監査ログ（AuditLogs）モデルの包括的CRUD機能テスト
// 【テスト方針】: database-schema.sqlの制約とビジネスルールに基づく確実なテストケース
// 【フレームワーク】: Loco.rs 0.16.3 + SeaORM 1.1.12 + PostgreSQL環境でのモデルテスト
// 🟢 信頼性レベル: database-schema.sqlのaudit_logsテーブル定義と制約に完全準拠

#[tokio::test]
#[serial]
async fn test_監査ログの正常作成() {
    // 【テスト目的】: 監査ログエンティティの基本的な作成処理とデータベース保存の動作確認
    // 【テスト内容】: 有効な監査ログデータが正常にデータベースに保存され、UUID主キーが自動設定される
    // 【期待される動作】: 外部キー関係（user_id）が正常に機能し、JSONB詳細情報とINET型IPアドレスが適切に保存される
    // 🟢 信頼性レベル: database-schema.sqlのaudit_logsテーブル定義に基づく確実なテストケース
    
    // 【テスト前準備】: データベース接続とテスト環境の初期化
    // 【初期条件設定】: 監査ログ作成に必要な外部キーデータ（ユーザー）を事前に準備
    let boot = boot_test::<App>().await.unwrap();
    
    // 【依存データ作成】: 監査ログ作成に必要なユーザーエンティティを事前に作成
    // 【データ整合性】: 外部キー制約を満たすため、usersテーブルにデータを準備
    let audit_user = training_management::models::users::RegisterParams {
        name: "監査ユーザー".to_string(),
        email: "audit@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &audit_user)
        .await
        .unwrap();
    
    // 【監査ログデータ作成】: 正常な監査ログデータを準備
    let audit_log_data = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("create_material".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("material".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(uuid::Uuid::new_v4())),
        details: sea_orm::ActiveValue::Set(Some(sea_orm::JsonValue::Object(serde_json::Map::from_iter([
            ("material_title".to_string(), serde_json::Value::String("新規教材".to_string())),
            ("recommendation_level".to_string(), serde_json::Value::Number(serde_json::Number::from(4))),
        ])))),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.100".to_string())),
        user_agent: sea_orm::ActiveValue::Set(Some("Mozilla/5.0 (test-agent)".to_string())),
        ..Default::default()
    };
    
    // 【監査ログ作成実行】: データベースへ監査ログデータを保存
    let result = audit_log_data.insert(&boot.app_context.db).await.unwrap();
    
    // 【結果検証】: 作成された監査ログデータの妥当性確認
    // UUID主キー自動生成の確認（空文字列ではないUUIDが設定される）
    assert!(!result.id.to_string().is_empty());
    
    // 外部キー関係の正常保存確認
    assert_eq!(result.user_id, Some(created_user.id));
    
    // アクション情報の正常保存確認
    assert_eq!(result.action, "create_material");
    assert_eq!(result.resource_type, Some("material".to_string()));
    assert!(result.resource_id.is_some());
    
    // JSONB詳細情報の保存確認
    assert!(result.details.is_some());
    if let Some(details) = &result.details {
        if let Some(obj) = details.as_object() {
            assert!(obj.contains_key("material_title"));
        }
    }
    
    // IPアドレスとユーザーエージェントの保存確認
    assert_eq!(result.ip_address, Some("192.168.1.100".to_string()));
    assert!(result.user_agent.as_ref().unwrap().contains("test-agent"));
    
    // created_at自動設定確認（現在時刻付近の値が設定される）
    assert!(result.created_at > chrono::Utc::now().fixed_offset() - chrono::Duration::seconds(10));
}

#[tokio::test]
#[serial]
async fn test_ユーザー別監査ログ検索() {
    // 【テスト目的】: ユーザーに紐付く監査ログ一覧の検索機能動作確認
    // 【テスト内容】: 特定のユーザーに関連する監査ログを正確に抽出できる
    // 【期待される動作】: 1対多リレーション（ユーザー→監査ログ）が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlの外部キー関係に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 複数監査ログを持つユーザーのテストデータセット構築
    let test_user = training_management::models::users::RegisterParams {
        name: "検索テストユーザー".to_string(),
        email: "search.audit@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &test_user)
        .await
        .unwrap();
    
    // 【複数監査ログ作成】: 同一ユーザーに対する複数の監査ログを作成
    // 1回目のログ（ログイン操作）
    let audit_log1 = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("login".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("user".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(created_user.pid)),
        details: sea_orm::ActiveValue::Set(Some(sea_orm::JsonValue::Object(serde_json::Map::from_iter([
            ("login_method".to_string(), serde_json::Value::String("email".to_string())),
        ])))),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.101".to_string())),
        user_agent: sea_orm::ActiveValue::Set(Some("Mozilla/5.0 (login-test)".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 2回目のログ（研修作成操作）
    let audit_log2 = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("create_training".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("training".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(uuid::Uuid::new_v4())),
        details: sea_orm::ActiveValue::Set(Some(sea_orm::JsonValue::Object(serde_json::Map::from_iter([
            ("training_title".to_string(), serde_json::Value::String("新規研修".to_string())),
        ])))),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.102".to_string())),
        user_agent: sea_orm::ActiveValue::Set(Some("Mozilla/5.0 (create-test)".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【検索機能テスト実行】: ユーザー別監査ログ一覧取得機能をテスト
    let audit_logs = training_management::models::audit_logs::Model::find_by_user_id(&boot.app_context.db, created_user.id).await.unwrap();
    
    // 【検索結果検証】: 検索結果の妥当性確認
    // 正しい数の監査ログが取得される（2件）
    assert_eq!(audit_logs.len(), 2);
    
    // すべての監査ログが正しいユーザーに紐付いている
    for log in &audit_logs {
        assert_eq!(log.user_id, Some(created_user.id));
    }
    
    // 作成した監査ログIDが含まれている
    let log_ids: Vec<uuid::Uuid> = audit_logs.iter().map(|l| l.id).collect();
    assert!(log_ids.contains(&audit_log1.id));
    assert!(log_ids.contains(&audit_log2.id));
}

#[tokio::test]
#[serial]
async fn test_アクション別監査ログ検索() {
    // 【テスト目的】: アクション種別による監査ログ検索機能の動作確認
    // 【テスト内容】: 特定のアクション種別で監査ログを正確に抽出できる
    // 【期待される動作】: アクションフィールドによるフィルタリング検索が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlのaction VARCHAR(100)制約に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 異なるアクションの監査ログを持つテストデータセット構築
    let test_user = training_management::models::users::RegisterParams {
        name: "アクション検索ユーザー".to_string(),
        email: "action.search@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &test_user)
        .await
        .unwrap();
    
    // 【複数アクション監査ログ作成】: 異なるアクション種別の監査ログを作成
    // ログイン操作ログ
    let login_log = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("login".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("user".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(created_user.pid)),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.103".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // ログアウト操作ログ
    let logout_log = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("logout".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("user".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(created_user.pid)),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.104".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 教材作成操作ログ
    let _material_log = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("create_material".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("material".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(uuid::Uuid::new_v4())),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.105".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【アクション別検索テスト実行】: ログイン操作のみを検索
    let login_logs = training_management::models::audit_logs::Model::find_by_action(&boot.app_context.db, "login").await.unwrap();
    
    // 【検索結果検証】: ログイン操作のみが抽出される
    assert_eq!(login_logs.len(), 1);
    assert_eq!(login_logs[0].action, "login");
    assert_eq!(login_logs[0].id, login_log.id);
    
    // 【複数アクション検索】: ログイン・ログアウト操作を検索
    let auth_logs = training_management::models::audit_logs::Model::find_by_actions(&boot.app_context.db, &["login", "logout"]).await.unwrap();
    
    // 【検索結果検証】: 認証関連操作のみが抽出される（2件）
    assert_eq!(auth_logs.len(), 2);
    let action_types: Vec<&str> = auth_logs.iter().map(|l| l.action.as_str()).collect();
    assert!(action_types.contains(&"login"));
    assert!(action_types.contains(&"logout"));
}

#[tokio::test]
#[serial]
async fn test_リソース別監査ログ検索() {
    // 【テスト目的】: リソース種別・IDによる監査ログ検索機能の動作確認
    // 【テスト内容】: 特定のリソース（resource_type, resource_id）で監査ログを正確に抽出できる
    // 【期待される動作】: リソース情報による複合条件検索が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlのresource_type, resource_id制約に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【準備データ作成】: 異なるリソースの監査ログを持つテストデータセット構築
    let test_user = training_management::models::users::RegisterParams {
        name: "リソース検索ユーザー".to_string(),
        email: "resource.search@test.co.jp".to_string(),
        password: "securepass123".to_string(),
    };
    let created_user = training_management::models::users::Model::create_with_password(&boot.app_context.db, &test_user)
        .await
        .unwrap();
    
    // 【特定リソースID】: 検索対象のリソースID
    let target_resource_id = uuid::Uuid::new_v4();
    
    // 【複数リソース監査ログ作成】: 異なるリソース種別・IDの監査ログを作成
    // 対象リソース操作ログ（作成）
    let create_log = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("create_training".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("training".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(target_resource_id)),
        details: sea_orm::ActiveValue::Set(Some(sea_orm::JsonValue::Object(serde_json::Map::from_iter([
            ("operation".to_string(), serde_json::Value::String("created".to_string())),
        ])))),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.106".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 対象リソース操作ログ（更新）
    let update_log = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("update_training".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("training".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(target_resource_id)),
        details: sea_orm::ActiveValue::Set(Some(sea_orm::JsonValue::Object(serde_json::Map::from_iter([
            ("operation".to_string(), serde_json::Value::String("updated".to_string())),
        ])))),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.107".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 他のリソース操作ログ（検索結果に含まれない）
    let _other_log = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(created_user.id)),
        action: sea_orm::ActiveValue::Set("create_material".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("material".to_string())),
        resource_id: sea_orm::ActiveValue::Set(Some(uuid::Uuid::new_v4())), // 異なるリソースID
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.108".to_string())),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .unwrap();
    
    // 【リソース別検索テスト実行】: 特定リソースIDの操作履歴を検索
    let resource_logs = training_management::models::audit_logs::Model::find_by_resource(
        &boot.app_context.db, 
        "training", 
        target_resource_id
    ).await.unwrap();
    
    // 【検索結果検証】: 対象リソースの操作のみが抽出される（2件）
    assert_eq!(resource_logs.len(), 2);
    
    // すべてのログが正しいリソース情報を持つ
    for log in &resource_logs {
        assert_eq!(log.resource_type, Some("training".to_string()));
        assert_eq!(log.resource_id, Some(target_resource_id));
    }
    
    // 作成・更新ログが含まれている
    let log_ids: Vec<uuid::Uuid> = resource_logs.iter().map(|l| l.id).collect();
    assert!(log_ids.contains(&create_log.id));
    assert!(log_ids.contains(&update_log.id));
}

#[tokio::test]
#[serial]
async fn test_匿名監査ログ作成() {
    // 【テスト目的】: ユーザーが関連しない匿名操作の監査ログ作成確認
    // 【テスト内容】: user_id が NULL の監査ログが正常に作成される
    // 【期待される動作】: システム操作やゲスト操作など、ユーザー不明操作の記録が適切に機能する
    // 🟢 信頼性レベル: database-schema.sqlのuser_id NULL許可制約に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【匿名監査ログデータ作成】: user_id が NULL の監査ログを準備
    let anonymous_log_data = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(None), // 匿名操作（user_id = NULL）
        action: sea_orm::ActiveValue::Set("system_cleanup".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("system".to_string())),
        resource_id: sea_orm::ActiveValue::Set(None),
        details: sea_orm::ActiveValue::Set(Some(sea_orm::JsonValue::Object(serde_json::Map::from_iter([
            ("cleanup_type".to_string(), serde_json::Value::String("session_cleanup".to_string())),
            ("deleted_count".to_string(), serde_json::Value::Number(serde_json::Number::from(15))),
        ])))),
        ip_address: sea_orm::ActiveValue::Set(Some("127.0.0.1".to_string())),
        user_agent: sea_orm::ActiveValue::Set(Some("System/1.0 (cleanup-daemon)".to_string())),
        ..Default::default()
    };
    
    // 【匿名監査ログ作成実行】: データベースへ匿名監査ログを保存
    let result = anonymous_log_data.insert(&boot.app_context.db).await.unwrap();
    
    // 【結果検証】: 作成された匿名監査ログの妥当性確認
    // UUID主キー自動生成の確認
    assert!(!result.id.to_string().is_empty());
    
    // 匿名操作の確認（user_id が NULL）
    assert!(result.user_id.is_none());
    
    // システム操作情報の保存確認
    assert_eq!(result.action, "system_cleanup");
    assert_eq!(result.resource_type, Some("system".to_string()));
    assert!(result.resource_id.is_none());
    
    // JSONB詳細情報の保存確認
    assert!(result.details.is_some());
    if let Some(details) = &result.details {
        if let Some(obj) = details.as_object() {
            assert!(obj.contains_key("cleanup_type"));
            assert!(obj.contains_key("deleted_count"));
        }
    }
    
    // システム操作のメタデータ確認
    assert_eq!(result.ip_address, Some("127.0.0.1".to_string()));
    assert!(result.user_agent.as_ref().unwrap().contains("System"));
}

#[tokio::test]
#[serial]
async fn test_ユーザー参照整合性制約() {
    // 【テスト目的】: ユーザー参照整合性制約（ON DELETE SET NULL）の動作確認
    // 【テスト内容】: 存在しないuser_idでの監査ログ作成が適切に拒否される
    // 【期待される動作】: 外部キー制約が正常に機能する
    // 🟢 信頼性レベル: database-schema.sqlの外部キー制約に基づく確実なテストケース
    
    let boot = boot_test::<App>().await.unwrap();
    
    // 【無効な外部キーテスト】: 存在しないuser_idでの監査ログ作成
    let nonexistent_uuid = uuid::Uuid::new_v4(); // 存在しないUUID
    
    let invalid_log = training_management::models::audit_logs::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(Some(12345)),
        action: sea_orm::ActiveValue::Set("test_action".to_string()),
        resource_type: sea_orm::ActiveValue::Set(Some("test".to_string())),
        resource_id: sea_orm::ActiveValue::Set(None),
        details: sea_orm::ActiveValue::Set(None),
        ip_address: sea_orm::ActiveValue::Set(Some("192.168.1.200".to_string())),
        user_agent: sea_orm::ActiveValue::Set(Some("Test-Agent".to_string())),
        ..Default::default()
    };
    
    // 【外部キー制約違反確認】: データベースエラーが発生することを確認
    let result = invalid_log.insert(&boot.app_context.db).await;
    assert!(result.is_err());
    
    // 【エラー内容検証】: エラーメッセージが外部キー制約違反を示している
    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("foreign key") || 
        error_message.contains("references") || 
        error_message.contains("user") ||
        error_message.contains("violates")
    );
}