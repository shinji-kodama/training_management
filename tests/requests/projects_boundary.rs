use insta::{assert_debug_snapshot, with_settings};
use loco_rs::{app::AppContext, testing::prelude::*, TestServer};
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App, 
    models::{projects, companies, trainings, students, project_participants},
};

use super::prepare_data;

/// プロジェクト管理機能境界値テスト
/// 
/// 【テスト対象】: プロジェクト管理の境界値機能実装確認テスト（TDD Green Phase）
/// 【実装方針】: 最小値・最大値・境界値でのシステム堅牢性をテスト
/// 【確認項目】: Controller境界値実装により全テストが成功することを確認
/// 🟢 TDD Green Phase: Controller境界値実装により確実な成功が期待される

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("projects_boundary_request");
        let _guard = settings.bind_to_scope();
    };
}

/// テスト用境界値データ作成ヘルパー関数
/// 【機能概要】: 境界値テスト用のプロジェクトをDBに作成
/// 【改善内容】: 境界値テスト機能に特化したテストデータ作成
async fn create_test_project_for_boundary(ctx: &AppContext, _request: &TestServer) -> projects::Model {
    // 【境界値テストデータ】: 境界値テスト用のプロジェクトデータ
    let project_data = projects::ActiveModel {
        title: ActiveValue::set("境界値テスト用プロジェクト".to_string()),
        training_id: ActiveValue::set(uuid::Uuid::new_v4()), // テスト用ダミーUUID
        company_id: ActiveValue::set(uuid::Uuid::new_v4()),   // テスト用ダミーUUID
        start_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 9, 1).unwrap()),
        end_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        created_by: ActiveValue::set(1), // テスト用固定値
        ..Default::default()
    };

    project_data.insert(&ctx.db).await.expect("テスト用境界値プロジェクトの作成に失敗")
}

#[tokio::test]
#[serial]
async fn test_参加者ステータス最小値1_正常処理確認() {
    // 【テスト目的】: 研修状況評価の最低値（1: failed）での動作保証
    // 【テスト内容】: status = 1（最低評価）の参加者データでの正常処理確認
    // 【期待される動作】: 正常処理（HTTP 200 OK）+ データベース保存成功
    // 🟢 TDD Green Phase: Controller境界値実装により確実な成功が期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: ステータス値下限での正常動作確認用データ
        // 【初期条件設定】: 最低評価でも正常にシステム処理されることを確認
        let test_project = create_test_project_for_boundary(&ctx, &request).await;
        
        let boundary_participant_payload = serde_json::json!({
            "project_id": test_project.id,
            "student_id": uuid::Uuid::new_v4(), // テスト用受講者ID
            "status": 1, // 最小値（failed）
            "all_interviews_completed": false
        });

        // 【実際の処理実行】: POST /projects/{id}/participants エンドポイントで最小値テスト
        // 【処理内容】: Controller境界値実装による200確認（Green フェーズで成功確認）
        let response = request
            .post(&format!("/projects/{}/participants", test_project.id))
            .json(&boundary_participant_payload)
            .await;

        // 【結果検証】: Greenフェーズ - 最小値での正常処理が成功
        // 【期待値確認】: Green フェーズ - 境界値Controllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "境界値テスト: ステータス最小値1での正常処理が成功"
        ); // 【確認内容】: 境界値での正常動作確認 🟢

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
async fn test_参加者ステータス最大値5_正常処理確認() {
    // 【テスト目的】: 研修状況評価の最高値（5: excellent）での動作保証
    // 【テスト内容】: status = 5（最高評価）の参加者データでの正常処理確認
    // 【期待される動作】: 正常処理（HTTP 200 OK）+ 統計計算正確性
    // 🟢 TDD Green Phase: Controller境界値実装により確実な成功が期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: ステータス値上限での正常動作確認用データ
        // 【初期条件設定】: 最高評価での集計・表示処理の正確性確認
        let test_project = create_test_project_for_boundary(&ctx, &request).await;
        
        let boundary_participant_payload = serde_json::json!({
            "project_id": test_project.id,
            "student_id": uuid::Uuid::new_v4(), // テスト用受講者ID
            "status": 5, // 最大値（excellent）
            "all_interviews_completed": true
        });

        // 【実際の処理実行】: POST /projects/{id}/participants エンドポイントで最大値テスト
        // 【処理内容】: Controller境界値実装による200確認（Green フェーズで成功確認）
        let response = request
            .post(&format!("/projects/{}/participants", test_project.id))
            .json(&boundary_participant_payload)
            .await;

        // 【結果検証】: Greenフェーズ - 最大値での正常処理が成功
        // 【期待値確認】: Green フェーズ - 境界値Controllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "境界値テスト: ステータス最大値5での正常処理が成功"
        ); // 【確認内容】: 境界値での正常動作確認 🟢

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
async fn test_参加者ステータス範囲外0_422バリデーションエラー() {
    // 【テスト目的】: 許可範囲を下回る不正値での堅牢性確認
    // 【テスト内容】: status = 0（範囲外下限）の参加者データでのエラー処理確認
    // 【期待される動作】: HTTP 422 Unprocessable Entity + 範囲外エラーメッセージ
    // 🟢 TDD Green Phase: Controller境界値実装により確実な422が期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: ステータス値範囲外での適切な例外処理確認用データ
        // 【初期条件設定】: 不正値に対する適切なエラー処理の確認
        let test_project = create_test_project_for_boundary(&ctx, &request).await;
        
        let invalid_participant_payload = serde_json::json!({
            "project_id": test_project.id,
            "student_id": uuid::Uuid::new_v4(), // テスト用受講者ID
            "status": 0, // 範囲外（無効値）
            "all_interviews_completed": false
        });

        // 【実際の処理実行】: POST /projects/{id}/participants エンドポイントで範囲外値テスト
        // 【処理内容】: Controller境界値実装による422確認（Green フェーズで422確認）
        let response = request
            .post(&format!("/projects/{}/participants", test_project.id))
            .json(&invalid_participant_payload)
            .await;

        // 【結果検証】: Greenフェーズ - 範囲外値での適切なエラー処理が成功
        // 【期待値確認】: Green フェーズ - 境界値Controllerが実装されたため422が期待される
        assert_eq!(
            response.status_code(),
            422,
            "境界値テスト: ステータス範囲外0での422バリデーションエラーが成功"
        ); // 【確認内容】: 範囲外値での適切なエラー処理確認 🟢

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
async fn test_プロジェクト開始終了同一日_正常処理確認() {
    // 【テスト目的】: 最短期間（1日）プロジェクトでの動作保証
    // 【テスト内容】: start_date = end_date（同一日）のプロジェクトデータでの正常処理確認
    // 【期待される動作】: 正常処理（HTTP 200 OK）+ 期間計算正確性
    // 🟢 TDD Green Phase: Controller境界値実装により確実な成功が期待される

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 最短期間プロジェクトでの全機能正常動作確認用データ
        // 【初期条件設定】: 極端に短いプロジェクト期間での正常処理確認
        
        let same_day_project_payload = serde_json::json!({
            "title": "1日完結プロジェクト",
            "training_id": uuid::Uuid::new_v4(),
            "company_id": uuid::Uuid::new_v4(),
            "start_date": "2025-09-01", // 開始日
            "end_date": "2025-09-01",   // 終了日（同一日）
            "created_by": 1
        });

        // 【実際の処理実行】: POST /projects エンドポイントで同一日プロジェクト作成テスト
        // 【処理内容】: Controller境界値実装による200確認（Green フェーズで成功確認）
        let response = request
            .post("/projects")
            .json(&same_day_project_payload)
            .await;

        // 【結果検証】: Greenフェーズ - 同一日プロジェクトでの正常処理が成功
        // 【期待値確認】: Green フェーズ - 境界値Controllerが実装されたため200が期待される
        assert_eq!(
            response.status_code(),
            200,
            "境界値テスト: プロジェクト開始終了同一日での正常処理が成功"
        ); // 【確認内容】: 最短期間での正常動作確認 🟢

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