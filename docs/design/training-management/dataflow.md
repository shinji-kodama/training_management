# 研修管理システム データフロー図

## システム全体フロー

```mermaid
flowchart TD
    U[ユーザー] --> B[Web Browser]
    B --> L[Loco.rs Controller]
    L --> S[Session Auth]
    L --> V[View Templates]
    L --> BL[Business Logic Services]
    BL --> M[Models/ORM]
    M --> DB[(PostgreSQL)]
    
    V --> B
    S --> DB
```

## 認証フロー

```mermaid
sequenceDiagram
    participant U as ユーザー
    participant B as Browser
    participant C as Controller
    participant S as Session Auth
    participant DB as PostgreSQL
    
    U->>B: ログイン情報入力
    B->>C: POST /login
    C->>S: 認証情報検証
    S->>DB: ユーザー情報照会
    DB-->>S: ユーザーデータ
    S->>DB: セッション作成
    S-->>C: セッションID
    C-->>B: Cookie設定 + ダッシュボード
    B-->>U: ダッシュボード表示
```

## 教材管理フロー

```mermaid
sequenceDiagram
    participant U as 研修担当者
    participant B as Browser
    participant C as MaterialController
    participant S as MaterialService
    participant M as Material Model
    participant DB as PostgreSQL
    
    U->>B: 教材登録フォーム入力
    B->>C: POST /materials
    C->>S: 教材データ処理
    Note over S: URLからドメイン自動抽出
    S->>M: 教材データ保存
    M->>DB: INSERT材料
    DB-->>M: 登録完了
    M-->>S: 作成結果
    S-->>C: 処理結果
    C-->>B: 教材一覧ページ
    B-->>U: 登録完了表示
```

## 研修コース設計フロー

```mermaid
sequenceDiagram
    participant U as 研修担当者
    participant B as Browser
    participant C as TrainingController
    participant S as TrainingService
    participant TM as Training Model
    participant DB as PostgreSQL
    
    U->>B: 研修コース作成
    B->>C: POST /trainings
    C->>S: コースデータ処理
    S->>TM: 教材紐付け処理
    TM->>DB: 複数教材との関連作成
    TM->>DB: 取り組み期間設定
    DB-->>TM: 作成完了
    TM-->>S: 結果返却
    S-->>C: 処理結果
    C-->>B: コース詳細ページ
    B-->>U: 作成完了表示
```

## プロジェクト実施管理フロー

```mermaid
sequenceDiagram
    participant U as 研修担当者
    participant B as Browser
    participant C as ProjectController
    participant S as ProjectService
    participant PM as Project Model
    participant SM as Student Model
    participant DB as PostgreSQL
    
    U->>B: プロジェクト作成
    B->>C: POST /projects
    C->>S: プロジェクトデータ処理
    S->>PM: プロジェクト保存
    PM->>DB: INSERT project
    U->>B: 参加者追加
    B->>C: POST /projects/:id/students
    C->>S: 参加者登録処理
    S->>SM: 受講者紐付け
    SM->>DB: 関連テーブル更新
    DB-->>PM: 完了通知
    PM-->>C: 結果返却
    C-->>B: プロジェクト詳細更新
    B-->>U: 参加者一覧表示
```

## 面談管理フロー

```mermaid
sequenceDiagram
    participant U as 研修担当者
    participant B as Browser
    participant C as InterviewController
    participant S as InterviewService
    participant IM as Interview Model
    participant DB as PostgreSQL
    
    U->>B: 面談予約
    B->>C: POST /interviews
    C->>S: 面談スケジュール処理
    Note over S: 時間競合チェック
    S->>IM: 面談予約保存
    IM->>DB: INSERT interview
    
    Note over U,DB: --- 面談実施後 ---
    
    U->>B: 面談記録入力
    B->>C: PUT /interviews/:id
    C->>S: 記録処理
    Note over S: Markdown形式検証
    S->>IM: 面談記録更新
    IM->>DB: UPDATE interview
    
    alt 全面談完了フラグOFF
        S->>B: 次回面談設定促進表示
    end
    
    DB-->>IM: 更新完了
    IM-->>C: 結果返却
    C-->>B: 面談詳細更新
    B-->>U: 記録完了表示
```

## 定例会管理フロー

```mermaid
sequenceDiagram
    participant U as 研修講師
    participant B as Browser
    participant C as MeetingController
    participant S as MeetingService
    participant MM as Meeting Model
    participant DB as PostgreSQL
    
    U->>B: 定例会設定
    B->>C: POST /meetings
    C->>S: 定例会データ処理
    
    alt 繰り返し設定あり
        Note over S: 毎週・隔週スケジュール生成
        S->>MM: 複数回分の定例会作成
        MM->>DB: 一括INSERT meetings
    else 単発設定
        S->>MM: 単一定例会作成
        MM->>DB: INSERT meeting
    end
    
    DB-->>MM: 作成完了
    MM-->>S: 結果返却
    S-->>C: 処理結果
    C-->>B: 定例会一覧更新
    B-->>U: 設定完了表示
```

## 権限制御フロー

```mermaid
flowchart TD
    REQ[HTTPリクエスト] --> AUTH{認証チェック}
    AUTH -->|未認証| LOGIN[ログインページ]
    AUTH -->|認証済み| AUTHZ{認可チェック}
    
    AUTHZ -->|管理者| ADMIN[全機能アクセス]
    AUTHZ -->|研修担当者| TRAINER[教材・コース・面談管理]
    AUTHZ -->|研修講師| INSTRUCTOR[定例会管理・閲覧]
    AUTHZ -->|権限なし| FORBIDDEN[403エラー]
    
    ADMIN --> RESOURCE[リソースアクセス]
    TRAINER --> RESOURCE
    INSTRUCTOR --> RESOURCE
```

## データアクセスパターン

```mermaid
flowchart LR
    C[Controller] --> S[Service Layer]
    S --> M[Model Layer]
    M --> ORM[SeaORM]
    ORM --> DB[(PostgreSQL)]
    
    S --> CACHE{アプリケーション\nキャッシュ}
    CACHE -->|Hit| S
    CACHE -->|Miss| M
    
    DB --> SESSION[(sessions table)]
    DB --> AUDIT[(audit_logs table)]
```

## エラーハンドリングフロー

```mermaid
sequenceDiagram
    participant B as Browser
    participant C as Controller
    participant S as Service
    participant M as Model
    participant DB as PostgreSQL
    
    B->>C: リクエスト
    C->>S: 処理依頼
    S->>M: データ操作
    M->>DB: クエリ実行
    
    alt 正常処理
        DB-->>M: 成功レスポンス
        M-->>S: データ返却
        S-->>C: 処理結果
        C-->>B: 成功画面
    else エラー発生
        DB-->>M: エラー
        M-->>S: エラー情報
        Note over S: ログ記録
        S-->>C: エラー詳細
        C-->>B: エラーページ/メッセージ
    end
```

## HTMX部分更新フロー（必要最小限）

```mermaid
sequenceDiagram
    participant U as ユーザー
    participant B as Browser/HTMX
    participant C as Controller
    participant S as Service
    participant DB as PostgreSQL
    
    Note over U,DB: フォームバリデーション例
    
    U->>B: フォーム入力
    B->>C: hx-post (部分送信)
    C->>S: バリデーション処理
    
    alt バリデーション成功
        S-->>C: 成功結果
        C-->>B: 部分HTMLフラグメント
        B-->>U: インライン成功表示
    else バリデーションエラー
        S-->>C: エラー詳細
        C-->>B: エラーHTMLフラグメント
        B-->>U: インラインエラー表示
    end
```

## セッション管理フロー

```mermaid
sequenceDiagram
    participant B as Browser
    participant MW as SessionMiddleware
    participant DB as PostgreSQL
    
    B->>MW: リクエスト + Cookie
    MW->>DB: セッション照会
    
    alt 有効セッション
        DB-->>MW: セッションデータ
        MW->>MW: セッション更新
        MW->>DB: 最終アクセス時刻更新
        MW-->>B: 認証済みリクエスト続行
    else 無効セッション
        MW-->>B: ログインページリダイレクト
    end
    
    Note over B,DB: セッション自動クリーンアップ
    MW->>DB: 期限切れセッション削除
```

これらのデータフローにより、研修管理システムの各機能における処理の流れと、モノリシック構成でのシンプルな通信パターンが明確になります。