use thiserror::Error;
#[cfg(feature = "rusqlite")]
use rusqlite::{Error as RusqliteError};
#[cfg(feature = "rusqlite")]
use std::fmt::{Error as FmtError};

/// Errors for all Structs and Functions in this Crate.
#[derive(Error, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
#[non_exhaustive]
pub enum Error {
    /// Error used when a [ForeignKey](crate::ForeignKey) has an empty `foreign_table` Name
    #[error("Foreign Table Name cannot be Empty")]
    EmptyForeignTableName,

    /// Error used when a [ForeignKey](crate::ForeignKey) has an empty `foreign_column` Name
    #[error("Foreign Column Name cannot be Empty")]
    EmptyForeignColumnName,

    /// Error used when a [Column](crate::Column) has an empty `name`
    #[error("Column Name cannot be Empty")]
    EmptyColumnName,

    /// Error used when a [Column](crate::Column) has a [PrimaryKey](crate::PrimaryKey) and [ForeignKey](crate::ForeignKey) at the same time
    #[error("Column cannot be a Primary Key and a Foreign Key at the same Time")]
    PrimaryKeyAndForeignKey,

    /// Error used when a [Column](crate::Column) has a [PrimaryKey](crate::PrimaryKey) and [Unique](crate::Unique) at the same time
    /// (Primary Key implies Unique, see [here](https://www.sqlite.org/lang_createtable.html#unique_constraints))
    #[error("Primary Key implies Unique")]
    PrimaryKeyAndUnique,
    
    /// Error used when a [Generated](crate::Generated) [Column](crate::Column) has an empty Expression
    #[error("Generator Expression cannot be empty")]
    EmptyGeneratorExpr,
    
    /// Error used when a [View](crate::View) has an empty `select`
    #[error("View Select cannot be empty")]
    EmptyViewSelect,

    /// Error used when a [Table](crate::Table) has an empty `name`
    #[error("Table Name cannot be Empty")]
    EmptyTableName,

    /// Error used when a [View](crate::View) has an empty `name`
    #[error("View Name cannot be Empty")]
    EmptyViewName,

    /// Error used when a [Table](crate::Table) has no [Columns](crate::Column)
    #[error("Table must have Columns")]
    NoColumns,

    /// Error used when a [Table](crate::Table) has multiple [Columns](crate::Column) with a [PrimaryKey](crate::PrimaryKey)
    #[error("Table can only have one Primary Key")]
    MultiplePrimaryKeys,

    /// Error used when a table marked as `without_rowid` has no [Column](crate::Column) with a [PrimaryKey](crate::PrimaryKey)
    /// (`WITHOUT ROWID` tables need a Primary Key, see [here](https://www.sqlite.org/withoutrowid.html#differences_from_ordinary_rowid_tables))
    #[error("Tables without rowid must have one Primary Key")]
    WithoutRowidNoPrimaryKey,

    /// Error used when a [Schema](crate::Schema) has no [Tables](crate::Table) or [Views](crate::View)
    #[error("Schema must not be empty")]
    EmptySchema,
}

#[cfg(feature = "rusqlite")]
#[derive(Error, Debug, PartialEq)]
pub enum CheckError {
    /// Error pass though when a [RusqliteError](rusqlite::Error) occurs
    #[error(transparent)]
    RusqliteError(#[from] RusqliteError),

    /// Error pass though a [FmtError](std::fmt::Error) occurs
    #[error(transparent)]
    FmtError(#[from] FmtError),
}

#[cfg(feature = "rusqlite")]
#[derive(Error, Debug, PartialEq)]
pub enum ExecError {
    /// Error pass though when a [RusqliteError](rusqlite::Error) occurs
    #[error(transparent)]
    RusqliteError(#[from] RusqliteError),

    /// Error pass though when a [Error] occurs
    #[error(transparent)]
    Error(#[from] Error),
}

/// Result type used in this crate, Error type is [enum@Error]
pub type Result<T, E = Error> = std::result::Result<T, E>;
