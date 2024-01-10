use sea_orm_migration::prelude::*;

use super::{
    m20240111_00001_create_client_table::Client, m20240111_00001_create_order_table::Order,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240107_00001_create_report_table"
    }
}

#[derive(Iden)]
pub enum Report {
    Table,
    Id,
    ClientId,
    OrderId,
    Cost,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Report::Table)
                    .col(
                        ColumnDef::new(Report::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Report::ClientId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report-client_id")
                            .from(Report::Table, Report::ClientId)
                            .to(Client::Table, Client::Id),
                    )
                    .col(ColumnDef::new(Report::OrderId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report-request_id")
                            .from(Report::Table, Report::OrderId)
                            .to(Order::Table, Order::Id),
                    )
                    .col(ColumnDef::new(Report::Cost).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Report::Table).to_owned())
            .await
    }
}
