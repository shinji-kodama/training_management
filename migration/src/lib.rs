#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20250817_135834_create_database_schema;
mod m20250822_072326_add_date_check_constraint_to_projects;
mod m20250823_080123_add_csrf_token_to_sessions;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20250817_135834_create_database_schema::Migration),
            Box::new(m20250822_072326_add_date_check_constraint_to_projects::Migration),
            Box::new(m20250823_080123_add_csrf_token_to_sessions::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}