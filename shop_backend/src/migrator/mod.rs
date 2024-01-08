mod m20240108_00001_create_client_table;
mod m20240108_00001_create_employee_table;
mod m20240108_00001_create_order_table;
mod m20240108_00001_create_report_table;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240108_00001_create_client_table::Migration),
            Box::new(m20240108_00001_create_report_table::Migration),
            Box::new(m20240108_00001_create_order_table::Migration),
            Box::new(m20240108_00001_create_employee_table::Migration),
        ]
    }
}
