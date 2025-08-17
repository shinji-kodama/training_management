-- 研修管理システム データベーススキーマ
-- PostgreSQL 15+ 対応

-- UUID拡張の有効化
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ===== ユーザー管理 =====

-- ユーザー（研修提供者）テーブル
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('admin', 'trainer', 'instructor')),
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- セッション管理テーブル
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_accessed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ===== 企業管理 =====

-- 企業テーブル
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    contact_person VARCHAR(255) NOT NULL,
    contact_email VARCHAR(255) NOT NULL,
    chat_link TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 受講者テーブル
CREATE TABLE students (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    role_type VARCHAR(20) NOT NULL CHECK (role_type IN ('student', 'company_admin')),
    organization VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(email, company_id) -- 同一企業内でのメール重複防止
);

-- ===== 教材・研修管理 =====

-- 教材テーブル
CREATE TABLE materials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    domain VARCHAR(255) NOT NULL, -- URLから自動抽出
    description TEXT NOT NULL,
    recommendation_level INTEGER NOT NULL CHECK (recommendation_level BETWEEN 1 AND 5),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 研修コーステーブル
CREATE TABLE trainings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    prerequisites TEXT NOT NULL, -- 受講前提条件
    goals TEXT NOT NULL, -- ゴール
    completion_criteria TEXT NOT NULL, -- 完了条件
    company_id UUID REFERENCES companies(id) ON DELETE SET NULL, -- NULL=公開、UUID=企業限定
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 研修コース-教材関連テーブル
CREATE TABLE training_materials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    training_id UUID NOT NULL REFERENCES trainings(id) ON DELETE CASCADE,
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    period_weeks INTEGER NOT NULL CHECK (period_weeks > 0), -- 取り組み期間（週）
    order_index INTEGER NOT NULL CHECK (order_index >= 0), -- 教材の順序
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(training_id, material_id), -- 同一研修での教材重複防止
    UNIQUE(training_id, order_index) -- 同一研修での順序重複防止
);

-- ===== プロジェクト管理 =====

-- 実施研修プロジェクトテーブル
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    training_id UUID NOT NULL REFERENCES trainings(id) ON DELETE RESTRICT,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE RESTRICT, -- 実施組織
    title VARCHAR(255) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    CHECK (end_date >= start_date) -- 終了日は開始日以降
);

-- プロジェクト参加者テーブル
CREATE TABLE project_participants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(project_id, student_id) -- 同一プロジェクトでの重複参加防止
);

-- ===== 面談管理 =====

-- 個別面談テーブル
CREATE TABLE interviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    interviewer_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    scheduled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'completed', 'cancelled')),
    notes TEXT, -- Markdown形式の面談記録
    all_interviews_completed BOOLEAN NOT NULL DEFAULT FALSE, -- 全面談完了フラグ
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ===== 定例会管理 =====

-- 定例会テーブル
CREATE TABLE meetings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    scheduled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    recurrence_type VARCHAR(20) NOT NULL DEFAULT 'none' CHECK (recurrence_type IN ('none', 'weekly', 'biweekly')),
    recurrence_end_date DATE, -- 繰り返し終了日
    instructor_id UUID REFERENCES users(id) ON DELETE SET NULL, -- 任意参加の研修講師
    notes TEXT, -- Markdown形式の研修記録
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- 繰り返し設定がある場合は終了日が必須
    CHECK (
        (recurrence_type = 'none') OR 
        (recurrence_type != 'none' AND recurrence_end_date IS NOT NULL)
    )
);

-- ===== 監査・ログ =====

-- 監査ログテーブル
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL, -- 'login', 'create_material', 'update_training', etc.
    resource_type VARCHAR(50), -- 'user', 'material', 'training', etc.
    resource_id UUID,
    details JSONB, -- 詳細情報（JSON形式）
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ===== インデックス =====

-- ユーザー関連
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);

-- セッション関連
CREATE INDEX idx_sessions_token ON sessions(session_token);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- 企業・受講者関連
CREATE INDEX idx_companies_name ON companies(name);
CREATE INDEX idx_students_company_id ON students(company_id);
CREATE INDEX idx_students_email ON students(email);

-- 教材関連
CREATE INDEX idx_materials_domain ON materials(domain);
CREATE INDEX idx_materials_recommendation_level ON materials(recommendation_level);
CREATE INDEX idx_materials_created_by ON materials(created_by);
CREATE INDEX idx_materials_title ON materials USING gin(to_tsvector('japanese', title));

-- 研修コース関連
CREATE INDEX idx_trainings_company_id ON trainings(company_id);
CREATE INDEX idx_trainings_created_by ON trainings(created_by);
CREATE INDEX idx_trainings_title ON trainings USING gin(to_tsvector('japanese', title));
CREATE INDEX idx_training_materials_training_id ON training_materials(training_id);
CREATE INDEX idx_training_materials_material_id ON training_materials(material_id);
CREATE INDEX idx_training_materials_order ON training_materials(training_id, order_index);

-- プロジェクト関連
CREATE INDEX idx_projects_training_id ON projects(training_id);
CREATE INDEX idx_projects_company_id ON projects(company_id);
CREATE INDEX idx_projects_created_by ON projects(created_by);
CREATE INDEX idx_projects_dates ON projects(start_date, end_date);
CREATE INDEX idx_project_participants_project_id ON project_participants(project_id);
CREATE INDEX idx_project_participants_student_id ON project_participants(student_id);

-- 面談関連
CREATE INDEX idx_interviews_project_id ON interviews(project_id);
CREATE INDEX idx_interviews_student_id ON interviews(student_id);
CREATE INDEX idx_interviews_interviewer_id ON interviews(interviewer_id);
CREATE INDEX idx_interviews_scheduled_at ON interviews(scheduled_at);
CREATE INDEX idx_interviews_status ON interviews(status);

-- 定例会関連
CREATE INDEX idx_meetings_project_id ON meetings(project_id);
CREATE INDEX idx_meetings_instructor_id ON meetings(instructor_id);
CREATE INDEX idx_meetings_scheduled_at ON meetings(scheduled_at);
CREATE INDEX idx_meetings_created_by ON meetings(created_by);

-- 監査ログ関連
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- ===== トリガー関数 =====

-- updated_at自動更新関数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- updated_atトリガー設定
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_companies_updated_at BEFORE UPDATE ON companies FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_students_updated_at BEFORE UPDATE ON students FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_materials_updated_at BEFORE UPDATE ON materials FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_trainings_updated_at BEFORE UPDATE ON trainings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_projects_updated_at BEFORE UPDATE ON projects FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_interviews_updated_at BEFORE UPDATE ON interviews FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_meetings_updated_at BEFORE UPDATE ON meetings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- セッション最終アクセス時刻更新関数
CREATE OR REPLACE FUNCTION update_session_last_accessed()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_accessed_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- セッション更新トリガー（SELECTでは発動しないため、アプリケーション側で更新）

-- ===== セッションクリーンアップ関数 =====

-- 期限切れセッション削除関数
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM sessions WHERE expires_at < CURRENT_TIMESTAMP;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ language 'plpgsql';

-- ===== データ整合性制約 =====

-- プロジェクト参加者は同一企業の受講者のみ
CREATE OR REPLACE FUNCTION check_project_participant_company()
RETURNS TRIGGER AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM projects p 
        JOIN students s ON s.id = NEW.student_id 
        WHERE p.id = NEW.project_id AND p.company_id = s.company_id
    ) THEN
        RAISE EXCEPTION 'Student must belong to the same company as the project';
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER check_project_participant_company_trigger
    BEFORE INSERT OR UPDATE ON project_participants
    FOR EACH ROW EXECUTE FUNCTION check_project_participant_company();

-- 面談の受講者はプロジェクト参加者である必要がある
CREATE OR REPLACE FUNCTION check_interview_participant()
RETURNS TRIGGER AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM project_participants 
        WHERE project_id = NEW.project_id AND student_id = NEW.student_id
    ) THEN
        RAISE EXCEPTION 'Student must be a participant of the project for interviews';
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER check_interview_participant_trigger
    BEFORE INSERT OR UPDATE ON interviews
    FOR EACH ROW EXECUTE FUNCTION check_interview_participant();

-- ===== 初期データ =====

-- 管理者ユーザーの作成（パスワード: 'admin123' - 本番環境では変更必須）
INSERT INTO users (email, name, role, password_hash) VALUES 
('admin@example.com', 'システム管理者', 'admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewV5oUcmK6UPj2Pi');

-- サンプル企業データ
INSERT INTO companies (name, contact_person, contact_email) VALUES 
('株式会社サンプル', '田中太郎', 'tanaka@sample.co.jp'),
('テストコーポレーション', '佐藤花子', 'sato@test-corp.co.jp');

-- ===== ビュー =====

-- アクティブなプロジェクト一覧ビュー
CREATE VIEW active_projects AS
SELECT 
    p.*,
    t.title as training_title,
    c.name as company_name,
    u.name as created_by_name,
    COUNT(pp.student_id) as participant_count
FROM projects p
JOIN trainings t ON p.training_id = t.id
JOIN companies c ON p.company_id = c.id
JOIN users u ON p.created_by = u.id
LEFT JOIN project_participants pp ON p.id = pp.project_id
WHERE p.end_date >= CURRENT_DATE
GROUP BY p.id, t.title, c.name, u.name;

-- 今週の面談一覧ビュー
CREATE VIEW this_week_interviews AS
SELECT 
    i.*,
    s.name as student_name,
    u.name as interviewer_name,
    p.title as project_title
FROM interviews i
JOIN students s ON i.student_id = s.id
JOIN users u ON i.interviewer_id = u.id
JOIN projects p ON i.project_id = p.id
WHERE i.scheduled_at >= date_trunc('week', CURRENT_TIMESTAMP)
  AND i.scheduled_at < date_trunc('week', CURRENT_TIMESTAMP) + interval '1 week'
  AND i.status = 'scheduled';

-- 教材の利用統計ビュー
CREATE VIEW material_usage_stats AS
SELECT 
    m.*,
    COUNT(tm.training_id) as used_in_trainings,
    AVG(tm.period_weeks) as avg_period_weeks
FROM materials m
LEFT JOIN training_materials tm ON m.id = tm.material_id
GROUP BY m.id;

-- ===== 権限設定 =====

-- アプリケーション用ユーザーの作成（本番環境用）
-- CREATE USER training_app WITH PASSWORD 'secure_password_here';
-- GRANT CONNECT ON DATABASE training_management TO training_app;
-- GRANT USAGE ON SCHEMA public TO training_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO training_app;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO training_app;

-- ===== パフォーマンス設定 =====

-- 統計情報の更新
ANALYZE;

-- VACUUM の設定（自動バキュームが有効な場合は不要）
-- 定期的なメンテナンス用