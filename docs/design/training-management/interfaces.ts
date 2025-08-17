// 研修管理システム TypeScript インターフェース定義

// ===== 基本的な共通型 =====

export type UUID = string;
export type ISODate = string; // ISO 8601 format
export type MarkdownText = string;

// ユーザー役割
export enum UserRole {
  ADMIN = 'admin',
  TRAINER = 'trainer', 
  INSTRUCTOR = 'instructor'
}

// 受講者役割タイプ
export enum StudentRoleType {
  STUDENT = 'student',
  COMPANY_ADMIN = 'company_admin'
}

// 面談状態
export enum InterviewStatus {
  SCHEDULED = 'scheduled',
  COMPLETED = 'completed',
  CANCELLED = 'cancelled'
}

// 定例会繰り返しタイプ
export enum MeetingRecurrenceType {
  NONE = 'none',
  WEEKLY = 'weekly',
  BIWEEKLY = 'biweekly'
}

// ===== エンティティインターフェース =====

// ユーザー（研修提供者）
export interface User {
  id: UUID;
  email: string;
  name: string;
  role: UserRole;
  password_hash: string;
  created_at: ISODate;
  updated_at: ISODate;
}

// 教材
export interface Material {
  id: UUID;
  title: string;
  url: string;
  domain: string; // URLから自動抽出
  description: string;
  recommendation_level: number; // 1-5
  created_by: UUID; // User.id
  created_at: ISODate;
  updated_at: ISODate;
}

// 研修コース
export interface Training {
  id: UUID;
  title: string;
  description: string;
  prerequisites: string; // 受講前提条件
  goals: string; // ゴール
  completion_criteria: string; // 完了条件
  company_id: UUID | null; // 企業紐付け（nullの場合は公開）
  created_by: UUID; // User.id
  created_at: ISODate;
  updated_at: ISODate;
}

// 研修コース-教材関連
export interface TrainingMaterial {
  id: UUID;
  training_id: UUID;
  material_id: UUID;
  period_days: number; // 取り組み期間（日）
  order_index: number; // 教材の順序
  created_at: ISODate;
}

// 企業
export interface Company {
  id: UUID;
  name: string;
  contact_person: string; // 担当者
  contact_email: string; // 連絡先
  chat_link: string | null; // チャットリンク
  created_at: ISODate;
  updated_at: ISODate;
}

// 受講者
export interface Student {
  id: UUID;
  name: string;
  email: string;
  company_id: UUID;
  role_type: StudentRoleType;
  organization: string; // 所属組織
  created_at: ISODate;
  updated_at: ISODate;
}

// 実施研修プロジェクト
export interface Project {
  id: UUID;
  training_id: UUID;
  company_id: UUID; // 実施組織
  title: string;
  start_date: ISODate;
  end_date: ISODate;
  created_by: UUID; // User.id
  created_at: ISODate;
  updated_at: ISODate;
}

// プロジェクト参加者
export interface ProjectParticipant {
  id: UUID;
  project_id: UUID;
  student_id: UUID;
  status: number; // 研修の状況(1: failed, 2: poor, 3: average, 4: good, 5: excellent)
  all_interviews_completed: boolean; // 全面談完了フラグ
  created_at: ISODate;
  updated_at: ISODate;
}

// 個別面談
export interface Interview {
  id: UUID;
  project_participant_id: UUID; // ProjectParticipant.id
  interviewer_id: UUID; // User.id
  scheduled_at: ISODate;
  status: InterviewStatus;
  notes: MarkdownText | null; // 面談記録
  created_at: ISODate;
  updated_at: ISODate;
}

// 定例会
export interface Meeting {
  id: UUID;
  project_id: UUID;
  title: string;
  scheduled_at: ISODate;
  recurrence_type: MeetingRecurrenceType;
  recurrence_end_date: ISODate | null; // 繰り返し終了日
  instructor_id: UUID | null; // 任意参加の研修講師
  notes: MarkdownText | null; // 研修記録
  created_by: UUID; // User.id
  created_at: ISODate;
  updated_at: ISODate;
}

// セッション
export interface Session {
  id: UUID;
  user_id: UUID;
  session_token: string;
  expires_at: ISODate;
  created_at: ISODate;
  last_accessed_at: ISODate;
}

// 監査ログ
export interface AuditLog {
  id: UUID;
  user_id: UUID | null;
  action: string;
  resource_type: string;
  resource_id: UUID | null;
  details: Record<string, any>; // JSON
  ip_address: string;
  user_agent: string;
  created_at: ISODate;
}

// ===== API リクエスト/レスポンス型 =====

// 基本APIレスポンス
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: Record<string, any>;
  };
}

// ページネーション情報
export interface PaginationInfo {
  page: number;
  per_page: number;
  total_count: number;
  total_pages: number;
}

// ページネーション付きレスポンス
export interface PaginatedResponse<T> {
  success: boolean;
  data: T[];
  pagination: PaginationInfo;
  error?: {
    code: string;
    message: string;
  };
}

// === 認証関連 ===

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  user: Omit<User, 'password_hash'>;
  session_token: string;
}

// === ユーザー管理 ===

export interface CreateUserRequest {
  email: string;
  name: string;
  password: string;
  role: UserRole;
}

export interface UpdateUserRequest {
  email?: string;
  name?: string;
  password?: string;
  role?: UserRole;
}

export interface UserResponse extends Omit<User, 'password_hash'> {}

// === 教材管理 ===

export interface CreateMaterialRequest {
  title: string;
  url: string;
  description: string;
  recommendation_level: number;
}

export interface UpdateMaterialRequest {
  title?: string;
  url?: string;
  description?: string;
  recommendation_level?: number;
}

export interface MaterialResponse extends Material {}

export interface MaterialWithVisibility extends Material {
  show_recommendation: boolean; // ログイン状態に応じた表示制御
}

// === 研修コース管理 ===

export interface CreateTrainingRequest {
  title: string;
  description: string;
  prerequisites: string;
  goals: string;
  completion_criteria: string;
  company_id?: UUID | null;
  materials: {
    material_id: UUID;
    period_days: number;
    order_index: number;
  }[];
}

export interface UpdateTrainingRequest {
  title?: string;
  description?: string;
  prerequisites?: string;
  goals?: string;
  completion_criteria?: string;
  company_id?: UUID | null;
}

export interface TrainingResponse extends Training {
  materials: (TrainingMaterial & {
    material: Material;
  })[];
  company?: Company;
  created_by_user: Pick<User, 'id' | 'name'>;
}

// === 企業管理 ===

export interface CreateCompanyRequest {
  name: string;
  contact_person: string;
  contact_email: string;
  chat_link?: string;
}

export interface UpdateCompanyRequest {
  name?: string;
  contact_person?: string;
  contact_email?: string;
  chat_link?: string;
}

export interface CompanyResponse extends Company {
  student_count: number;
}

// === 受講者管理 ===

export interface CreateStudentRequest {
  name: string;
  email: string;
  company_id: UUID;
  role_type: StudentRoleType;
  organization: string;
}

export interface UpdateStudentRequest {
  name?: string;
  email?: string;
  company_id?: UUID;
  role_type?: StudentRoleType;
  organization?: string;
}

export interface StudentResponse extends Student {
  company: Pick<Company, 'id' | 'name'>;
}

// === プロジェクト管理 ===

export interface CreateProjectRequest {
  training_id: UUID;
  company_id: UUID;
  title: string;
  start_date: ISODate;
  end_date: ISODate;
  participants: {
    student_id: UUID;
    status?: number; // デフォルト: 3 (average)
  }[];
}

export interface UpdateProjectRequest {
  training_id?: UUID;
  company_id?: UUID;
  title?: string;
  start_date?: ISODate;
  end_date?: ISODate;
}

export interface ProjectResponse extends Project {
  training: Pick<Training, 'id' | 'title'>;
  company: Pick<Company, 'id' | 'name'>;
  participants: StudentResponse[];
  created_by_user: Pick<User, 'id' | 'name'>;
}

// === 面談管理 ===

export interface CreateInterviewRequest {
  project_participant_id: UUID;
  scheduled_at: ISODate;
}

export interface UpdateInterviewRequest {
  scheduled_at?: ISODate;
  status?: InterviewStatus;
  notes?: MarkdownText;
}

export interface InterviewResponse extends Interview {
  project_participant: ProjectParticipant & {
    project: Pick<Project, 'id' | 'title'>;
    student: Pick<Student, 'id' | 'name'>;
  };
  interviewer: Pick<User, 'id' | 'name'>;
}

export interface InterviewAlert {
  project_participant_id: UUID;
  student_id: UUID;
  student_name: string;
  project_title: string;
  message: string;
  requires_next_interview: boolean;
}

// === 定例会管理 ===

export interface CreateMeetingRequest {
  project_id: UUID;
  title: string;
  scheduled_at: ISODate;
  recurrence_type: MeetingRecurrenceType;
  recurrence_end_date?: ISODate;
  instructor_id?: UUID;
}

export interface UpdateMeetingRequest {
  title?: string;
  scheduled_at?: ISODate;
  recurrence_type?: MeetingRecurrenceType;
  recurrence_end_date?: ISODate;
  instructor_id?: UUID;
  notes?: MarkdownText;
}

export interface MeetingResponse extends Meeting {
  project: Pick<Project, 'id' | 'title'>;
  instructor?: Pick<User, 'id' | 'name'>;
  created_by_user: Pick<User, 'id' | 'name'>;
}

// === フォーム バリデーション ===

export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export interface FormValidationResponse {
  valid: boolean;
  errors: ValidationError[];
}

// === 検索・フィルタ ===

export interface MaterialSearchParams {
  q?: string; // 検索キーワード
  domain?: string; // ドメインフィルタ
  recommendation_level?: number; // おすすめ度フィルタ
  page?: number;
  per_page?: number;
}

export interface TrainingSearchParams {
  q?: string;
  company_id?: UUID;
  created_by?: UUID;
  page?: number;
  per_page?: number;
}

export interface ProjectSearchParams {
  q?: string;
  training_id?: UUID;
  company_id?: UUID;
  status?: 'active' | 'completed' | 'upcoming';
  participant_status?: number; // 参加者の研修状況フィルタ
  page?: number;
  per_page?: number;
}

// === ダッシュボード ===

export interface DashboardStats {
  total_users: number;
  total_materials: number;
  total_trainings: number;
  total_companies: number;
  total_students: number;
  active_projects: number;
  upcoming_interviews: number;
  recent_activities: ActivityItem[];
}

export interface ActivityItem {
  id: UUID;
  type: 'user_created' | 'material_added' | 'training_created' | 'project_started' | 'interview_completed';
  description: string;
  user_name: string;
  created_at: ISODate;
}

// === HTMX 部分更新レスポンス ===

export interface HTMXFragment {
  html: string;
  trigger?: string; // HX-Trigger ヘッダー値
}

// === エラー定義 ===

export interface ErrorDetails {
  VALIDATION_ERROR: {
    code: 'VALIDATION_ERROR';
    message: string;
    details: {
      field_errors: ValidationError[];
    };
  };
  NOT_FOUND: {
    code: 'NOT_FOUND';
    message: string;
  };
  UNAUTHORIZED: {
    code: 'UNAUTHORIZED';
    message: string;
  };
  FORBIDDEN: {
    code: 'FORBIDDEN';
    message: string;
  };
  CONFLICT: {
    code: 'CONFLICT';
    message: string;
    details?: {
      conflicting_resource?: string;
    };
  };
  INTERNAL_ERROR: {
    code: 'INTERNAL_ERROR';
    message: string;
  };
}

// === セッション管理 ===

export interface SessionInfo {
  user: UserResponse;
  expires_at: ISODate;
  csrf_token: string;
}

// === 設定・環境 ===

export interface AppConfig {
  app_name: string;
  version: string;
  environment: 'development' | 'staging' | 'production';
  features: {
    registration_enabled: boolean;
    htmx_enhanced: boolean;
    audit_logging: boolean;
  };
}

// === ヘルスチェック ===

export interface HealthCheckResponse {
  status: 'healthy' | 'unhealthy';
  checks: {
    database: {
      status: 'up' | 'down';
      response_time_ms?: number;
    };
    session_store: {
      status: 'up' | 'down';
    };
  };
  timestamp: ISODate;
}