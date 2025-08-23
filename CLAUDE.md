# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## System Overview

This is a ToB (To Business) IT training management system built with Loco.rs framework. It manages comprehensive training operations including student management, material management, training course design, project management, individual interviews, and regular meetings scheduling.

## Key Development Commands

### Database Operations
```bash
# Generate SeaORM entities from database schema
cargo loco db entities

# Run database migrations
cargo loco db migrate

# Check migration status
cargo loco db status

# Reset database (destructive)
cargo loco db reset

# Create new migration
cargo loco generate migration <name>
```

### Development Server
```bash
# Start development server
cargo loco start

# Start with file watching (recommended for development)
cargo loco watch

# Health check
curl http://localhost:5150/_health
```

### Testing
```bash
# Run all tests
cargo test

# Run specific model tests
cargo test models::companies
cargo test students

# Run tests with database URL override
DATABASE_URL="postgres://postgres:password@localhost:6543/training_management" cargo test students

# Run tests with serial execution (for database tests)
cargo test --test mod
```

### Code Generation
```bash
# Generate new model
cargo loco generate model <name>

# Generate new controller
cargo loco generate controller <name>

# View all routes
cargo loco routes

# System health check
cargo loco doctor
```

## Architecture & Structure

### Framework Architecture
- **Framework**: Loco.rs 0.16.3 (Ruby on Rails-inspired Rust framework)
- **Architecture Pattern**: Monolithic MVC + HTMX
- **ORM**: SeaORM 1.1.14 for database operations
- **Database**: PostgreSQL 15+ with comprehensive schema
- **Frontend**: Server-side rendering with Tera templates + HTMX

### Core Directory Structure
```
src/
├── app.rs                 # Main application configuration and hooks
├── controllers/           # HTTP request handlers (auth, dashboard)
├── models/               # Business logic and database models
│   ├── _entities/        # Auto-generated SeaORM entities (DO NOT EDIT)
│   └── *.rs             # Model implementations with business logic
├── views/                # Server-side rendering logic
├── mailers/              # Email templates and logic
├── tasks/                # Background tasks
└── workers/              # Background job workers

tests/
├── models/               # Model unit tests
├── requests/             # Integration tests for HTTP endpoints
└── snapshots/            # Insta snapshot test files

migration/                # Database migration files
docs/                     # Design documents and implementation notes
```

### Database Schema Design
The system uses a comprehensive PostgreSQL schema with:
- **13 main tables**: users, companies, students, materials, trainings, projects, interviews, meetings, etc.
- **18 foreign key constraints** for referential integrity
- **4 unique constraints** including UNIQUE(email, company_id) for students
- **51+ indexes** for performance optimization
- **9 triggers** for automatic timestamp updates and data consistency

## Model Implementation Patterns

### Standard Model Structure
Each model follows this pattern (see `src/models/companies.rs` and `src/models/students.rs`):

1. **Entity Import**: Import from `_entities/` directory
2. **Validator Struct**: Input validation with `validator` crate
3. **Validatable Implementation**: Connect validator to ActiveModel
4. **ActiveModelBehavior**: Handle UUID generation and validation on save
5. **Model Methods**: Business logic and custom finders

### TDD Development Approach
The project follows Test-Driven Development:
- **Red Phase**: Write failing tests first
- **Green Phase**: Implement minimal code to pass tests
- **Refactor Phase**: Improve code quality while maintaining tests

Test files use Japanese comments for clarity and include detailed purpose descriptions.

### UUID Primary Keys
All models use UUID primary keys that are auto-generated in `before_save()`:
```rust
if insert {
    let mut this = self;
    this.id = ActiveValue::Set(uuid::Uuid::new_v4());
    Ok(this)
}
```

## Testing Strategies

### Model Testing
- Use `boot_test::<App>()` for test setup
- Use `#[serial]` attribute for database tests to prevent conflicts
- Tests include comprehensive Japanese comments explaining purpose and expectations
- Test patterns: basic CRUD, validation, relationships, constraint violations

### Database Testing
- Tests run against development database with DATABASE_URL override
- Foreign key relationships are tested by creating related entities first
- Unique constraints are verified by attempting duplicate data insertion

## Configuration

### Environment Files
- `config/development.yaml`: Loco.rs development configuration
- `config/test.yaml`: Test environment configuration
- `.env`: Environment variables (database URLs, secrets)

### Database Configuration
- Development: `postgres://postgres:password@localhost:6543/training_management`
- Auto-migration enabled in development
- Session-based authentication (no JWT)

## Development Workflow

### Task-Based Development
The project follows a structured task-based approach:
- Tasks are documented in `docs/tasks/training-management-tasks.md`
- Implementation documentation in `docs/implements/TASK-XXX/`
- Each task includes requirements, test cases, and implementation memos

### TDD Commands for Complex Features
For major features, use TDD workflow commands:
```bash
# Define requirements
/tdd-requirements @docs/tasks/training-management-tasks.md TASK-XXX

# Create test cases
/tdd-testcases @docs/tasks/training-management-tasks.md TASK-XXX

# Red phase (failing tests)
/tdd-red @docs/tasks/training-management-tasks.md TASK-XXX

# Green phase (minimal implementation)
/tdd-green @docs/tasks/training-management-tasks.md TASK-XXX

# Refactor phase (code quality)
/tdd-refactor @docs/tasks/training-management-tasks.md TASK-XXX
```

## Key Implementation Notes

### Session-Based Authentication
- Uses session-based authentication rather than JWT
- Session management handled by Loco.rs built-in auth system
- User roles: admin, trainer, instructor

### Entity Relationships
- Companies have many Students (1:many)
- Students belong to Companies (foreign key: company_id)
- Unique constraint: (email, company_id) prevents duplicate emails within same company
- Projects link Trainings to Companies with participant tracking

### Business Logic Patterns
- Validation happens in custom Validator structs
- Business logic goes in Model impl blocks
- Database constraints are mirrored in application validation
- Error handling uses Loco.rs ModelResult types

## Migration Strategy

### Entity Generation
After database schema changes:
1. Run `cargo loco db entities` to regenerate SeaORM entities
2. Update corresponding model files in `src/models/`
3. Add any new modules to `src/models/mod.rs`
4. Run tests to verify functionality

### Testing New Models
1. Create test file in `tests/models/`
2. Add module import to `tests/models/mod.rs`
3. Follow existing test patterns with Japanese documentation
4. Verify foreign key relationships by creating related entities

This codebase emphasizes thorough testing, clear documentation, and systematic development practices following the Loco.rs framework conventions.
