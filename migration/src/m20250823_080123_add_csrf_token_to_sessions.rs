use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Sessions::Table)
                .add_column(
                    ColumnDef::new(Sessions::CsrfToken)
                        .string_len(255)
                        .null()
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Sessions::Table)
                .drop_column(Sessions::CsrfToken)
                .to_owned(),
        )
        .await
    }
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    CsrfToken,
}
