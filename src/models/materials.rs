/**
 * 【機能概要】: 教材（Materials）モデルの実装
 * 【実装方針】: CompaniesとStudentsモデルの実装パターンを踏襲し、テストが通る最小限の機能を実装
 * 【テスト対応】: Redフェーズで作成されたMaterialsテストケースを通すための実装
 * 🟢 信頼性レベル: 既存CompaniesとStudentsモデルと同等パターンで実装
 */

use loco_rs::prelude::*;
use sea_orm::{QueryOrder, QuerySelect}; // 【検索機能拡張】: ページネーションと並び順制御に必要なトレイト
use serde::Deserialize;

pub use super::_entities::materials::{self, ActiveModel, Entity, Model};

// 【設定定数】: URL・ドメイン処理の安全性とパフォーマンスを保証する制限値
// 【調整可能性】: 将来的な要件変更に対応できる設定値として定義
const MAX_URL_LENGTH: usize = 2048; // 【URL長制限】: RFC 2616に基づく実用的な上限値 🟢
const MAX_DOMAIN_LENGTH: usize = 253; // 【ドメイン長制限】: RFC 1035に基づくドメイン名の最大長 🟢
const FALLBACK_DOMAIN: &str = "unknown"; // 【フォールバック値】: 解析失敗時の安全なデフォルト値 🟢
const MIN_RECOMMENDATION_LEVEL: i32 = 1; // 【推奨レベル下限】: データベースCHECK制約と同期した最小値 🟢
const MAX_RECOMMENDATION_LEVEL: i32 = 5; // 【推奨レベル上限】: データベースCHECK制約と同期した最大値 🟢
const MAX_PAGE_SIZE: u32 = 100; // 【ページサイズ上限】: メモリ効率とレスポンス性能を考慮した実用的制限 🟢

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 1, max = 255, message = "教材タイトルは1文字以上255文字以下である必要があります"))]
    pub title: String,
    #[validate(url(message = "有効なURL形式である必要があります"))]
    pub url: String,
    #[validate(length(min = 1, max = 255, message = "ドメインは1文字以上255文字以下である必要があります"))]
    pub domain: String,
    #[validate(length(min = 1, message = "教材説明は1文字以上である必要があります"))]
    pub description: String,
    #[validate(custom(function = "validate_recommendation_level"))]
    pub recommendation_level: i32,
}

/**
 * 【機能概要】: 推奨レベルの妥当性をチェックする強化版カスタムバリデーション関数
 * 【改善内容】: より詳細なエラーメッセージと設定値の外部化によりメンテナンス性を向上
 * 【設計方針】: データベース制約とアプリケーション制約の二重チェックによる堅牢性確保
 * 【パフォーマンス】: 分岐処理を最小化し、効率的な範囲チェックを実装
 * 【保守性】: 制限値を定数化して将来的な変更に対応しやすい構造
 * 🟢 信頼性レベル: database-schema.sqlとビジネスルール仕様に基づく確実な実装
 */
fn validate_recommendation_level(recommendation_level: i32) -> Result<(), validator::ValidationError> {
    // 【推奨レベル範囲チェック】: ビジネスルールに基づく厳密な範囲検証
    // 【効率的判定】: 単一の条件式による高速な範囲チェック
    if (MIN_RECOMMENDATION_LEVEL..=MAX_RECOMMENDATION_LEVEL).contains(&recommendation_level) {
        Ok(())
    } else {
        // 【詳細エラー処理】: ユーザーにとって分かりやすい具体的なエラーメッセージ
        // 【ユーザビリティ】: 有効な範囲を明示することで修正方針を明確化
        let mut error = validator::ValidationError::new("invalid_recommendation_level");
        error.message = Some(format!(
            "推奨レベルは{}から{}の範囲内である必要があります（入力値: {}）",
            MIN_RECOMMENDATION_LEVEL, MAX_RECOMMENDATION_LEVEL, recommendation_level
        ).into());
        Err(error)
    }
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            title: self.title.as_ref().to_owned(),
            url: self.url.as_ref().to_owned(),
            domain: self.domain.as_ref().to_owned(),
            description: self.description.as_ref().to_owned(),
            recommendation_level: self.recommendation_level.as_ref().to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::materials::ActiveModel {
    /**
     * 【機能概要】: データ保存前の前処理（バリデーション実行、UUID生成、ドメイン抽出）
     * 【実装方針】: CompaniesとStudentsモデルと同様のパターンで基本機能を実装
     * 【テスト対応】: test_教材情報の正常作成テストのUUID生成要件に対応
     * 🟢 信頼性レベル: 既存実装パターンの踏襲による確実な実装
     */
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // 【バリデーション実行】: 保存前にデータの妥当性をチェック
        self.validate()?;
        
        if insert {
            let mut this = self;
            // 【UUID主キー生成】: 新規作成時にUUIDを自動生成
            this.id = ActiveValue::Set(uuid::Uuid::new_v4());
            
            // 【ドメイン抽出】: URLからドメイン部分を自動抽出
            // 🟡 簡易実装: リファクタ段階でより厳密なドメイン抽出ロジックに改善予定
            if let ActiveValue::Set(url) = &this.url {
                let domain = extract_domain_from_url(url);
                this.domain = ActiveValue::Set(domain);
            }
            
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

/**
 * 【機能概要】: URLからドメイン名を安全に抽出する改善版実装
 * 【改善内容】: より厳密なURL解析とセキュリティを考慮したドメイン検証を実装
 * 【設計方針】: RFC 3986に準拠したURL解析とセキュリティベストプラクティスの適用
 * 【パフォーマンス】: 不要な文字列アロケーションを最小化し、効率的な処理を実現
 * 【保守性】: エラーケースを明確に分類し、将来の機能拡張に対応しやすい構造
 * 🟢 信頼性レベル: RFC標準とセキュリティガイドラインに基づく確実な実装
 */
fn extract_domain_from_url(url: &str) -> String {
    // 【入力値事前検証】: 空文字列や異常に長いURLの早期検出
    // 【セキュリティ考慮】: DoS攻撃を防ぐためのURL長制限チェック
    if url.is_empty() || url.len() > MAX_URL_LENGTH {
        return FALLBACK_DOMAIN.to_string();
    }
    
    // 【安全なURL解析】: RFC 3986準拠の厳密なURL解析を実行
    // 【エラーハンドリング】: 解析失敗時の適切な分類と対処
    match url::Url::parse(url) {
        Ok(parsed_url) => {
            match parsed_url.host_str() {
                Some(host) => {
                    // 【ドメイン正規化】: 安全なドメイン名への正規化処理
                    // 【セキュリティ検証】: 悪意のあるドメインパターンの検出
                    normalize_domain(host)
                },
                None => {
                    // 【URL構造エラー】: ホスト部分が存在しないURL（file:// 等）
                    FALLBACK_DOMAIN.to_string()
                }
            }
        },
        Err(_) => {
            // 【URL形式エラー】: 不正な形式のURL文字列
            // 【ログ記録】: 将来のデバッグのためエラー詳細を記録（本実装では省略）
            FALLBACK_DOMAIN.to_string()
        }
    }
}

/**
 * 【ヘルパー関数】: ドメイン名の正規化と安全性検証
 * 【再利用性】: ドメイン関連の処理で共通利用可能
 * 【単一責任】: ドメインの正規化のみに特化した責任分離
 */
fn normalize_domain(domain: &str) -> String {
    // 【正規化処理】: ドメイン名の小文字変換と不要文字除去
    // 【可読性向上】: 一貫したドメイン表記への統一
    let normalized = domain.to_lowercase().trim().to_string();
    
    // 【長さ制限】: 異常に長いドメイン名の検出と制限
    // 【セキュリティ考慮】: バッファオーバーフロー攻撃の防止
    if normalized.len() > MAX_DOMAIN_LENGTH {
        FALLBACK_DOMAIN.to_string()
    } else {
        normalized
    }
}

impl Model {
    /**
     * 【機能概要】: 教材をタイトルで検索する機能（一意検索）
     * 【改善内容】: 入力値の正規化と詳細なエラーハンドリングを追加
     * 【設計方針】: 一意性を前提とした単一レコード検索に特化
     * 【パフォーマンス】: インデックスを活用した効率的なクエリ実行
     * 【保守性】: エラーケースの明確な分類と処理
     * 🟢 信頼性レベル: データベーススキーマとインデックス設計に基づく確実な実装
     */
    pub async fn find_by_title(db: &DatabaseConnection, title: &str) -> ModelResult<Self> {
        // 【入力値検証】: 空文字列や異常に長いタイトルの早期検出
        if title.is_empty() || title.len() > 255 {
            return Err(ModelError::EntityNotFound);
        }
        
        let material = materials::Entity::find()
            .filter(materials::Column::Title.eq(title.trim()))
            .one(db)
            .await?;
        material.ok_or_else(|| ModelError::EntityNotFound)
    }

    /**
     * 【機能概要】: 教材をドメインで検索する機能（ページネーション対応）
     * 【改善内容】: 大量データ対応のページネーション機能と並び順制御を追加
     * 【設計方針】: スケーラブルな検索体験の提供とメモリ効率の最適化
     * 【パフォーマンス】: LIMIT/OFFSETクエリによる効率的なデータ取得
     * 【保守性】: ページサイズ制限による予期しない大量データ取得の防止
     * 🟢 信頼性レベル: データベースインデックス設計とページネーション設計に基づく確実な実装
     */
    pub async fn find_by_domain_paginated(
        db: &DatabaseConnection, 
        domain: &str,
        page: u32,
        page_size: u32
    ) -> ModelResult<Vec<Self>> {
        // 【入力値検証】: ページサイズの制限とドメイン名の妥当性チェック
        // 【セキュリティ考慮】: 過大なページサイズによるメモリ枯渇攻撃の防止
        if page_size == 0 || page_size > MAX_PAGE_SIZE || domain.is_empty() {
            return Ok(vec![]);
        }
        
        let offset = page.saturating_mul(page_size);
        
        let materials = materials::Entity::find()
            .filter(materials::Column::Domain.eq(domain.trim().to_lowercase()))
            .order_by_asc(materials::Column::Title) // 【並び順】: 一貫した表示順序の保証
            .limit(page_size as u64)
            .offset(offset as u64)
            .all(db)
            .await?;
        Ok(materials)
    }

    /**
     * 【機能概要】: 推奨レベル別の教材検索機能（範囲検索対応）
     * 【改善内容】: 範囲検索機能とソート機能を追加して検索体験を向上
     * 【設計方針】: 柔軟な検索条件設定による高度なフィルタリング機能
     * 【パフォーマンス】: インデックスを活用した効率的な範囲検索
     * 【保守性】: 推奨レベルの妥当性チェックと安全な範囲検索
     * 🟢 信頼性レベル: データベースインデックス設計とビジネスルールに基づく確実な実装
     */
    pub async fn find_by_recommendation_range(
        db: &DatabaseConnection, 
        min_level: i32, 
        max_level: i32
    ) -> ModelResult<Vec<Self>> {
        // 【範囲検証】: 推奨レベル範囲の妥当性と論理的整合性をチェック
        // 【ビジネスルール検証】: データベース制約と同期した値域チェック
        if min_level < MIN_RECOMMENDATION_LEVEL || max_level > MAX_RECOMMENDATION_LEVEL || min_level > max_level {
            return Ok(vec![]);
        }
        
        let materials = materials::Entity::find()
            .filter(materials::Column::RecommendationLevel.gte(min_level))
            .filter(materials::Column::RecommendationLevel.lte(max_level))
            .order_by_desc(materials::Column::RecommendationLevel) // 【優先順序】: 高評価順での表示
            .order_by_asc(materials::Column::Title) // 【副次ソート】: 同評価内でのタイトル順
            .all(db)
            .await?;
        Ok(materials)
    }

    /**
     * 【機能概要】: 推奨レベル別の教材検索機能（単一レベル検索、下位互換性維持）
     * 【改善内容】: 既存APIとの互換性を保ちつつ内部実装を範囲検索に統一
     * 【設計方針】: レガシーAPIサポートと新機能の統合による一貫性確保
     * 【パフォーマンス】: 内部で範囲検索を活用した効率的な実装
     * 【保守性】: 単一実装による保守コストの削減
     * 🟢 信頼性レベル: 新機能への統合による確実性の向上
     */
    pub async fn find_by_recommendation_level(db: &DatabaseConnection, level: i32) -> ModelResult<Vec<Self>> {
        // 【実装統一】: 範囲検索機能を活用した単一レベル検索の実現
        // 【下位互換性】: 既存のAPIインターフェースを維持
        Self::find_by_recommendation_range(db, level, level).await
    }
}