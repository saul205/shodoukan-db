use sea_orm::entity::prelude::*;

#[sea_orm::Model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jmdict_entries")]
pub struct JMDictEntries {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub kanji_readings: 
}