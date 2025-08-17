use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // UUID拡張の有効化
        let db = m.get_connection();
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .await?;

        // セッション管理テーブル
        m.create_table(
            Table::create()
                .table(Sessions::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Sessions::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(Sessions::UserId).integer().not_null())
                .col(
                    ColumnDef::new(Sessions::SessionToken)
                        .string()
                        .not_null()
                        .unique_key(),
                )
                .col(
                    ColumnDef::new(Sessions::ExpiresAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Sessions::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Sessions::LastAccessedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_sessions_user_id")
                        .from(Sessions::Table, Sessions::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

        // 企業テーブル
        m.create_table(
            Table::create()
                .table(Companies::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Companies::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(Companies::Name).string().not_null())
                .col(ColumnDef::new(Companies::ContactPerson).string().not_null())
                .col(ColumnDef::new(Companies::ContactEmail).string().not_null())
                .col(ColumnDef::new(Companies::ChatLink).text())
                .col(
                    ColumnDef::new(Companies::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Companies::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await?;

        // 受講者テーブル
        m.create_table(
            Table::create()
                .table(Students::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Students::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(Students::Name).string().not_null())
                .col(ColumnDef::new(Students::Email).string().not_null())
                .col(ColumnDef::new(Students::CompanyId).uuid().not_null())
                .col(ColumnDef::new(Students::RoleType).string().not_null())
                .col(ColumnDef::new(Students::Organization).string().not_null())
                .col(
                    ColumnDef::new(Students::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Students::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_students_company_id")
                        .from(Students::Table, Students::CompanyId)
                        .to(Companies::Table, Companies::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .to_owned(),
        )
        .await?;

        // 教材テーブル
        m.create_table(
            Table::create()
                .table(Materials::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Materials::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(Materials::Title).string().not_null())
                .col(ColumnDef::new(Materials::Url).text().not_null())
                .col(ColumnDef::new(Materials::Domain).string().not_null())
                .col(ColumnDef::new(Materials::Description).text().not_null())
                .col(
                    ColumnDef::new(Materials::RecommendationLevel)
                        .integer()
                        .not_null(),
                )
                .col(ColumnDef::new(Materials::CreatedBy).integer().not_null())
                .col(
                    ColumnDef::new(Materials::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Materials::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_materials_created_by")
                        .from(Materials::Table, Materials::CreatedBy)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .to_owned(),
        )
        .await?;

        // 研修コーステーブル
        m.create_table(
            Table::create()
                .table(Trainings::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Trainings::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(Trainings::Title).string().not_null())
                .col(ColumnDef::new(Trainings::Description).text().not_null())
                .col(ColumnDef::new(Trainings::Prerequisites).text().not_null())
                .col(ColumnDef::new(Trainings::Goals).text().not_null())
                .col(
                    ColumnDef::new(Trainings::CompletionCriteria)
                        .text()
                        .not_null(),
                )
                .col(ColumnDef::new(Trainings::CompanyId).uuid())
                .col(ColumnDef::new(Trainings::CreatedBy).integer().not_null())
                .col(
                    ColumnDef::new(Trainings::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Trainings::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_trainings_company_id")
                        .from(Trainings::Table, Trainings::CompanyId)
                        .to(Companies::Table, Companies::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_trainings_created_by")
                        .from(Trainings::Table, Trainings::CreatedBy)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .to_owned(),
        )
        .await?;

        // 研修コース-教材関連テーブル
        m.create_table(
            Table::create()
                .table(TrainingMaterials::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(TrainingMaterials::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(
                    ColumnDef::new(TrainingMaterials::TrainingId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(TrainingMaterials::MaterialId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(TrainingMaterials::PeriodDays)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(TrainingMaterials::OrderIndex)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(TrainingMaterials::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_training_materials_training_id")
                        .from(TrainingMaterials::Table, TrainingMaterials::TrainingId)
                        .to(Trainings::Table, Trainings::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_training_materials_material_id")
                        .from(TrainingMaterials::Table, TrainingMaterials::MaterialId)
                        .to(Materials::Table, Materials::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

        // 実施研修プロジェクトテーブル
        m.create_table(
            Table::create()
                .table(Projects::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Projects::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(Projects::TrainingId).uuid().not_null())
                .col(ColumnDef::new(Projects::CompanyId).uuid().not_null())
                .col(ColumnDef::new(Projects::Title).string().not_null())
                .col(ColumnDef::new(Projects::StartDate).date().not_null())
                .col(ColumnDef::new(Projects::EndDate).date().not_null())
                .col(ColumnDef::new(Projects::CreatedBy).integer().not_null())
                .col(
                    ColumnDef::new(Projects::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Projects::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_projects_training_id")
                        .from(Projects::Table, Projects::TrainingId)
                        .to(Trainings::Table, Trainings::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_projects_company_id")
                        .from(Projects::Table, Projects::CompanyId)
                        .to(Companies::Table, Companies::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_projects_created_by")
                        .from(Projects::Table, Projects::CreatedBy)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .to_owned(),
        )
        .await?;

        // プロジェクト参加者テーブル
        m.create_table(
            Table::create()
                .table(ProjectParticipants::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ProjectParticipants::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(
                    ColumnDef::new(ProjectParticipants::ProjectId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProjectParticipants::StudentId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProjectParticipants::Status)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProjectParticipants::AllInterviewsCompleted)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(
                    ColumnDef::new(ProjectParticipants::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(ProjectParticipants::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_project_participants_project_id")
                        .from(ProjectParticipants::Table, ProjectParticipants::ProjectId)
                        .to(Projects::Table, Projects::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_project_participants_student_id")
                        .from(ProjectParticipants::Table, ProjectParticipants::StudentId)
                        .to(Students::Table, Students::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

        // 個別面談テーブル
        m.create_table(
            Table::create()
                .table(Interviews::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Interviews::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(
                    ColumnDef::new(Interviews::ProjectParticipantId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Interviews::InterviewerId)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Interviews::ScheduledAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Interviews::Status)
                        .string()
                        .not_null()
                        .default("scheduled"),
                )
                .col(ColumnDef::new(Interviews::Notes).text())
                .col(
                    ColumnDef::new(Interviews::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Interviews::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_interviews_project_participant_id")
                        .from(Interviews::Table, Interviews::ProjectParticipantId)
                        .to(ProjectParticipants::Table, ProjectParticipants::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_interviews_interviewer_id")
                        .from(Interviews::Table, Interviews::InterviewerId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .to_owned(),
        )
        .await?;

        // 定例会テーブル
        m.create_table(
            Table::create()
                .table(Meetings::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Meetings::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(Meetings::ProjectId).uuid().not_null())
                .col(ColumnDef::new(Meetings::Title).string().not_null())
                .col(
                    ColumnDef::new(Meetings::ScheduledAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Meetings::RecurrenceType)
                        .string()
                        .not_null()
                        .default("none"),
                )
                .col(ColumnDef::new(Meetings::RecurrenceEndDate).date())
                .col(ColumnDef::new(Meetings::InstructorId).integer())
                .col(ColumnDef::new(Meetings::Notes).text())
                .col(ColumnDef::new(Meetings::CreatedBy).integer().not_null())
                .col(
                    ColumnDef::new(Meetings::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Meetings::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_meetings_project_id")
                        .from(Meetings::Table, Meetings::ProjectId)
                        .to(Projects::Table, Projects::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_meetings_instructor_id")
                        .from(Meetings::Table, Meetings::InstructorId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_meetings_created_by")
                        .from(Meetings::Table, Meetings::CreatedBy)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .to_owned(),
        )
        .await?;

        // 監査ログテーブル
        m.create_table(
            Table::create()
                .table(AuditLogs::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AuditLogs::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT uuid_generate_v4()"),
                )
                .col(ColumnDef::new(AuditLogs::UserId).integer())
                .col(ColumnDef::new(AuditLogs::Action).string().not_null())
                .col(ColumnDef::new(AuditLogs::ResourceType).string())
                .col(ColumnDef::new(AuditLogs::ResourceId).uuid())
                .col(ColumnDef::new(AuditLogs::Details).json())
                .col(ColumnDef::new(AuditLogs::IpAddress).string())
                .col(ColumnDef::new(AuditLogs::UserAgent).text())
                .col(
                    ColumnDef::new(AuditLogs::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_audit_logs_user_id")
                        .from(AuditLogs::Table, AuditLogs::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

        // === インデックス作成 ===

        // ユーザー関連インデックス (既存の場合はスキップ)
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);")
            .await?;

        // すべてのインデックスをIF NOT EXISTSで作成
        let indexes = vec![
            // セッション関連
            "CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(session_token);",
            "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);",
            "CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);",

            // 企業・受講者関連
            "CREATE INDEX IF NOT EXISTS idx_companies_name ON companies(name);",
            "CREATE INDEX IF NOT EXISTS idx_students_company_id ON students(company_id);",
            "CREATE INDEX IF NOT EXISTS idx_students_email ON students(email);",

            // 教材関連
            "CREATE INDEX IF NOT EXISTS idx_materials_domain ON materials(domain);",
            "CREATE INDEX IF NOT EXISTS idx_materials_recommendation_level ON materials(recommendation_level);",
            "CREATE INDEX IF NOT EXISTS idx_materials_created_by ON materials(created_by);",

            // 研修コース関連
            "CREATE INDEX IF NOT EXISTS idx_trainings_company_id ON trainings(company_id);",
            "CREATE INDEX IF NOT EXISTS idx_trainings_created_by ON trainings(created_by);",
            "CREATE INDEX IF NOT EXISTS idx_training_materials_training_id ON training_materials(training_id);",
            "CREATE INDEX IF NOT EXISTS idx_training_materials_material_id ON training_materials(material_id);",

            // プロジェクト関連
            "CREATE INDEX IF NOT EXISTS idx_projects_training_id ON projects(training_id);",
            "CREATE INDEX IF NOT EXISTS idx_projects_company_id ON projects(company_id);",
            "CREATE INDEX IF NOT EXISTS idx_projects_created_by ON projects(created_by);",
            "CREATE INDEX IF NOT EXISTS idx_project_participants_project_id ON project_participants(project_id);",
            "CREATE INDEX IF NOT EXISTS idx_project_participants_student_id ON project_participants(student_id);",
            "CREATE INDEX IF NOT EXISTS idx_project_participants_status ON project_participants(status);",

            // 面談関連
            "CREATE INDEX IF NOT EXISTS idx_interviews_project_participant_id ON interviews(project_participant_id);",
            "CREATE INDEX IF NOT EXISTS idx_interviews_interviewer_id ON interviews(interviewer_id);",
            "CREATE INDEX IF NOT EXISTS idx_interviews_scheduled_at ON interviews(scheduled_at);",
            "CREATE INDEX IF NOT EXISTS idx_interviews_status ON interviews(status);",

            // 定例会関連
            "CREATE INDEX IF NOT EXISTS idx_meetings_project_id ON meetings(project_id);",
            "CREATE INDEX IF NOT EXISTS idx_meetings_instructor_id ON meetings(instructor_id);",
            "CREATE INDEX IF NOT EXISTS idx_meetings_scheduled_at ON meetings(scheduled_at);",
            "CREATE INDEX IF NOT EXISTS idx_meetings_created_by ON meetings(created_by);",

            // 監査ログ関連
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);",
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);",
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);",
        ];

        for index_sql in indexes {
            db.execute_unprepared(index_sql).await?;
        }

        // === トリガー関数とトリガーの作成 ===

        // updated_at自動更新関数
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = CURRENT_TIMESTAMP;
                RETURN NEW;
            END;
            $$ language 'plpgsql';
        "#,
        )
        .await?;

        // 各テーブルのupdated_atトリガー設定
        let update_triggers = vec![
            "CREATE TRIGGER update_companies_updated_at BEFORE UPDATE ON companies FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            "CREATE TRIGGER update_students_updated_at BEFORE UPDATE ON students FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            "CREATE TRIGGER update_materials_updated_at BEFORE UPDATE ON materials FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            "CREATE TRIGGER update_trainings_updated_at BEFORE UPDATE ON trainings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            "CREATE TRIGGER update_projects_updated_at BEFORE UPDATE ON projects FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            "CREATE TRIGGER update_project_participants_updated_at BEFORE UPDATE ON project_participants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            "CREATE TRIGGER update_interviews_updated_at BEFORE UPDATE ON interviews FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            "CREATE TRIGGER update_meetings_updated_at BEFORE UPDATE ON meetings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
        ];

        for trigger_sql in update_triggers {
            db.execute_unprepared(trigger_sql).await?;
        }

        // プロジェクト参加者企業整合性チェック関数
        db.execute_unprepared(
            r#"
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
        "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TRIGGER check_project_participant_company_trigger
                BEFORE INSERT OR UPDATE ON project_participants
                FOR EACH ROW EXECUTE FUNCTION check_project_participant_company();
        "#,
        )
        .await?;

        // セッションクリーンアップ関数
        db.execute_unprepared(
            r#"
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
        "#,
        )
        .await?;

        // === 一意制約の追加 ===

        let unique_indexes = vec![
            // 学生テーブル: 同一企業内でのメール重複防止
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_students_unique_email_company ON students(email, company_id);",

            // 研修教材テーブル: 同一研修での教材重複防止
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_training_materials_unique_training_material ON training_materials(training_id, material_id);",

            // 研修教材テーブル: 同一研修での順序重複防止
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_training_materials_unique_training_order ON training_materials(training_id, order_index);",

            // プロジェクト参加者テーブル: 同一プロジェクトでの重複参加防止
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_project_participants_unique_project_student ON project_participants(project_id, student_id);",
        ];

        for index_sql in unique_indexes {
            db.execute_unprepared(index_sql).await?;
        }

        // === 初期データの投入 ===

        // 管理者ユーザーを作成（ハッシュ化されたパスワード 'admin123'）
        db.execute_unprepared(r#"
            INSERT INTO users (pid, email, name, password, api_key) VALUES
            (uuid_generate_v4(), 'admin@example.com', 'システム管理者', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewV5oUcmK6UPj2Pi', 'admin-api-key-12345');
        "#).await?;

        // サンプル企業データの投入
        db.execute_unprepared(
            r#"
            INSERT INTO companies (name, contact_person, contact_email) VALUES
            ('株式会社サンプル', '田中太郎', 'tanaka@sample.co.jp'),
            ('テストコーポレーション', '佐藤花子', 'sato@test-corp.co.jp');
        "#,
        )
        .await?;

        // 統計情報の更新
        db.execute_unprepared("ANALYZE;").await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();

        // Drop triggers first
        let drop_triggers = vec![
            "DROP TRIGGER IF EXISTS update_companies_updated_at ON companies;",
            "DROP TRIGGER IF EXISTS update_students_updated_at ON students;",
            "DROP TRIGGER IF EXISTS update_materials_updated_at ON materials;",
            "DROP TRIGGER IF EXISTS update_trainings_updated_at ON trainings;",
            "DROP TRIGGER IF EXISTS update_projects_updated_at ON projects;",
            "DROP TRIGGER IF EXISTS update_project_participants_updated_at ON project_participants;",
            "DROP TRIGGER IF EXISTS update_interviews_updated_at ON interviews;",
            "DROP TRIGGER IF EXISTS update_meetings_updated_at ON meetings;",
            "DROP TRIGGER IF EXISTS check_project_participant_company_trigger ON project_participants;",
        ];

        for trigger_sql in drop_triggers {
            db.execute_unprepared(trigger_sql).await?;
        }

        // Drop functions
        db.execute_unprepared("DROP FUNCTION IF EXISTS update_updated_at_column();")
            .await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS check_project_participant_company();")
            .await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS cleanup_expired_sessions();")
            .await?;

        // Drop tables in reverse order of dependencies
        m.drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Meetings::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Interviews::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(ProjectParticipants::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(TrainingMaterials::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Trainings::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Materials::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Students::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Companies::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Define table and column enums
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    UserId,
    SessionToken,
    ExpiresAt,
    CreatedAt,
    LastAccessedAt,
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
    Name,
    ContactPerson,
    ContactEmail,
    ChatLink,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Students {
    Table,
    Id,
    Name,
    Email,
    CompanyId,
    RoleType,
    Organization,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Materials {
    Table,
    Id,
    Title,
    Url,
    Domain,
    Description,
    RecommendationLevel,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Trainings {
    Table,
    Id,
    Title,
    Description,
    Prerequisites,
    Goals,
    CompletionCriteria,
    CompanyId,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TrainingMaterials {
    Table,
    Id,
    TrainingId,
    MaterialId,
    PeriodDays,
    OrderIndex,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
    TrainingId,
    CompanyId,
    Title,
    StartDate,
    EndDate,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProjectParticipants {
    Table,
    Id,
    ProjectId,
    StudentId,
    Status,
    AllInterviewsCompleted,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Interviews {
    Table,
    Id,
    ProjectParticipantId,
    InterviewerId,
    ScheduledAt,
    Status,
    Notes,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Meetings {
    Table,
    Id,
    ProjectId,
    Title,
    ScheduledAt,
    RecurrenceType,
    RecurrenceEndDate,
    InstructorId,
    Notes,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    UserId,
    Action,
    ResourceType,
    ResourceId,
    Details,
    IpAddress,
    UserAgent,
    CreatedAt,
}
