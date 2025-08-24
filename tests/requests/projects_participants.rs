use insta::{assert_debug_snapshot, with_settings};
use loco_rs::{app::AppContext, testing::prelude::*, TestServer};
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App, 
    models::{projects, companies, trainings, students, project_participants},
};

use super::prepare_data;

/// プロジェクト参加者管理機能のHTTPエンドポイント統合テスト
/// 
/// 【テスト対象】: プロジェクト参加者管理Controller層の実装確認テスト（TDD Green Phase）
/// 【実装方針】: 既存projects.rsパターンを踏襲し、参加者管理機能のHTTPエンドポイントをテスト
/// 【確認項目】: Controller実装により全テストが成功することを確認
/// 🟢 TDD Green Phase: Controller実装により確実な成功が期待される

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("projects_participants_request");
        let _guard = settings.bind_to_scope();
    };
}

/// テスト用プロジェクトデータ作成ヘルパー関数
/// 【機能概要】: 参加者管理テスト用のプロジェクトをDBに作成
/// 【改善内容】: 参加者管理機能テストに特化したテストデータ作成
async fn create_test_project_for_participants(ctx: &AppContext, _request: &TestServer) -> projects::Model {
    // 【参加者管理テストデータ】: 参加者管理機能テスト用のプロジェクトデータ
    let project_data = projects::ActiveModel {
        title: ActiveValue::set("参加者管理テスト用プロジェクト".to_string()),
        training_id: ActiveValue::set(uuid::Uuid::new_v4()), // テスト用ダミーUUID
        company_id: ActiveValue::set(uuid::Uuid::new_v4()),   // テスト用ダミーUUID
        start_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 9, 1).unwrap()),
        end_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        created_by: ActiveValue::set(1), // テスト用固定値
        ..Default::default()
    };

    project_data.insert(&ctx.db).await.expect("テスト用参加者管理プロジェクトの作成に失敗")
}

#[tokio::test]
#[serial]
async fn test_プロジェクト参加者追加_controller実装200成功() {
    // 【テスト目的】: Controller実装による200 OK成功確認
    // 【テスト内容】: POST /projects/{id}/participants エンドポイントでの実装済みアクセス
    // 【期待される動作】: HTTP 200 OK、正常な参加者追加レスポンス
    // 🟢 TDD Green Phase: projects controllerが実装され正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 参加者追加テスト用のプロジェクトデータ事前作成
        // 【初期条件設定】: Green フェーズでは実装済みアクセス、Controller実装200確認
        
        // 参加者追加用のテストプロジェクトを作成
        let test_project = create_test_project_for_participants(&ctx, &request).await;
        
        let participant_payload = serde_json::json!({
            "project_id": test_project.id,
            "student_id": uuid::Uuid::new_v4(), // テスト用受講者ID
            "status": 3, // average
            "all_interviews_completed": false
        });

        // 【実際の処理実行】: POST /projects/{id}/participants エンドポイントへの参加者追加リクエスト
        // 【処理内容】: Controller実装による200確認（Green フェーズで成功確認）
        let response = request
            .post(&format!("/projects/{}/participants", test_project.id))
            .json(&participant_payload)
            .await;

        // 【結果検証】: Greenフェーズ - Controller実装により200 OKが返される
        // 【期待値確認】: Green フェーズ - 参加者追加Controllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "projects参加者追加controllerが実装されたため200 OKが期待される"
        ); // 【確認内容】: Controller実装による200レスポンス確認 🟢

        with_settings!(
            {
                filters => vec![]
            },
            {
                assert_debug_snapshot!((response.status_code(), response.text()));
            }
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_参加者研修状況更新_controller実装200成功() {
    // 【テスト目的】: Controller実装による200 OK成功確認
    // 【テスト内容】: PUT /projects/{id}/participants/{participant_id} での実装済みアクセス
    // 【期待される動作】: HTTP 200 OK、正常な参加者状況更新レスポンス
    // 🟢 TDD Green Phase: projects controllerが実装され正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 参加者状況更新テスト用のプロジェクト・参加者データ事前作成
        // 【初期条件設定】: Green フェーズでは実装済みアクセス、Controller実装200確認
        
        // 状況更新用のテストプロジェクトを作成
        let test_project = create_test_project_for_participants(&ctx, &request).await;
        let test_participant_id = uuid::Uuid::new_v4(); // テスト用参加者ID
        
        let status_update_payload = serde_json::json!({
            "status": 4, // good
            "all_interviews_completed": true
        });

        // 【実際の処理実行】: PUT /projects/{id}/participants/{participant_id} エンドポイントへの状況更新リクエスト
        // 【処理内容】: Controller実装による200確認（Green フェーズで成功確認）
        let response = request
            .put(&format!("/projects/{}/participants/{}", test_project.id, test_participant_id))
            .json(&status_update_payload)
            .await;

        // 【結果検証】: Greenフェーズ - Controller実装により200 OKが返される
        // 【期待値確認】: Green フェーズ - 参加者状況更新Controllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "projects参加者状況更新controllerが実装されたため200 OKが期待される"
        ); // 【確認内容】: Controller実装による200レスポンス確認 🟢

        with_settings!(
            {
                filters => vec![]
            },
            {
                assert_debug_snapshot!((response.status_code(), response.text()));
            }
        );
    })
    .await;
}