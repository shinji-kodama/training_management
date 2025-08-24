use insta::{assert_debug_snapshot, with_settings};
use loco_rs::{app::AppContext, testing::prelude::*, TestServer};
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App, 
    models::{trainings, companies},
};

use super::prepare_data;

/// 研修コース管理機能のHTTPエンドポイント統合テスト
/// 
/// 【テスト対象】: 研修コース管理Controller層の実装前失敗テスト（TDD Red Phase）
/// 【実装方針】: 既存materials.rsパターンを踏襲し、研修コース管理機能のHTTPエンドポイントをテスト
/// 【確認項目】: Controller未実装により全テストが失敗することを確認
/// 🔴 TDD Red Phase: Controller未実装により確実な失敗が期待される

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("trainings_request");
        let _guard = settings.bind_to_scope();
    };
}

/// テスト用研修コースデータ作成ヘルパー関数
/// 【機能概要】: シンプルなテスト研修コースをDBに作成
/// 【改善内容】: 認証関連を削除し、基本データ作成に集中
async fn create_test_training(ctx: &AppContext, _request: &TestServer) -> trainings::Model {
    // 【シンプルテストデータ】: 認証なしで基本データ作成
    let training_data = trainings::ActiveModel {
        title: ActiveValue::set("Rust実践研修コーステスト".to_string()),
        description: ActiveValue::set("Rust言語を実践的に習得する包括的な研修プログラム".to_string()),
        prerequisites: ActiveValue::set("プログラミング基礎知識".to_string()),
        goals: ActiveValue::set("Rustでの実践的アプリケーション開発スキル習得".to_string()),
        completion_criteria: ActiveValue::set("最終課題プロジェクトの完成と発表".to_string()),
        company_id: ActiveValue::set(None), // 全社共通研修として設定
        created_by: ActiveValue::set(1), // テスト用固定値
        ..Default::default()
    };

    training_data.insert(&ctx.db).await.expect("テスト研修コースの作成に失敗")
}

#[tokio::test]
#[serial]
async fn test_研修コース一覧画面表示_controller実装200成功() {
    // 【テスト目的】: Controller実装による200成功確認
    // 【テスト内容】: GET /trainings エンドポイントへの正常アクセス
    // 【期待される動作】: HTTP 200 OK、正常レスポンス
    // 🟢 TDD Green Phase: trainings controllerが実装により正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, _ctx| async move {
        // 【テストデータ準備】: Controller実装確認のため正常アクセス
        // 【初期条件設定】: 研修コースルートが実装状態での200成功確認

        // 【実際の処理実行】: GET /trainings エンドポイントへのリクエスト送信
        // 【処理内容】: 実装Controllerへのアクセス試行（Green フェーズで200成功確認）
        let response = request
            .get("/trainings")
            .await;

        // 【結果検証】: Controller実装によりHTTP 200 OKが返される
        // 【期待値確認】: Green フェーズ - Controllerが実装のため200成功が期待される
        // 【システムの正常性】: 実装ルートへの適切な成功レスポンス確認
        assert_eq!(
            response.status_code(),
            200,
            "trainings controllerが実装のため200 OKが期待される"
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
async fn test_研修コース作成フォーム表示_controller実装200成功() {
    // 【テスト目的】: Controller実装による200成功確認
    // 【テスト内容】: GET /trainings/new エンドポイントへの正常アクセス
    // 【期待される動作】: HTTP 200 OK、正常レスポンス
    // 🟢 TDD Green Phase: trainings controllerが実装により正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: Controller実装確認のため正常アクセス
        // 【初期条件設定】: 研修コース作成フォームルートが実装状態での200成功確認

        // 【実際の処理実行】: GET /trainings/new エンドポイントへのリクエスト送信
        // 【処理内容】: 実装Controllerへの作成フォーム表示要求（Green フェーズで200成功確認）
        let response = request
            .get("/trainings/new")
            .await;

        // 【結果検証】: Controller実装によりHTTP 200 OKが返される
        // 【期待値確認】: Green フェーズ - 作成フォームControllerが実装のため200成功が期待される
        assert_eq!(
            response.status_code(),
            200,
            "trainings作成フォームcontrollerが実装のため200 OKが期待される"
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
async fn test_研修コース作成処理_controller実装200成功() {
    // 【テスト目的】: Controller実装による200成功確認
    // 【テスト内容】: POST /trainings エンドポイントでの正常アクセス
    // 【期待される動作】: HTTP 200 OK、正常レスポンス
    // 🟢 TDD Green Phase: trainings controllerが実装により正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 実際のユーザーが入力する標準的な研修コース情報
        // 【初期条件設定】: Green フェーズでは正常アクセス、Controller実装200成功確認
        
        let training_payload = serde_json::json!({
            "title": "実践Rust開発研修",
            "description": "Rustでの実践的なアプリケーション開発を学ぶ研修コース",
            "prerequisites": "プログラミング基礎、Git使用経験",
            "goals": "Rustでの安全で高性能なアプリケーション開発能力を習得する",
            "completion_criteria": "最終プロジェクトの完成とコードレビューの合格",
            "company_id": null
        });

        // 【実際の処理実行】: POST /trainings エンドポイントへの研修コース作成リクエスト
        // 【処理内容】: Controller実装による200成功確認（Green フェーズで成功確認）
        let response = request
            .post("/trainings")
            .json(&training_payload)
            .await;

        // 【結果検証】: Controller実装によりHTTP 200 OKが返される
        // 【期待値確認】: Green フェーズ - 作成処理Controllerが実装のため200成功が期待される
        assert_eq!(
            response.status_code(),
            200,
            "trainings作成処理controllerが実装のため200 OKが期待される"
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
async fn test_研修コース詳細表示_controller実装200成功() {
    // 【テスト目的】: Controller実装による200成功確認
    // 【テスト内容】: GET /trainings/{id} エンドポイントでの正常アクセス
    // 【期待される動作】: HTTP 200 OK、正常レスポンス
    // 🟢 TDD Green Phase: trainings controllerが実装により正常レスポンスが期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: Green フェーズでは正常アクセス
        // 【初期条件設定】: テスト用研修コースデータの事前作成（正常処理確認用）
        
        // 詳細表示用のテスト研修コースを作成
        let test_training = create_test_training(&ctx, &request).await;

        // 【実際の処理実行】: GET /trainings/{id} エンドポイントへの詳細表示リクエスト
        // 【処理内容】: Controller実装による200成功確認（Green フェーズで成功確認）
        let response = request
            .get(&format!("/trainings/{}", test_training.id))
            .await;

        // 【結果検証】: Controller実装によりHTTP 200 OKが返される
        // 【期待値確認】: Green フェーズ - 詳細表示Controllerが実装のため200成功が期待される
        assert_eq!(
            response.status_code(),
            200,
            "trainings詳細表示controllerが実装のため200 OKが期待される"
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