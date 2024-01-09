use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::*;

use super::m20240108_00001_create_client_table::Client;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240107_00001_create_order_table"
    }
}

#[derive(Iden)]
pub enum Order {
    Table,
    Id,
    ClientId,
    Finished,
    Service,
}

#[derive(Iden, EnumIter)]
pub enum Service {
    Table,
    #[iden = "Repair"]
    Repair,
    #[iden = "Inspection"]
    Inspection,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Order::Table)
                    .col(
                        ColumnDef::new(Order::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Order::ClientId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-request-client_id")
                            .from(Order::Table, Order::ClientId)
                            .to(Client::Table, Client::Id),
                    )
                    .col(
                        ColumnDef::new(Order::Finished)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Order::Service)
                            .enumeration(Service::Table, Service::iter().skip(1))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Order::Table).to_owned())
            .await
    }
}
