use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(KvStore::Table)
                    .if_not_exists()
                    .col(uuid(KvStore::Id).primary_key().not_null())
                    .col(string(KvStore::Key).unique_key().not_null())
                    .col(
                        timestamp(KvStore::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(string(KvStore::Value).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(KvStore::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum KvStore {
    Table,
    Id,
    Key,
    Value,
    UpdatedAt,
}
