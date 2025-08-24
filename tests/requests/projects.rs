use insta::{assert_debug_snapshot, with_settings};
use loco_rs::{app::AppContext, testing::prelude::*, TestServer};
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App, 
    models::{projects, companies, trainings, students},
};

use super::prepare_data;

/// プロジェクト管理機能のHTTPエンドポイント統合テスト
/// 
/// 【テスト対象】: プロジェクト管理Controller層の実装前失敗テスト（TDD Red Phase）
/// 【実装方針】: 既存materials.rsパターンを踏襲し、プロジェクト管理機能のHTTPエンドポイントをテスト
/// 【確認項目】: Controller未実装により全テストが失敗することを確認
/// 🔴 TDD Red Phase: Controller未実装により確実な失敗が期待される

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("projects_request");
        let _guard = settings.bind_to_scope();
    };
}

/// テスト用プロジェクトデータ作成ヘルパー関数
/// 【機能概要】: シンプルなテストプロジェクトをDBに作成
/// 【改善内容】: 認証関連を削除し、基本データ作成に集中
async fn create_test_project(ctx: &AppContext, _request: &TestServer) -> projects::Model {
    // 【シンプルテストデータ】: 認証なしで基本データ作成
    let project_data = projects::ActiveModel {
        title: ActiveValue::set("実践Rust開発プロジェクト".to_string()),
        training_id: ActiveValue::set(uuid::Uuid::new_v4()), // テスト用ダミーUUID
        company_id: ActiveValue::set(uuid::Uuid::new_v4()),   // テスト用ダミーUUID
        start_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 9, 1).unwrap()),
        end_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        created_by: ActiveValue::set(1), // テスト用固定値
        ..Default::default()
    };

    project_data.insert(&ctx.db).await.expect("テストプロジェクトの作成に失敗")
}

#[tokio::test]
#[serial]
async fn test_プロジェクト一覧画面表示_controller実装200成功() {
    // 【テスト目的】: Controller実装による200 OK成功確認
    // 【テスト内容】: GET /projects エンドポイントへの実装済みアクセス
    // 【期待される動作】: HTTP 200 OK、正常なJSONレスポンス
    // 🟢 TDD Green Phase: projects controllerが実装され正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, _ctx| async move {
        // 【テストデータ準備】: Controller実装確認のため基本的なリクエストのみ実行
        // 【初期条件設定】: プロジェクトルートが実装済み状態での200 OK確認

        // 【実際の処理実行】: GET /projects エンドポイントへのリクエスト送信
        // 【処理内容】: 実装済みControllerへのアクセス試行（Green フェーズで200確認）
        let response = request
            .get("/projects")
            .await;

        // 【結果検証】: Greenフェーズ - Controller実装により200 OKが返される
        // 【期待値確認】: Green フェーズ - 実装済みControllerから正常レスポンスが期待される
        // 【システムの健全性】: 実装されたルートへの正常なレスポンス確認
        assert_eq!(
            response.status_code(),
            200,
            "projects controllerが実装されたため200 OKが期待される"
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
async fn test_プロジェクト作成フォーム表示_controller実装200成功() {
    // 【テスト目的】: Controller実装による200 OK成功確認
    // 【テスト内容】: GET /projects/new エンドポイントへの実装済みアクセス
    // 【期待される動作】: HTTP 200 OK、正常なフォーム表示レスポンス
    // 🟢 TDD Green Phase: projects controllerが実装され正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: Controller実装確認のため基本的なリクエストのみ実行
        // 【初期条件設定】: プロジェクト作成フォームルートが実装済み状態での200 OK確認

        // 【実際の処理実行】: GET /projects/new エンドポイントへのリクエスト送信
        // 【処理内容】: 実装済みControllerへの作成フォーム表示要求（Green フェーズで200確認）
        let response = request
            .get("/projects/new")
            .await;

        // 【結果検証】: Greenフェーズ - Controller実装により200 OKが返される
        // 【期待値確認】: Green フェーズ - 作成フォームControllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "projects作成フォームcontrollerが実装されたため200 OKが期待される"
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
async fn test_プロジェクト作成処理_controller実装200成功() {
    // 【テスト目的】: Controller実装による200 OK成功確認
    // 【テスト内容】: POST /projects エンドポイントでの実装済みアクセス
    // 【期待される動作】: HTTP 200 OK、正常なプロジェクト作成レスポンス
    // 🟢 TDD Green Phase: projects controllerが実装され正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 実際のユーザーが入力する標準的なプロジェクト情報
        // 【初期条件設定】: Green フェーズでは実装済みアクセス、Controller実装200確認
        
        let project_payload = serde_json::json!({
            "title": "実践Rust開発プロジェクト",
            "training_id": uuid::Uuid::new_v4(),
            "company_id": uuid::Uuid::new_v4(),
            "start_date": "2025-09-01",
            "end_date": "2025-12-31",
            "created_by": 1
        });

        // 【実際の処理実行】: POST /projects エンドポイントへのプロジェクト作成リクエスト
        // 【処理内容】: Controller実装による200確認（Green フェーズで成功確認）
        let response = request
            .post("/projects")
            .json(&project_payload)
            .await;

        // 【結果検証】: Greenフェーズ - Controller実装により200 OKが返される
        // 【期待値確認】: Green フェーズ - 作成処理Controllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "projects作成処理controllerが実装されたため200 OKが期待される"
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
async fn test_プロジェクト詳細表示_controller実装200成功() {
    // 【テスト目的】: Controller実装による200 OK成功確認
    // 【テスト内容】: GET /projects/{id} エンドポイントでの実装済みアクセス
    // 【期待される動作】: HTTP 200 OK、正常なプロジェクト詳細レスポンス
    // 🟢 TDD Green Phase: projects controllerが実装され正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: Green フェーズでは実装済みアクセス
        // 【初期条件設定】: テスト用プロジェクトデータの事前作成（実装処理確認用）
        
        // 詳細表示用のテストプロジェクトを作成
        let test_project = create_test_project(&ctx, &request).await;

        // 【実際の処理実行】: GET /projects/{id} エンドポイントへの詳細表示リクエスト
        // 【処理内容】: Controller実装による200確認（Green フェーズで成功確認）
        let response = request
            .get(&format!("/projects/{}", test_project.id))
            .await;

        // 【結果検証】: Greenフェーズ - Controller実装により200 OKが返される
        // 【期待値確認】: Green フェーズ - 詳細表示Controllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "projects詳細表示controllerが実装されたため200 OKが期待される"
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