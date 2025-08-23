use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Add CHECK constraint to ensure end_date >= start_date
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE projects ADD CONSTRAINT chk_projects_date_range CHECK (end_date >= start_date)"
        )
        .await?;
        
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Remove the CHECK constraint
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE projects DROP CONSTRAINT IF EXISTS chk_projects_date_range"
        )
        .await?;
        
        Ok(())
    }
}
