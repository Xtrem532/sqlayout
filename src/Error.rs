use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Error {

    #[error("Foreign Table Name cannot be Empty")]
    EmptyForeignTableName,

    #[error("Foreign Column Name cannot be Empty")]
    EmptyForeignColumnName,

    #[error("Column Name cannot be Empty")]
    EmptyColumnName,

    #[error("Column cannot be a Primary Key and a Foreign Key at the same Time")]
    PrimaryKeyAndForeignKey,

    #[error("Primary Key implies Unique")]
    PrimaryKeyAndUnique,

    #[error("Table Name cannot be Empty")]
    EmptyTableName,

    #[error("Table must have Columns")]
    NoColumns,

    #[error("Table can only have one Primary Key")]
    MultiplePrimaryKeys,

    #[error("Tables without rowid must have one Primary Key")]
    WithoutRowidNoPrimaryKey,

    #[error("Schema must contain Tables")]
    SchemaWithoutTables
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
