use insta::{assert_debug_snapshot, with_settings};
use loco_rs::{app::AppContext, testing::prelude::*, TestServer};
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use training_management::{
    app::App, 
    models::{projects, companies, trainings, students, project_participants},
};

use super::prepare_data;

/// プロジェクト管理機能セキュリティ統合テスト
/// 
/// 【テスト対象】: プロジェクト管理のセキュリティ機能実装前失敗テスト（TDD Red Phase）
/// 【実装方針】: 認証・認可・企業制限の多層セキュリティをテスト
/// 【確認項目】: セキュリティController未実装により全テストが失敗することを確認
/// 🔴 TDD Red Phase: セキュリティ統合Controller未実装により確実な失敗が期待される

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("projects_security_request");
        let _guard = settings.bind_to_scope();
    };
}

/// テスト用企業・プロジェクト・参加者データ統合作成ヘルパー関数
/// 【機能概要】: セキュリティテスト用の包括的テストデータをDBに作成
/// 【改善内容】: 複数企業・プロジェクト・参加者の関連データを一括作成
async fn create_test_security_data(ctx: &AppContext, _request: &TestServer) -> (projects::Model, projects::Model) {
    // 【企業A作成】: セキュリティテスト用の第1企業
    let company_a = companies::ActiveModel {
        name: ActiveValue::set("株式会社テックA".to_string()),
        contact_person: ActiveValue::set("田中太郎".to_string()),
        contact_email: ActiveValue::set("tanaka@tech-a.com".to_string()),
        chat_link: ActiveValue::set(Some("https://chat.tech-a.com/project".to_string())),
        ..Default::default()
    };
    let company_a = company_a.insert(&ctx.db).await.expect("企業Aの作成に失敗");

    // 【企業B作成】: セキュリティテスト用の第2企業
    let company_b = companies::ActiveModel {
        name: ActiveValue::set("株式会社テックB".to_string()),
        contact_person: ActiveValue::set("佐藤花子".to_string()),
        contact_email: ActiveValue::set("sato@tech-b.com".to_string()),
        chat_link: ActiveValue::set(Some("https://chat.tech-b.com/project".to_string())),
        ..Default::default()
    };
    let company_b = company_b.insert(&ctx.db).await.expect("企業Bの作成に失敗");

    // 【プロジェクトA作成】: 企業A専用プロジェクト（セキュリティテスト用）
    let project_a = projects::ActiveModel {
        title: ActiveValue::set("企業A専用Rustプロジェクト".to_string()),
        training_id: ActiveValue::set(uuid::Uuid::new_v4()),
        company_id: ActiveValue::set(company_a.id),
        start_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 9, 1).unwrap()),
        end_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        created_by: ActiveValue::set(1),
        ..Default::default()
    };
    let project_a = project_a.insert(&ctx.db).await.expect("プロジェクトAの作成に失敗");

    // 【プロジェクトB作成】: 企業B専用プロジェクト（セキュリティテスト用）
    let project_b = projects::ActiveModel {
        title: ActiveValue::set("企業B専用Goプロジェクト".to_string()),
        training_id: ActiveValue::set(uuid::Uuid::new_v4()),
        company_id: ActiveValue::set(company_b.id),
        start_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()),
        end_date: ActiveValue::set(chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
        created_by: ActiveValue::set(2),
        ..Default::default()
    };
    let project_b = project_b.insert(&ctx.db).await.expect("プロジェクトBの作成に失敗");

    (project_a, project_b)
}

#[tokio::test]
#[serial]
async fn test_未認証ユーザープロジェクトアクセス拒否_controller未実装404エラー() {
    // 【テスト目的】: 未認証ユーザーのプロジェクト管理機能アクセス拒否確認
    // 【テスト内容】: セッション認証なしでのプロジェクト一覧アクセス試行
    // 【期待される動作】: Controller未実装のため、まずHTTP 404 Not Foundが返される
    // 🔴 TDD Red Phase: セキュリティController未実装により404が期待される（将来401に変更）

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 未認証アクセステスト用のプロジェクトデータ作成
        // 【初期条件設定】: セッション情報なしでの不正アクセス状況再現
        let (test_project_a, _) = create_test_security_data(&ctx, &request).await;

        // 【実際の処理実行】: セッション認証なしでGET /projects エンドポイントアクセス
        // 【処理内容】: 未認証でのプロジェクト一覧取得試行（Red フェーズで404確認）
        let response = request
            .get("/projects")
            // セッション情報を意図的に送信しない（未認証状態）
            .await;

        // 【結果検証】: Controller未実装によりHTTP 404 Not Foundが返される
        // 【期待値確認】: Red フェーズ - まずController実装、その後認証統合で401に変更予定
        // 【セキュリティ注記】: 将来的にはHTTP 401 Unauthorizedに変更される
        assert_eq!(
            response.status_code(),
            404,
            "セキュリティController未実装のため404 Not Found（将来401 Unauthorized）が期待される"
        ); // 【確認内容】: 未認証アクセス拒否機能の基盤確認（Controller実装後は401） 🔴

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
async fn test_instructor権限プロジェクト作成拒否_controller未実装404エラー() {
    // 【テスト目的】: instructor権限によるプロジェクト作成操作の拒否確認
    // 【テスト内容】: 閲覧専用権限でのプロジェクト作成試行
    // 【期待される動作】: Controller未実装のため、まずHTTP 404 Not Foundが返される
    // 🔴 TDD Red Phase: RBAC統合Controller未実装により404が期待される（将来403に変更）

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: instructor権限での不正操作テスト用データ
        // 【初期条件設定】: 権限レベル不足でのプロジェクト作成試行状況再現
        let (_, _) = create_test_security_data(&ctx, &request).await;

        let project_payload = serde_json::json!({
            "title": "instructor権限での不正作成試行",
            "training_id": uuid::Uuid::new_v4(),
            "company_id": uuid::Uuid::new_v4(),
            "start_date": "2025-09-01",
            "end_date": "2025-12-31",
            "created_by": 3 // instructorユーザー想定
        });

        // 【実際の処理実行】: instructor権限セッションでPOST /projects エンドポイントアクセス
        // 【処理内容】: 権限不足でのプロジェクト作成試行（Red フェーズで404確認）
        let response = request
            .post("/projects")
            .json(&project_payload)
            // TODO: instructor権限のセッション情報を追加（Green/Refactorフェーズで実装）
            .await;

        // 【結果検証】: Controller未実装によりHTTP 404 Not Foundが返される
        // 【期待値確認】: Red フェーズ - まずController実装、その後RBAC統合で403に変更予定
        // 【セキュリティ注記】: 将来的にはHTTP 403 Forbiddenに変更される
        assert_eq!(
            response.status_code(),
            404,
            "RBACController未実装のため404 Not Found（将来403 Forbidden）が期待される"
        ); // 【確認内容】: 権限レベル制御機能の基盤確認（Controller実装後は403） 🔴

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
async fn test_他社専用プロジェクト不正アクセス拒否_controller未実装404エラー() {
    // 【テスト目的】: 企業間データ分離による他社プロジェクト不正アクセス拒否確認
    // 【テスト内容】: 企業Aのユーザーが企業Bのプロジェクトにアクセス試行
    // 【期待される動作】: Controller未実装のため、まずHTTP 404 Not Foundが返される
    // 🔴 TDD Red Phase: 企業制限Controller未実装により404が期待される（将来403に変更）

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 企業間データ分離テスト用の複数企業プロジェクト
        // 【初期条件設定】: 異なる企業間でのプロジェクト不正アクセス状況再現
        let (project_a, project_b) = create_test_security_data(&ctx, &request).await;

        // 【実際の処理実行】: 企業Aユーザーが企業Bプロジェクトにアクセス試行
        // 【処理内容】: 企業制限違反でのプロジェクト詳細取得試行（Red フェーズで404確認）
        let response = request
            .get(&format!("/projects/{}", project_b.id)) // 企業Bプロジェクトに不正アクセス
            // TODO: 企業Aユーザーのセッション情報を追加（Green/Refactorフェーズで実装）
            .await;

        // 【結果検証】: Controller未実装によりHTTP 404 Not Foundが返される
        // 【期待値確認】: Red フェーズ - まずController実装、その後企業制限で403に変更予定
        // 【セキュリティ注記】: 将来的にはHTTP 403 Forbiddenに変更される
        assert_eq!(
            response.status_code(),
            404,
            "企業制限Controller未実装のため404 Not Found（将来403 Forbidden）が期待される"
        ); // 【確認内容】: 企業間データ分離機能の基盤確認（Controller実装後は403） 🔴

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
async fn test_重複参加者追加エラー処理_controller未実装404エラー() {
    // 【テスト目的】: 同一プロジェクトへの重複参加者追加のエラー処理確認
    // 【テスト内容】: 既存参加者を同一プロジェクトに再度追加する試行
    // 【期待される動作】: Controller未実装のため、まずHTTP 404 Not Foundが返される
    // 🔴 TDD Red Phase: 参加者管理Controller未実装により404が期待される（将来422に変更）

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 重複制約テスト用のプロジェクト・参加者データ
        // 【初期条件設定】: 既存参加者の重複追加による一意制約違反状況再現
        let (test_project, _) = create_test_security_data(&ctx, &request).await;

        let participant_payload = serde_json::json!({
            "project_id": test_project.id,
            "student_id": uuid::Uuid::new_v4(), // 重複予定の受講者ID
            "status": 3, // average
            "all_interviews_completed": false
        });

        // 【実際の処理実行】: POST /projects/{id}/participants エンドポイントで参加者追加
        // 【処理内容】: 重複参加者追加によるデータ整合性違反試行（Red フェーズで404確認）
        let response = request
            .post(&format!("/projects/{}/participants", test_project.id))
            .json(&participant_payload)
            .await;

        // 【結果検証】: Controller未実装によりHTTP 404 Not Foundが返される
        // 【期待値確認】: Red フェーズ - まずController実装、その後データ整合性で422に変更予定
        // 【データ整合性注記】: 将来的にはHTTP 422 Unprocessable Entityに変更される
        assert_eq!(
            response.status_code(),
            404,
            "参加者管理Controller未実装のため404 Not Found（将来422 Unprocessable Entity）が期待される"
        ); // 【確認内容】: データ整合性保護機能の基盤確認（Controller実装後は422） 🔴

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
async fn test_存在しない研修コース選択エラー_controller未実装404エラー() {
    // 【テスト目的】: 存在しない研修コースIDでのプロジェクト作成エラー処理確認
    // 【テスト内容】: 削除済み/無効な研修コースIDでプロジェクト作成試行
    // 【期待される動作】: Controller未実装のため、まずHTTP 404 Not Foundが返される
    // 🔴 TDD Red Phase: 外部キー検証Controller未実装により404が期待される（将来422に変更）

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: 外部キー制約違反テスト用の無効参照データ
        // 【初期条件設定】: 存在しない研修コースによる参照整合性違反状況再現
        let (_, _) = create_test_security_data(&ctx, &request).await;

        let project_payload = serde_json::json!({
            "title": "存在しない研修コースを参照するプロジェクト",
            "training_id": uuid::Uuid::new_v4(), // 存在しない研修コースID
            "company_id": uuid::Uuid::new_v4(),
            "start_date": "2025-09-01",
            "end_date": "2025-12-31",
            "created_by": 1
        });

        // 【実際の処理実行】: POST /projects エンドポイントで無効研修コース参照
        // 【処理内容】: 外部キー制約違反によるデータ参照エラー試行（Red フェーズで404確認）
        let response = request
            .post("/projects")
            .json(&project_payload)
            .await;

        // 【結果検証】: Controller未実装によりHTTP 404 Not Foundが返される
        // 【期待値確認】: Red フェーズ - まずController実装、その後外部キー検証で422に変更予定
        // 【データ参照注記】: 将来的にはHTTP 422 Unprocessable Entityに変更される
        assert_eq!(
            response.status_code(),
            404,
            "外部キー検証Controller未実装のため404 Not Found（将来422 Unprocessable Entity）が期待される"
        ); // 【確認内容】: 参照整合性保護機能の基盤確認（Controller実装後は422） 🔴

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
async fn test_日付整合性違反エラー処理_controller未実装404エラー() {
    // 【テスト目的】: プロジェクト期間の日付整合性違反エラー処理確認
    // 【テスト内容】: 終了日が開始日より早いプロジェクト作成試行
    // 【期待される動作】: Controller未実装のため、まずHTTP 404 Not Foundが返される
    // 🔴 TDD Red Phase: ビジネスルール検証Controller未実装により404が期待される（将来422に変更）

    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 【テストデータ準備】: ビジネスルール違反テスト用の不正日付データ
        // 【初期条件設定】: 論理的に矛盾する日付によるビジネスルール違反状況再現
        let (_, _) = create_test_security_data(&ctx, &request).await;

        let project_payload = serde_json::json!({
            "title": "日付が論理的に矛盾するプロジェクト",
            "training_id": uuid::Uuid::new_v4(),
            "company_id": uuid::Uuid::new_v4(),
            "start_date": "2025-12-31", // 開始日
            "end_date": "2025-09-01",   // 終了日（開始日より早い＝論理違反）
            "created_by": 1
        });

        // 【実際の処理実行】: POST /projects エンドポイントで日付整合性違反
        // 【処理内容】: ビジネスルール違反による論理エラー試行（Red フェーズで404確認）
        let response = request
            .post("/projects")
            .json(&project_payload)
            .await;

        // 【結果検証】: Controller未実装によりHTTP 404 Not Foundが返される
        // 【期待値確認】: Red フェーズ - まずController実装、その後ビジネスルール検証で422に変更予定
        // 【ビジネスルール注記】: 将来的にはHTTP 422 Unprocessable Entityに変更される
        assert_eq!(
            response.status_code(),
            404,
            "ビジネスルール検証Controller未実装のため404 Not Found（将来422 Unprocessable Entity）が期待される"
        ); // 【確認内容】: ビジネスルール保護機能の基盤確認（Controller実装後は422） 🔴

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