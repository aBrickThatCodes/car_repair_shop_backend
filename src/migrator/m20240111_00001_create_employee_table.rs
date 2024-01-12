use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum Employee {
    Table,
    Id,
    PasswordHash,
    Name,
    Role,
}

#[derive(Iden, EnumIter)]
pub enum Role {
    Table,
    #[iden = "Technician"]
    Technician,
    #[iden = "Mechanic"]
    Mechanic,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Employee::Table)
                    .col(
                        ColumnDef::new(Employee::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Employee::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Employee::Name).string().not_null())
                    .col(
                        ColumnDef::new(Employee::Role)
                            .enumeration(Role::Table, Role::iter().skip(1))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Employee::Table).to_owned())
            .await
    }
}
