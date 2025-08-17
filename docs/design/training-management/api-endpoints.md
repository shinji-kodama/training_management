# 研修管理システム API エンドポイント仕様

## 基本設計原則

- **RESTful設計**: リソースベースの URL設計
- **HTTP メソッド**: GET, POST, PUT, DELETE の適切な使用
- **ステータスコード**: 標準的な HTTP ステータスコードの使用
- **必要最小限のAPI**: HTMXを必要最小限に抑えるため、JSON APIも最小限とする
- **サーバーサイドレンダリング中心**: 基本的にはHTMLレスポンスを返し、JSON APIは必要な場合のみ

## 認証・認可

### セッションベース認証
すべての保護されたエンドポイントで Cookie ベースのセッション認証を使用

```
Cookie: session_id=<session-token>
```

### CSRF保護
POST, PUT, DELETE リクエストには CSRF トークンが必要

```html
<input type="hidden" name="_token" value="<csrf-token>">
```

## エンドポイント一覧

### 認証

#### POST /login
ユーザーログイン

**リクエスト（Form）:**
```
email: string
password: string
```

**レスポンス（成功時）:**
```
Status: 302 Found
Location: /dashboard
Set-Cookie: session_id=<token>; HttpOnly; Secure
```

**レスポンス（失敗時）:**
```
Status: 200 OK
Content-Type: text/html
[ログインフォーム with エラーメッセージ]
```

#### POST /logout
ユーザーログアウト

**レスポンス:**
```
Status: 302 Found
Location: /login
Set-Cookie: session_id=; expires=Thu, 01 Jan 1970 00:00:00 GMT
```

#### GET /login
ログインフォーム表示

**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[ログインフォーム HTML]
```

---

### ダッシュボード

#### GET /dashboard
ダッシュボード表示

**認証:** 必須  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[ダッシュボード HTML with 統計情報、最近の活動]
```

---

### ユーザー管理（管理者のみ）

#### GET /users
ユーザー一覧表示

**認証:** 管理者  
**クエリパラメータ:**
- `page`: integer (デフォルト: 1)
- `per_page`: integer (デフォルト: 20)
- `q`: string (検索キーワード)

**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[ユーザー一覧 HTML with ページネーション]
```

#### GET /users/new
ユーザー作成フォーム

**認証:** 管理者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[ユーザー作成フォーム HTML]
```

#### POST /users
ユーザー作成

**認証:** 管理者  
**リクエスト（Form）:**
```
name: string
email: string
password: string
role: enum(admin, trainer, instructor)
_token: string (CSRF)
```

**レスポンス（成功時）:**
```
Status: 302 Found
Location: /users
```

**レスポンス（バリデーションエラー時）:**
```
Status: 200 OK
Content-Type: text/html
[フォーム HTML with エラーメッセージ]
```

#### GET /users/:id
ユーザー詳細表示

**認証:** 管理者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[ユーザー詳細 HTML]
```

#### GET /users/:id/edit
ユーザー編集フォーム

**認証:** 管理者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[ユーザー編集フォーム HTML]
```

#### PUT /users/:id
ユーザー更新

**認証:** 管理者  
**リクエスト（Form）:**
```
name?: string
email?: string
password?: string
role?: enum(admin, trainer, instructor)
_token: string (CSRF)
```

**レスポンス（成功時）:**
```
Status: 302 Found
Location: /users/:id
```

#### DELETE /users/:id
ユーザー削除

**認証:** 管理者  
**リクエスト（Form）:**
```
_token: string (CSRF)
```

**レスポンス:**
```
Status: 302 Found
Location: /users
```

---

### 教材管理（管理者・研修担当者）

#### GET /materials
教材一覧表示

**認証:** 管理者・研修担当者（閲覧は全ユーザー可能）  
**クエリパラメータ:**
- `page`: integer
- `per_page`: integer
- `q`: string (検索キーワード)
- `domain`: string (ドメインフィルタ)
- `recommendation_level`: integer (1-5)

**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[教材一覧 HTML with フィルタ・検索機能]
```

#### GET /materials/new
教材作成フォーム

**認証:** 管理者・研修担当者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[教材作成フォーム HTML]
```

#### POST /materials
教材作成

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
title: string
url: string (URL形式)
description: string
recommendation_level: integer (1-5)
_token: string (CSRF)
```

**レスポンス（成功時）:**
```
Status: 302 Found
Location: /materials
```

#### GET /materials/:id
教材詳細表示

**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[教材詳細 HTML - おすすめ度は認証状態により表示制御]
```

#### GET /materials/:id/edit
教材編集フォーム

**認証:** 管理者・研修担当者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[教材編集フォーム HTML]
```

#### PUT /materials/:id
教材更新

**認証:** 管理者・研修担当者  
**リクエスト（Form）:** POST /materials と同様

#### DELETE /materials/:id
教材削除

**認証:** 管理者・研修担当者  

---

### 研修コース管理（管理者・研修担当者）

#### GET /trainings
研修コース一覧表示

**クエリパラメータ:**
- `page`: integer
- `per_page`: integer
- `q`: string
- `company_id`: UUID (企業フィルタ)

**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[研修コース一覧 HTML - 企業紐付けによる閲覧制御適用]
```

#### GET /trainings/new
研修コース作成フォーム

**認証:** 管理者・研修担当者  

#### POST /trainings
研修コース作成

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
title: string
description: string
prerequisites: string
goals: string
completion_criteria: string
company_id?: UUID (null=公開)
materials[]: array of {
  material_id: UUID,
  period_days: integer,
  order_index: integer
}
_token: string
```

#### GET /trainings/:id
研修コース詳細表示

**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[研修コース詳細 HTML with 紐付け教材一覧]
```

#### GET /trainings/:id/edit
研修コース編集フォーム

**認証:** 管理者・研修担当者

#### PUT /trainings/:id
研修コース更新

**認証:** 管理者・研修担当者

#### DELETE /trainings/:id
研修コース削除

**認証:** 管理者・研修担当者

---

### 企業管理（管理者）

#### GET /companies
企業一覧表示

**認証:** 管理者  

#### GET /companies/new
企業作成フォーム

**認証:** 管理者

#### POST /companies
企業作成

**認証:** 管理者  
**リクエスト（Form）:**
```
name: string
contact_person: string
contact_email: string (email形式)
chat_link?: string
_token: string
```

#### GET /companies/:id
企業詳細表示

**認証:** 管理者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[企業詳細 HTML with 所属受講者一覧]
```

#### GET /companies/:id/edit
企業編集フォーム

**認証:** 管理者

#### PUT /companies/:id
企業更新

**認証:** 管理者

#### DELETE /companies/:id
企業削除

**認証:** 管理者  
**注意:** 関連受講者が存在する場合は確認ダイアログ表示

---

### 受講者管理（管理者・研修担当者）

#### GET /students
受講者一覧表示

**認証:** 管理者・研修担当者  
**クエリパラメータ:**
- `company_id`: UUID (企業フィルタ)
- `role_type`: enum(student, company_admin)

#### GET /students/new
受講者作成フォーム

**認証:** 管理者・研修担当者

#### POST /students
受講者作成

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
name: string
email: string
company_id: UUID
role_type: enum(student, company_admin)
organization: string
_token: string
```

#### GET /students/:id
受講者詳細表示

**認証:** 管理者・研修担当者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[受講者詳細 HTML with 参加プロジェクト履歴]
```

#### GET /students/:id/edit
受講者編集フォーム

**認証:** 管理者・研修担当者

#### PUT /students/:id
受講者更新

**認証:** 管理者・研修担当者

#### DELETE /students/:id
受講者削除

**認証:** 管理者・研修担当者

---

### プロジェクト管理（管理者・研修担当者）

#### GET /projects
プロジェクト一覧表示

**認証:** 管理者・研修担当者  
**クエリパラメータ:**
- `status`: enum(active, completed, upcoming)
- `company_id`: UUID
- `training_id`: UUID

#### GET /projects/new
プロジェクト作成フォーム

**認証:** 管理者・研修担当者

#### POST /projects
プロジェクト作成

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
training_id: UUID
company_id: UUID
title: string
start_date: date
end_date: date
participants[]: array of {
  student_id: UUID,
  status?: integer (1-5, デフォルト: 3)
}
_token: string
```

#### GET /projects/:id
プロジェクト詳細表示

**認証:** 管理者・研修担当者  
**レスポンス:**
```
Status: 200 OK
Content-Type: text/html
[プロジェクト詳細 HTML with 参加者一覧、面談・定例会一覧]
```

#### GET /projects/:id/edit
プロジェクト編集フォーム

**認証:** 管理者・研修担当者

#### PUT /projects/:id
プロジェクト更新

**認証:** 管理者・研修担当者

#### DELETE /projects/:id
プロジェクト削除

**認証:** 管理者・研修担当者

#### POST /projects/:id/participants
参加者追加

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
student_id: UUID
status?: integer (1-5, デフォルト: 3)
_token: string
```

#### PUT /projects/:id/participants/:participant_id
参加者状況更新

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
status?: integer (1-5)
all_interviews_completed?: boolean
_token: string
```

#### DELETE /projects/:id/participants/:participant_id
参加者削除

**認証:** 管理者・研修担当者

---

### 面談管理（管理者・研修担当者）

#### GET /interviews
面談一覧表示

**認証:** 管理者・研修担当者  
**クエリパラメータ:**
- `project_id`: UUID
- `student_id`: UUID
- `project_participant_id`: UUID
- `status`: enum(scheduled, completed, cancelled)
- `date_from`: date
- `date_to`: date

#### GET /interviews/new
面談作成フォーム

**認証:** 管理者・研修担当者

#### POST /interviews
面談作成

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
project_participant_id: UUID
scheduled_at: datetime
_token: string
```

#### GET /interviews/:id
面談詳細表示

**認証:** 管理者・研修担当者

#### GET /interviews/:id/edit
面談編集フォーム

**認証:** 管理者・研修担当者

#### PUT /interviews/:id
面談更新

**認証:** 管理者・研修担当者  
**リクエスト（Form）:**
```
scheduled_at?: datetime
status?: enum(scheduled, completed, cancelled)
notes?: string (Markdown)
_token: string
```

#### DELETE /interviews/:id
面談削除

**認証:** 管理者・研修担当者

---

### 定例会管理（研修講師・管理者・研修担当者）

#### GET /meetings
定例会一覧表示

**認証:** 全ユーザー  
**クエリパラメータ:**
- `project_id`: UUID
- `date_from`: date
- `date_to`: date

#### GET /meetings/new
定例会作成フォーム

**認証:** 研修講師・管理者・研修担当者

#### POST /meetings
定例会作成

**認証:** 研修講師・管理者・研修担当者  
**リクエスト（Form）:**
```
project_id: UUID
title: string
scheduled_at: datetime
recurrence_type: enum(none, weekly, biweekly)
recurrence_end_date?: date
instructor_id?: UUID
_token: string
```

#### GET /meetings/:id
定例会詳細表示

**認証:** 全ユーザー

#### GET /meetings/:id/edit
定例会編集フォーム

**認証:** 研修講師・管理者・研修担当者

#### PUT /meetings/:id
定例会更新

**認証:** 研修講師・管理者・研修担当者  
**リクエスト（Form）:**
```
title?: string
scheduled_at?: datetime
recurrence_type?: enum(none, weekly, biweekly)
recurrence_end_date?: date
instructor_id?: UUID
notes?: string (Markdown)
_token: string
```

#### DELETE /meetings/:id
定例会削除

**認証:** 研修講師・管理者・研修担当者

---

## 必要最小限のJSON APIエンドポイント

### HTMX部分更新用（必要最小限）

#### POST /api/materials/validate
教材URLの事前バリデーション（ライブバリデーション用）

**認証:** 管理者・研修担当者  
**リクエスト（JSON）:**
```json
{
  "url": "string"
}
```

**レスポンス（JSON）:**
```json
{
  "valid": true,
  "domain": "example.com"
}
```

#### GET /api/students/search
受講者検索（オートコンプリート用）

**認証:** 管理者・研修担当者  
**クエリパラメータ:**
- `q`: string (検索キーワード)
- `company_id`: UUID

**レスポンス（JSON）:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "name": "受講者名",
      "email": "email@example.com",
      "organization": "所属組織"
    }
  ]
}
```

#### GET /api/materials/search
教材検索（研修コース作成時の教材選択用）

**認証:** 管理者・研修担当者  
**クエリパラメータ:**
- `q`: string

**レスポンス（JSON）:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "title": "教材タイトル",
      "url": "https://example.com",
      "domain": "example.com"
    }
  ]
}
```

---

## エラーレスポンス

### HTML エラーページ

#### 404 Not Found
```
Status: 404 Not Found
Content-Type: text/html
[404エラーページ HTML]
```

#### 403 Forbidden
```
Status: 403 Forbidden
Content-Type: text/html
[403エラーページ HTML]
```

#### 500 Internal Server Error
```
Status: 500 Internal Server Error
Content-Type: text/html
[500エラーページ HTML]
```

### JSON エラーレスポンス（API使用時）

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "入力データにエラーがあります",
    "details": {
      "field_errors": [
        {
          "field": "email",
          "message": "有効なメールアドレスを入力してください",
          "code": "INVALID_EMAIL"
        }
      ]
    }
  }
}
```

---

## ヘルスチェック

#### GET /health
システムヘルスチェック

**認証:** 不要  
**レスポンス（JSON）:**
```json
{
  "status": "healthy",
  "checks": {
    "database": {
      "status": "up",
      "response_time_ms": 5
    },
    "session_store": {
      "status": "up"
    }
  },
  "timestamp": "2025-01-17T10:00:00Z"
}
```

---

## セキュリティ考慮事項

1. **CSRF保護**: すべての状態変更操作にCSRFトークンが必要
2. **セッション管理**: HttpOnly, Secure Cookieの使用
3. **入力検証**: すべての入力データの検証
4. **認可チェック**: エンドポイントごとの適切な権限確認
5. **監査ログ**: 重要な操作の記録
6. **レート制限**: APIエンドポイントでの過度なリクエスト制限（必要に応じて）

この設計により、サーバーサイドレンダリングを中心としつつ、必要最小限のHTMX・JSON APIで効率的な研修管理システムを構築できます。