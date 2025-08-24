use loco_rs::testing::prelude::*;
use serial_test::serial;
use training_management::app::App;

// 【テスト対象】: 面談（Interviews）Controller層の基本404テスト
// 🔴 Red Phase: Controller層完全未実装による確実な失敗

#[tokio::test]
#[serial]
async fn test_interviews_controller_404() {
    // 【テスト目的】: Controller層未実装による404 Not Found確認
    // 🔴 Red Phase: interviews controller完全未実装による確実な失敗
    
    request::<App, _, _>(|request, _ctx| async move {
        // 【面談一覧アクセス試行】: 未実装のGETエンドポイントへのリクエスト
        let response = request.get("/interviews").await;
        
        // 【Red Phase検証】: Controller未実装により404 Not Foundが返される
        assert_eq!(
            response.status_code(),
            404,
            "🔴 Red Phase: interviews controller未実装により404が期待される"
        );
    })
    .await;
}