use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum Users {
    Table,
    Role,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Users::Table)
                .add_column(
                    ColumnDef::new(Users::Role)
                        .string_len(20)
                        .not_null()
                        .default("instructor")
                )
                .to_owned(),
        )
        .await?;

        m.create_index(
            Index::create()
                .name("idx_users_role")
                .table(Users::Table)
                .col(Users::Role)
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_index(Index::drop().name("idx_users_role").to_owned())
            .await?;
        
        m.alter_table(
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::Role)
                .to_owned(),
        )
        .await
    }
}
