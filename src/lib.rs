//! A Library for generating SQLite-specific SQL to Initialize Databases (as in `CREATE TABLE...`).
//! SQLite Interface agnostic, e.g. can be used with [rusqlite](https://github.com/rusqlite/rusqlite), [sqlite](https://github.com/stainless-steel/sqlite) or any other SQLite Interface.
//!
//! # xml-config
//!
//! todo

//#![warn(missing_docs)]
mod error;

use std::default::Default;
#[cfg(feature = "xml-config")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "xml-config")]
pub use quick_xml::de::{from_str, from_reader};

#[cfg(feature = "rusqlite")]
use rusqlite::{Connection, Rows, Statement, Row};
#[cfg(feature = "rusqlite")]
use std::fmt::Write;
pub use error::{Error, Result};

#[cfg(feature = "rusqlite")]
use crate::error::{CheckError, ExecError};

// region Test Preamble
// these cannot be in the test mod b/c it is necessary for the test trait impls (SQLPart::possibilities)

#[cfg(test)]
fn option_iter<T: Clone>(input: Vec<Box<T>>) -> Vec<Option<T>> {
    let mut ret: Vec<Option<T>> = input.iter().map(|boxed| Some(*boxed.clone())).collect::<Vec<Option<T>>>();
    ret.push(None);
    ret
}

#[cfg(all(test, feature = "xml-config"))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AnySql {
    Schema(Schema),
    Table(Table),
    View(View),
}

// endregion

// region Traits

trait SQLPart {
    fn part_len(&self) -> Result<usize>;

    fn part_str(&self, sql: &mut String) -> Result<()>;

    // todo: for no-std
    // fn part_arr(&self, sql: &mut [u8]) -> Result<()>;

    #[cfg(test)]
    fn possibilities(illegal_variants: bool) -> Vec<Box<Self>>;
}

/// Any struct implementing this trait can be converted into an SQL statement [String].
/// Optionally, the statement can be wrapped in an SQL Transaction and/or guarded against already existing Tables with a `...IF NOT EXISTS...` guard.
pub trait SQLStatement {
    /// Calculates the exact length of the statement as it is currently configured.
    /// Any change to the configuration invalidates previously calculated lengths.
    /// Parameters are the same as in [SQLStatement::build].
    fn len(&mut self, transaction: bool, if_exists: bool) -> Result<usize>;

    /// Builds the SQL Statement as a [String].
    ///
    /// Arguments:
    ///
    /// * `transaction`: Weather the SQL-Statement should be wrapped in a SQL-Transaction
    /// * `if_exists`: Weather the `CREATE TABLE...` statement should include a `...IF NOT EXISTS...` guard
    fn build(&mut self, transaction: bool, if_exists: bool) -> Result<String>;

    // todo: for no-std
    // fn build_arr(&self, arr: &mut [u8], transaction: bool) -> Result<()>;

    /// Executes the [SQLStatement] on the given [Connection]
    #[cfg(feature = "rusqlite")]
    fn execute(&mut self, transaction: bool, if_exists: bool, conn: &Connection) -> Result<(), ExecError> {
        let sql: String = String::with_capacity(self.len(transaction, if_exists)?);
        conn.execute_batch(sql.as_str())?;
        Ok(())
    }

    /// Checks the given DB for deviations from the given SQL.
    /// If the check could not be completed, a [CheckError] is returned.
    /// If the check was completed but found discrepancies, a [String] with a human-readable Description is returned.
    /// If the check was completed and found no discrepancies, [None] is returned.
    #[cfg(feature = "rusqlite")]
    fn check_db(&mut self, conn: &Connection) -> Result<Option<String>, CheckError>;

    #[cfg(all(test, feature = "xml-config"))]
    fn to_any(self) -> AnySql;
}

// endregion

// region SQLiteType

/// Encodes all Column-Datatypes available in SQLite, see [here](https://www.sqlite.org/datatype3.html#type_affinity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "xml-config",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[allow(missing_docs)]
pub enum SQLiteType {
    // ref. https://www.sqlite.org/datatype3.html#type_affinity
    Blob,
    Numeric,
    Integer,
    Real,
    Text,
}

impl Default for SQLiteType {
    fn default() -> Self {
        // ref. https://www.sqlite.org/datatype3.html#affinity_name_examples
        Self::Blob
    }
}

impl SQLPart for SQLiteType {
    fn part_len(&self) -> Result<usize> {
        Ok(match self {
            SQLiteType::Blob => { 4 }
            SQLiteType::Numeric => { 7 }
            SQLiteType::Integer => { 7 }
            SQLiteType::Real => { 4 }
            SQLiteType::Text => { 4 }
        })
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        match self {
            SQLiteType::Blob => { sql.push_str("BLOB") }
            SQLiteType::Numeric => { sql.push_str("NUMERIC") }
            SQLiteType::Integer => { sql.push_str("INTEGER") }
            SQLiteType::Real => { sql.push_str("REAL") }
            SQLiteType::Text => { sql.push_str("TEXT") }
        };
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        vec![Box::new(Self::Blob), Box::new(Self::Numeric), Box::new(Self::Integer), Box::new(Self::Real), Box::new(Self::Text)]
    }
}

// endregion

// region Order

/// [PrimaryKey] direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "xml-config",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[allow(missing_docs)]
pub enum Order {
    Ascending,
    Descending,
}

impl Default for Order {
    fn default() -> Self {
        Self::Ascending
    }
}

impl SQLPart for Order {
    fn part_len(&self) -> Result<usize> {
        Ok(match self {
            Order::Ascending => { 3 }
            Order::Descending => { 4 }
        })
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        match self {
            Order::Ascending => { sql.push_str("ASC") }
            Order::Descending => { sql.push_str("DESC") }
        }
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        vec![Box::new(Self::Ascending), Box::new(Self::Descending)]
    }
}

// endregion

// region OnConflict

/// Reaction to a violated Constraint, used by [PrimaryKey], [NotNull] and [Unique].
/// See also [here](https://www.sqlite.org/lang_conflict.html)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "xml-config",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[allow(missing_docs)]
pub enum OnConflict {
    Rollback,
    Abort,
    Fail,
    Ignore,
    Replace,
}

impl Default for OnConflict {
    fn default() -> Self {
        // ref. https://www.sqlite.org/lang_conflict.html
        Self::Abort
    }
}

impl SQLPart for OnConflict {
    fn part_len(&self) -> Result<usize> {
        Ok(match self {
            OnConflict::Rollback => { 12 + 8 }
            OnConflict::Abort => { 12 + 5 }
            OnConflict::Fail => { 12 + 4 }
            OnConflict::Ignore => { 12 + 6 }
            OnConflict::Replace => { 12 + 7 }
        })
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        match self {
            OnConflict::Rollback => { sql.push_str("ON CONFLICT ROLLBACK") }
            OnConflict::Abort => { sql.push_str("ON CONFLICT ABORT") }
            OnConflict::Fail => { sql.push_str("ON CONFLICT FAIL") }
            OnConflict::Ignore => { sql.push_str("ON CONFLICT IGNORE") }
            OnConflict::Replace => { sql.push_str("ON CONFLICT REPLACE") }
        };
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        vec![Box::new(Self::Rollback), Box::new(Self::Abort), Box::new(Self::Fail), Box::new(Self::Ignore), Box::new(Self::Replace)]
    }
}

// endregion

// region FK OnAction

/// Reaction to an action on a Column with a [ForeignKey]
/// See also [here](https://www.sqlite.org/foreignkeys.html#fk_actions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
#[allow(missing_docs)]
pub enum FKOnAction {
    SetNull,
    SetDefault,
    Cascade,
    Restrict,
    NoAction,
}

impl Default for FKOnAction {
    fn default() -> Self {
        // ref. https://www.sqlite.org/foreignkeys.html#fk_actions
        Self::NoAction
    }
}

impl SQLPart for FKOnAction {
    fn part_len(&self) -> Result<usize> {
        Ok(match self {
            FKOnAction::SetNull => { 8 } // space
            FKOnAction::SetDefault => { 11 } // space
            FKOnAction::Cascade => { 7 }
            FKOnAction::Restrict => { 8 }
            FKOnAction::NoAction => { 9 } // space
        })
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        match self {
            FKOnAction::SetNull => { sql.push_str("SET NULL") }
            FKOnAction::SetDefault => { sql.push_str("SET DEFAULT") }
            FKOnAction::Cascade => { sql.push_str("CASCADE") }
            FKOnAction::Restrict => { sql.push_str("RESTRICT") }
            FKOnAction::NoAction => { sql.push_str("NO ACTION") }
        };
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        vec![Box::new(Self::SetNull), Box::new(Self::SetDefault), Box::new(Self::Cascade), Box::new(Self::Restrict), Box::new(Self::NoAction)]
    }
}

// endregion

// region Generated as

/// Weather a generated column should store its values or compute them on every access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
#[allow(missing_docs)]
pub enum GeneratedAs {
    Virtual,
    Stored,
}

impl Default for GeneratedAs {
    fn default() -> Self {
        Self::Virtual
    }
}

impl SQLPart for GeneratedAs {
    fn part_len(&self) -> Result<usize> {
        Ok(match self {
            GeneratedAs::Virtual => { 7 }
            GeneratedAs::Stored => { 6 }
        })
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        match self {
            GeneratedAs::Virtual => { sql.push_str("VIRTUAL") }
            GeneratedAs::Stored => { sql.push_str("STORED") }
        }
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        vec![Box::new(GeneratedAs::Virtual), Box::new(GeneratedAs::Stored)]
    }
}

// endregion

// region Primary Key

/// Marks a Column as a Primary Key.
/// It is an Error to have more than one Primary Key per [Table] ([Error::MultiplePrimaryKeys]).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct PrimaryKey {
    #[cfg_attr(feature = "xml-config", serde(default, rename = "@order"))]
    sort_order: Order,
    #[cfg_attr(feature = "xml-config", serde(default, rename = "@on_conflict"))]
    on_conflict: OnConflict,
    #[cfg_attr(feature = "xml-config", serde(default, rename = "@autoincrement"))]
    autoincrement: bool, // default false
}

impl PrimaryKey {
    pub fn new(sort_order: Order, on_conflict: OnConflict, autoincrement: bool) -> Self {
        Self {
            sort_order,
            on_conflict,
            autoincrement,
        }
    }

    pub fn set_sort_order(mut self, ord: Order) -> Self {
        self.sort_order = ord;
        self
    }

    pub fn set_on_conflict(mut self, on_conf: OnConflict) -> Self {
        self.on_conflict = on_conf;
        self
    }

    pub fn set_autoincrement(mut self, auto_inc: bool) -> Self {
        self.autoincrement = auto_inc;
        self
    }
}

impl SQLPart for PrimaryKey {
    fn part_len(&self) -> Result<usize> {
        Ok(12 + self.sort_order.part_len()? + 1 + self.on_conflict.part_len()? + self.autoincrement as usize * 14)
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        sql.push_str("PRIMARY KEY ");
        self.sort_order.part_str(sql)?;
        sql.push(' ');
        self.on_conflict.part_str(sql)?;
        if self.autoincrement {
            sql.push_str(" AUTOINCREMENT");
        }
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for so in Order::possibilities(false) {
            for conf in OnConflict::possibilities(false) {
                for auto_inc in [true, false] {
                    ret.push(Box::new(Self::new(*so, *conf, auto_inc)))
                }
            }
        }
        ret
    }
}

// endregion

// region Not Null

/// Marks a [Column] as `NOT NULL`, meaning the Column cannot contain `NULL` values and trying to insert `NULL` values is an Error.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct NotNull {
    #[cfg_attr(feature = "xml-config", serde(default, rename = "@on_conflict"))]
    on_conflict: OnConflict,
}

impl NotNull {
    pub fn new(on_conflict: OnConflict) -> Self {
        Self {
            on_conflict,
        }
    }

    pub fn set_on_conflict(mut self, on_conf: OnConflict) -> Self {
        self.on_conflict = on_conf;
        self
    }
}

impl SQLPart for NotNull {
    fn part_len(&self) -> Result<usize> {
        Ok(9 + self.on_conflict.part_len()?)
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        sql.push_str("NOT NULL ");
        self.on_conflict.part_str(sql)?;
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for conf in OnConflict::possibilities(false) {
            ret.push(Box::new(Self::new(*conf)))
        }
        ret
    }
}

// endregion

// region Unique

/// Marks a [Column] as "Unique", meaning the Column cannot contain the same value twice, and trying to insert a value for the second time is an Error.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct Unique {
    #[cfg_attr(feature = "xml-config", serde(default, rename = "@on_conflict"))]
    on_conflict: OnConflict,
}

impl Unique {
    pub fn new(on_conflict: OnConflict) -> Self {
        Self {
            on_conflict,
        }
    }

    pub fn set_on_conflict(mut self, on_conf: OnConflict) -> Self {
        self.on_conflict = on_conf;
        self
    }
}

impl SQLPart for Unique {
    fn part_len(&self) -> Result<usize> {
        Ok(7 + self.on_conflict.part_len()?)
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        sql.push_str("UNIQUE ");
        self.on_conflict.part_str(sql)?;
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for conf in OnConflict::possibilities(false) {
            ret.push(Box::new(Self::new(*conf)))
        }
        ret
    }
}

// endregion

// region Foreign Key

/// Defines a Foreign Key for a [Column]. It is an Error for the `foreign_table` and `foreign_column` [String]s to be Empty ([Error::EmptyForeignTableName], [Error::EmptyForeignColumnName]).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct ForeignKey {
    #[cfg_attr(feature = "xml-config", serde(rename = "@foreign_table"))]
    foreign_table: String,
    #[cfg_attr(feature = "xml-config", serde(rename = "@foreign_column"))]
    foreign_column: String,
    #[cfg_attr(feature = "xml-config", serde(rename = "@on_delete"))]
    on_delete: Option<FKOnAction>,
    #[cfg_attr(feature = "xml-config", serde(rename = "@on_update"))]
    on_update: Option<FKOnAction>,
    #[cfg_attr(feature = "xml-config", serde(rename = "@deferrable", default))]
    deferrable: bool,
}

impl ForeignKey {
    fn check(&self) -> Result<()> {
        if self.foreign_table.is_empty() {
            return Err(Error::EmptyForeignTableName);
        }
        if self.foreign_column.is_empty() {
            return Err(Error::EmptyForeignColumnName);
        }
        Ok(())
    }

    pub fn new(foreign_table: String, foreign_column: String, on_delete: Option<FKOnAction>, on_update: Option<FKOnAction>, deferrable: bool) -> Self {
        Self {
            foreign_table,
            foreign_column,
            on_delete,
            on_update,
            deferrable,
        }
    }

    pub fn new_default(foreign_table: String, foreign_column: String) -> Self {
        Self {
            foreign_table,
            foreign_column,
            on_delete: Default::default(),
            on_update: Default::default(),
            deferrable: Default::default(),
        }
    }

    pub fn set_foreign_table(mut self, foreign_table: String) -> Self {
        self.foreign_table = foreign_table;
        self
    }

    pub fn set_foreign_column(mut self, foreign_column: String) -> Self {
        self.foreign_column = foreign_column;
        self
    }

    pub fn set_on_delete(mut self, on_delete: Option<FKOnAction>) -> Self {
        self.on_delete = on_delete;
        self
    }

    pub fn set_on_update(mut self, on_update: Option<FKOnAction>) -> Self {
        self.on_update = on_update;
        self
    }

    pub fn set_deferrable(mut self, deferrable: bool) -> Self {
        self.deferrable = deferrable;
        self
    }
}

impl SQLPart for ForeignKey {
    fn part_len(&self) -> Result<usize> {
        self.check()?;

        let on_del_len: usize = if let Some(on_del) = self.on_delete.as_ref() {
            on_del.part_len()? + 1
        } else {
            0
        };

        let on_upd_len: usize = if let Some(on_upd) = self.on_update.as_ref() {
            on_upd.part_len()? + 1
        } else {
            0
        };

        Ok(11 + self.foreign_table.len() + 2 + self.foreign_column.len() + 1 + on_del_len + on_upd_len + self.deferrable as usize * 30)
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        self.check()?;
        sql.push_str("REFERENCES ");
        sql.push_str(self.foreign_table.as_str());
        sql.push_str(" (");
        sql.push_str(self.foreign_column.as_str());
        sql.push(')');

        if let Some(on_del) = self.on_delete.as_ref() {
            sql.push(' ');
            on_del.part_str(sql)?;
        }

        if let Some(on_upd) = self.on_update.as_ref() {
            sql.push(' ');
            on_upd.part_str(sql)?;
        }

        if self.deferrable {
            sql.push_str(" DEFERRABLE INITIALLY DEFERRED");
        }

        Ok(())
    }

    #[cfg(test)]
    fn possibilities(illegal: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for tbl in [if illegal { "".to_string() } else { "test".to_string() }, "test".to_string()] {
            for col in [if illegal { "".to_string() } else { "test".to_string() }, "test".to_string()] {
                for on_del in option_iter(FKOnAction::possibilities(false)) {
                    for on_upd in option_iter(FKOnAction::possibilities(false)) {
                        for defer in [true, false] {
                            ret.push(Box::new(Self::new(tbl.clone(), col.clone(), on_del, on_upd, defer)));
                        }
                    }
                }
            }
        }
        ret
    }
}

// endregion

// region Generated

/// Defines a [Column] as Generated. It is an Error for the `expr` [String] to be Empty ([Error::EmptyGeneratorExpr])
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct Generated {
    #[cfg_attr(feature = "xml-config", serde(rename = "@expr"))]
    expr: String,
    #[cfg_attr(feature = "xml-config", serde(rename = "@as"))]
    generated_as: Option<GeneratedAs>,
}

impl Generated {
    fn check(&self) -> Result<()> {
        if self.expr.is_empty() {
            return Err(Error::EmptyGeneratorExpr);
        }
        Ok(())
    }

    pub fn new(expr: String, generated_as: Option<GeneratedAs>) -> Self {
        Self {
            expr,
            generated_as,
        }
    }

    pub fn new_default(expr: String) -> Self {
        Self {
            expr,
            generated_as: Default::default(),
        }
    }

    pub fn set_expr(mut self, expr: String) -> Self {
        self.expr = expr;
        self
    }

    pub fn set_generated_as(mut self, generated_as: Option<GeneratedAs>) -> Self {
        self.generated_as = generated_as;
        self
    }
}

impl SQLPart for Generated {
    fn part_len(&self) -> Result<usize> {
        self.check()?;

        let generated_as_len: usize = if let Some(generated_as) = self.generated_as.as_ref() {
            generated_as.part_len()? + 1
        } else {
            0
        };

        Ok(21 + self.expr.len() + 1 + generated_as_len)
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        self.check()?;

        sql.push_str("GENERATED ALWAYS AS (");
        sql.push_str(self.expr.as_str());
        sql.push(')');
        if let Some(generated_as) = self.generated_as.as_ref() {
            sql.push(' ');
            generated_as.part_str(sql)?;
        }

        Ok(())
    }

    #[cfg(test)]
    fn possibilities(illegal: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for expr in [if illegal { "".to_string() } else { "test".to_string() }, "test".to_string()] {
            for gen_as in option_iter(GeneratedAs::possibilities(false)) {
                ret.push(Box::new(Self::new(expr.clone(), gen_as)));
            }
        }
        ret
    }
}

// endregion

// region Column

/// This struct Represents a Column in a [Table]. It is an Error for the `name` to be Empty ([Error::EmptyColumnName]).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct Column {
    #[cfg_attr(feature = "xml-config", serde(rename = "@type"))]
    typ: SQLiteType,
    #[cfg_attr(feature = "xml-config", serde(rename = "@name"))]
    name: String,
    #[cfg_attr(feature = "xml-config", serde(skip_serializing_if = "Option::is_none"))]
    pk: Option<PrimaryKey>,
    #[cfg_attr(feature = "xml-config", serde(skip_serializing_if = "Option::is_none"))]
    unique: Option<Unique>,
    #[cfg_attr(feature = "xml-config", serde(skip_serializing_if = "Option::is_none"))]
    fk: Option<ForeignKey>,
    #[cfg_attr(feature = "xml-config", serde(skip_serializing_if = "Option::is_none"))]
    not_null: Option<NotNull>,
    #[cfg_attr(feature = "xml-config", serde(skip_serializing_if = "Option::is_none"))]
    generated: Option<Generated>,
}

impl Column {
    fn check(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::EmptyColumnName);
        }

        if self.pk.is_some() && self.fk.is_some() {
            return Err(Error::PrimaryKeyAndForeignKey);
        }

        if self.pk.is_some() && self.unique.is_some() {
            return Err(Error::PrimaryKeyAndUnique);
        }

        Ok(())
    }

    pub fn new(typ: SQLiteType, name: String, pk: Option<PrimaryKey>, unique: Option<Unique>, fk: Option<ForeignKey>, not_null: Option<NotNull>, generated: Option<Generated>) -> Self {
        Self {
            typ,
            name,
            pk,
            unique,
            fk,
            not_null,
            generated,
        }
    }

    pub fn new_default(name: String) -> Self {
        Self {
            typ: Default::default(),
            name,
            pk: Default::default(),
            unique: Default::default(),
            fk: Default::default(),
            not_null: Default::default(),
            generated: Default::default(),
        }
    }

    pub fn new_typed(typ: SQLiteType, name: String) -> Self {
        Self {
            typ,
            name,
            pk: Default::default(),
            unique: Default::default(),
            fk: Default::default(),
            not_null: Default::default(),
            generated: Default::default(),
        }
    }

    pub fn set_type(mut self, typ: SQLiteType) -> Self {
        self.typ = typ;
        self
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn set_pk(mut self, pk: Option<PrimaryKey>) -> Self {
        self.pk = pk;
        self
    }

    pub fn set_unique(mut self, unique: Option<Unique>) -> Self {
        self.unique = unique;
        self
    }

    pub fn set_fk(mut self, fk: Option<ForeignKey>) -> Self {
        self.fk = fk;
        self
    }
}

impl SQLPart for Column {
    fn part_len(&self) -> Result<usize> {
        self.check()?;
        let pk_len: usize = if let Some(pk) = self.pk.as_ref() {
            pk.part_len()? + 1
        } else {
            0
        };

        let unique_len: usize = if let Some(unique) = self.unique.as_ref() {
            unique.part_len()? + 1
        } else {
            0
        };

        let fk_len: usize = if let Some(fk) = self.fk.as_ref() {
            fk.part_len()? + 1
        } else {
            0
        };

        Ok(self.name.len() + 1 + self.typ.part_len()? + pk_len + unique_len + fk_len)
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        self.check()?;
        sql.push_str(self.name.as_str());
        sql.push(' ');
        self.typ.part_str(sql)?;

        if let Some(pk) = self.pk.as_ref() {
            sql.push(' ');
            pk.part_str(sql)?;
        }

        if let Some(unique) = self.unique.as_ref() {
            sql.push(' ');
            unique.part_str(sql)?;
        }

        if let Some(fk) = self.fk.as_ref() {
            sql.push(' ');
            fk.part_str(sql)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(illegal: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for typ in SQLiteType::possibilities(false) {
            for name in [if illegal { "".to_string() } else { "test".to_string() }, "test".to_string()] {
                for pk in option_iter(PrimaryKey::possibilities(false)) {
                    for unique in option_iter(Unique::possibilities(false)) {
                        for fk in option_iter(ForeignKey::possibilities(illegal)) {
                            for nn in option_iter(NotNull::possibilities(false)) {
                                for gened in option_iter(Generated::possibilities(illegal)) {
                                    if !illegal && pk.is_some() && (fk.is_some() || unique.is_some()) {
                                        continue;
                                    }
                                    ret.push(Box::new(Self::new(*typ.clone(), name.clone(), pk.clone(), unique, fk.clone(), nn, gened)));
                                }
                            }
                        }
                    }
                }
            }
        }
        ret
    }
}

// endregion

// region ViewColumn

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct ViewColumn {
    #[cfg_attr(feature = "xml-config", serde(rename = "@name"))]
    name: String,
}

impl ViewColumn {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }

    pub fn set_name(mut self, name: String) {
        self.name = name;
    }
}

impl SQLPart for ViewColumn {
    fn part_len(&self) -> Result<usize> {
        Ok(self.name.len())
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        sql.push_str(self.name.as_str());
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(_illegal_variants: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        ret.push(Box::new(ViewColumn::new("test".to_string())));
        ret
    }
}

// endregion

// region View

/// Represents an entire View, which may be Part of a wider [Schema] or used standalone.
/// Can be converted into an SQL Statement via the [SQLStatement] Methods.
/// It is an Error for the `name` to be empty ([Error::EmptyViewName]) or the Select to be empty ([Error::EmptyViewSelect]).
#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize), serde(rename = "view"))]
pub struct View {
    #[cfg_attr(feature = "xml-config", serde(rename = "@name"))]
    name: String,
    #[cfg_attr(feature = "xml-config", serde(rename = "@temp", default))]
    temp: bool,
    #[cfg_attr(feature = "xml-config", serde(rename = "column"))]
    columns: Vec<ViewColumn>,
    #[cfg_attr(feature = "xml-config", serde(rename = "@select"))]
    select: String,
    #[cfg_attr(feature = "xml-config", serde(skip))]
    pub(crate) if_exists: bool,
}

impl View {
    fn check(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::EmptyViewName);
        }
        if self.select.is_empty() {
            return Err(Error::EmptyViewSelect);
        }
        Ok(())
    }

    pub fn new(name: String, temp: bool, columns: Vec<ViewColumn>, select: String) -> Self {
        Self {
            name,
            temp,
            columns,
            select,
            if_exists: false,
        }
    }

    pub fn new_default(name: String, select: String) -> Self {
        Self {
            name,
            temp: Default::default(),
            columns: Default::default(),
            select,
            if_exists: Default::default(),
        }
    }

    pub fn add_column(mut self, column: ViewColumn) -> Self {
        self.columns.push(column);
        self
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn set_temp(mut self, temp: bool) -> Self {
        self.temp = temp;
        self
    }

    pub fn set_select(mut self, select: String) -> Self {
        self.select = select;
        self
    }
}

impl SQLPart for View {
    fn part_len(&self) -> Result<usize> {
        self.check()?;
        let pre_coll: usize = 7 // "CREATE "
            + self.temp as usize * 10 // "TEMPORARY "
            + 5 // "VIEW "
            + self.if_exists as usize * 14 // "IF NOT EXISTS "
            + self.name.len()
            + 2; // " ("

        let post_coll: usize = 5 // ") AS "
            + self.select.len();

        if self.columns.is_empty() {
            return Err(Error::NoColumns);
        }
        let coll: usize = {
            let mut cnt: usize = 0;
            for coll in self.columns.iter() {
                cnt += 2; // " ,"
                cnt += coll.part_len()?;
            }
            cnt - 2 // remove last " ,"
        };
        Ok(pre_coll + coll + post_coll)
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        self.check()?;

        sql.push_str("CREATE ");
        if self.temp {
            sql.push_str("TEMPORARY ");
        }
        sql.push_str("VIEW ");
        if self.if_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        sql.push_str(&self.name);
        if !self.columns.is_empty() {
            sql.push_str(" (");
            let mut first: bool = true;
            for coll in self.columns.iter() {
                if !first {
                    sql.push_str(", ");
                } else {
                    first = false;
                }
                coll.part_str(sql)?;
            }
            sql.push(')');
        }
        sql.push_str(" AS ");
        sql.push_str(&self.select);

        Ok(())
    }

    #[cfg(test)]
    fn possibilities(illegal: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for name in [if illegal { "".to_string() } else { "test".to_string() }, "test".to_string()] {
            for select in [if illegal { "".to_string() } else { "test".to_string() }, "test".to_string()] {
                for temp in [true, false] {
                    for col_num in [if illegal { 0 } else { 3 }, 1, 2] {
                        let mut cols: Vec<ViewColumn> = Vec::new();
                        for n in 0..col_num {
                            cols.push(ViewColumn::new(format!("test{}", n)))
                            // todo not all column possibilities
                        }
                        ret.push(Box::new(View::new(name.clone(), temp, cols, select.clone())));
                    }
                }
            }
        }
        ret
    }
}

impl SQLStatement for View {
    fn len(&mut self, transaction: bool, if_exists: bool) -> Result<usize> {
        self.if_exists = if_exists;
        Ok(transaction as usize * 7 + self.part_len()? + 1 + transaction as usize * 5)
    }

    fn build(&mut self, transaction: bool, if_exists: bool) -> Result<String> {
        let mut str = String::with_capacity(self.len(transaction, if_exists)?);
        if transaction {
            str.push_str("BEGIN;\n");
        }
        self.part_str(&mut str)?;
        str.push(';');
        if transaction {
            str.push_str("\nEND;");
        }
        Ok(str)
    }

    #[cfg(feature = "rusqlite")]
    fn check_db(&mut self, _conn: &Connection) -> Result<Option<String>, CheckError> {
        todo!()
    }

    #[cfg(all(test, feature = "xml-config"))]
    fn to_any(self) -> AnySql {
        AnySql::View(self)
    }
}

impl PartialEq<Self> for View {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name || self.temp != other.temp || self.select != other.select {
            return false;
        }
        for columns in self.columns.iter().zip(other.columns.iter()) {
            if columns.0 != columns.1 {
                return false;
            }
        }
        true
    }
}

// endregion

// region Table

/// Represents an entire Table, which may be Part of a wider [Schema] or used standalone.
/// Can be converted into an SQL Statement via the [SQLStatement] Methods.
/// It is an Error for the `name` to be empty ([Error::EmptyTableName]) or the Table itself to be empty ([Error::NoColumns]).
#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize), serde(rename = "table"))]
pub struct Table {
    #[cfg_attr(feature = "xml-config", serde(rename = "@name"))]
    name: String,
    #[cfg_attr(feature = "xml-config", serde(rename = "column"))]
    columns: Vec<Column>,
    #[cfg_attr(feature = "xml-config", serde(rename = "@without_rowid", default))]
    without_rowid: bool,
    #[cfg_attr(feature = "xml-config", serde(rename = "@strict", default))]
    strict: bool,
    #[cfg_attr(feature = "xml-config", serde(skip))]
    pub(crate) if_exists: bool,
}

impl Table {
    fn check(&self) -> Result<()> {
        let mut has_pk: bool = false;
        for col in &self.columns {
            if col.pk.is_some() {
                if has_pk {
                    return Err(Error::MultiplePrimaryKeys);
                } else {
                    has_pk = true;
                }
            }
        }

        if self.name.is_empty() {
            return Err(Error::EmptyTableName);
        }

        if self.columns.is_empty() {
            return Err(Error::NoColumns);
        }

        if self.without_rowid && !has_pk {
            return Err(Error::WithoutRowidNoPrimaryKey);
        }
        Ok(())
    }

    pub fn new(name: String, columns: Vec<Column>, without_rowid: bool, strict: bool) -> Self {
        Self {
            name,
            columns,
            without_rowid,
            strict,
            if_exists: false,
        }
    }

    pub fn new_default(name: String) -> Self {
        Self {
            name,
            columns: Vec::new(),
            without_rowid: Default::default(),
            strict: Default::default(),
            if_exists: Default::default(),
        }
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn add_column(mut self, col: Column) -> Self {
        self.columns.push(col);
        self
    }

    pub fn set_without_rowid(mut self, without_rowid: bool) -> Self {
        self.without_rowid = without_rowid;
        self
    }

    pub fn set_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

impl SQLPart for Table {
    fn part_len(&self) -> Result<usize> {
        self.check()?;
        let mut cols_len: usize = 0;
        for col in &self.columns {
            cols_len += col.part_len()?;
        }
        Ok(
            13  // "CREATE TABLE "
                + self.if_exists as usize * 14 // "IF NOT EXISTS "
                + self.name.len()
                + 2 // " ("
                + cols_len
                + self.columns.len() - 1 // commas for cols, -1 b/c the last doesn't have a comma
                + 1 // ')'
                + self.without_rowid as usize * 14 // " WITHOUT ROWID"
                + ((self.without_rowid && self.strict) as usize) // ','
                + self.strict as usize * 7 // " STRICT"
        )
    }

    fn part_str(&self, sql: &mut String) -> Result<()> {
        self.check()?;

        sql.push_str("CREATE TABLE ");
        if self.if_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        sql.push_str(self.name.as_str());
        sql.push_str(" (");

        let mut needs_comma = false;
        for coll in &self.columns {
            if needs_comma {
                sql.push(',');
            }
            coll.part_str(sql)?;
            needs_comma = true;
        }
        sql.push(')');


        if self.without_rowid {
            sql.push_str(" WITHOUT ROWID");
        }
        if self.without_rowid && self.strict {
            sql.push(',');
        }
        if self.strict {
            sql.push_str(" STRICT");
        }
        Ok(())
    }

    #[cfg(test)]
    fn possibilities(illegal: bool) -> Vec<Box<Self>> {
        let mut ret: Vec<Box<Self>> = Vec::new();
        for name in [if illegal { "".to_string() } else { "test".to_string() }, "test".to_string()] {
            for wo_rowid in [true, false] {
                for col_num in [if illegal { 0 } else { 3 }, 1, 2] {
                    let mut cols: Vec<Column> = Vec::new();
                    for n in 0..col_num {
                        cols.push(Column::new_default(format!("test{}", n)))
                        // todo not all column possibilities
                    }
                    if !illegal && wo_rowid {
                        cols[0].pk = Some(Default::default());
                    }

                    for strict in [true, false] {
                        ret.push(Box::new(Self::new(name.clone(), cols.clone(), wo_rowid, strict)));
                    }
                }
            }
        }
        ret
    }
}

#[cfg(feature = "rusqlite")]
fn check_table(ret: &mut String, num: usize, tbl: &Table, row: &Row) -> Result<(), CheckError> {
    if tbl.name != row.get::<&str, String>("name")? {
        write!(ret, "Table {}: expected name '{}', got '{}'; ", num, tbl.name, row.get::<&str, String>("name")?)?;
    }
    if tbl.without_rowid != row.get::<&str, bool>("wr")? {
        write!(ret, "Table {}: expected without_rowid {}, got {}; ", num, tbl.without_rowid, row.get::<&str, bool>("wr")?)?;
    }
    if tbl.strict != row.get::<&str, bool>("strict")? {
        write!(ret, "Table {}: expected strict {}, got {}; ", num, tbl.strict, row.get::<&str, bool>("strict")?)?;
    }
    if tbl.columns.len() != row.get::<&str, usize>("ncol")? {
        write!(ret, "Table {}: expected number of columns {}, got {}; ", num, tbl.columns.len(), row.get::<&str, usize>("ncol")?)?;
    }
    Ok(())
}

impl SQLStatement for Table {
    fn len(&mut self, transaction: bool, if_exists: bool) -> Result<usize> {
        self.if_exists = if_exists;
        Ok(transaction as usize * 7 + self.part_len()? + 1 + transaction as usize * 5)
    }

    fn build(&mut self, transaction: bool, if_exists: bool) -> Result<String> {
        let mut str = String::with_capacity(self.len(transaction, if_exists)?);
        if transaction {
            str.push_str("BEGIN;\n");
        }
        self.part_str(&mut str)?;
        str.push(';');
        if transaction {
            str.push_str("\nEND;");
        }
        Ok(str)
    }

    #[cfg(feature = "rusqlite")]
    fn check_db(&mut self, conn: &Connection) -> Result<Option<String>, CheckError> {
        let mut ret: String = String::new();

        let mut stmt: Statement = conn.prepare(r#"SELECT name, ncol, wr, strict FROM pragma_table_list() WHERE (schema == "main") AND (type == "table") AND (name == ?1);"#)?;
        let mut rows: Rows = stmt.query([&self.name])?;

        if let Some(row) = rows.next()? {
            check_table(&mut ret, 1, self, row)?;
        } else {
            write!(ret, "Could not find table '{}'", self.name)?;
        }

        if let Some(_) = rows.next()? {
            write!(ret, "Found two Tables with the name '{}'", self.name)?;
        }

        if ret.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ret))
        }
    }

    #[cfg(all(test, feature = "xml-config"))]
    fn to_any(self) -> AnySql {
        AnySql::Table(self)
    }
}

impl PartialEq<Self> for Table {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        if self.without_rowid != other.without_rowid {
            return false;
        }
        if self.strict != other.strict {
            return false;
        }
        if self.columns.len() != other.columns.len() {
            return false;
        }
        for columns in self.columns.iter().zip(other.columns.iter()) {
            if columns.0 != columns.1 {
                return false;
            }
        }
        true
    }
}

// endregion

// region Schema

/// A Schema (or Layout, hence the crate name) encompasses one or more [Table]s.
/// Can be converted into an SQL Statement via the [SQLStatement] Methods.
/// It is an Error for the Schema to be empty ([Error::SchemaWithoutTables]).
#[derive(Debug, Clone, Default, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize), serde(rename = "schema"))]
pub struct Schema {
    #[cfg_attr(feature = "xml-config", serde(rename = "table"), serde(default))]
    tables: Vec<Table>,
    #[cfg_attr(feature = "xml-config", serde(rename = "view"), serde(default))]
    views: Vec<View>,
}

impl Schema {
    fn check(&self) -> Result<()> {
        if self.tables.is_empty() {
            return Err(Error::EmptySchema);
        }
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            views: Vec::new(),
        }
    }

    pub fn add_table(mut self, new_table: Table) -> Self {
        self.tables.push(new_table);
        self
    }

    pub fn add_view(mut self, new_view: View) -> Self {
        self.views.push(new_view);
        self
    }
}

impl SQLStatement for Schema {
    fn len(&mut self, transaction: bool, if_exists: bool) -> Result<usize> {
        self.check()?;
        let mut tables_len: usize = 0;
        for tbl in &mut self.tables {
            tbl.if_exists = if_exists;
            tables_len += tbl.part_len()?;
        }
        let mut views_len: usize = 0;
        for view in &mut self.views {
            view.if_exists = if_exists;
            views_len += view.part_len()?;
        }
        Ok(transaction as usize * 7 + tables_len + self.tables.len() + views_len + self.views.len() + transaction as usize * 5)
    }

    fn build(&mut self, transaction: bool, if_exists: bool) -> Result<String> {
        self.check()?;
        let mut ret: String = String::with_capacity(self.len(transaction, if_exists)?);
        if transaction {
            ret.push_str("BEGIN;\n");
        }

        for tbl in &self.tables {
            tbl.part_str(&mut ret)?;
            ret.push(';');
        }

        if transaction {
            ret.push_str("\nEND;")
        }
        Ok(ret)
    }

    #[cfg(feature = "rusqlite")]
    fn check_db(&mut self, conn: &Connection) -> Result<Option<String>, CheckError> {
        self.tables.sort_unstable_by_key(|table: &Table| table.name.clone()); // todo ugly :(

        let mut ret: String = String::new();

        let mut stmt: Statement = conn.prepare(r#"SELECT name, ncol, wr, strict FROM pragma_table_list() WHERE (schema == "main") AND (type == "table") AND name NOT LIKE "%schema" ORDER BY name;"#)?;
        let mut rows: Rows = stmt.query(())?;


        for (num, table) in self.tables.iter().enumerate() {
            let row: &Row = {
                let raw_row = rows.next()?;
                match raw_row {
                    None => {
                        write!(ret, "Table {}: expected table '{}', got nothing; ", num, table.name)?;
                        break;
                    }
                    Some(row) => { row }
                }
            };
            check_table(&mut ret, num, table, row)?;
        }

        let mut i: usize = self.tables.len();
        while let Some(row) = rows.next()? {
            write!(ret, "Table {}: expected nothing, got table '{}'; ", i, row.get::<&str, String>("name")?)?;
            i += 1;
        }

        if ret.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ret))
        }
    }

    #[cfg(all(test, feature = "xml-config"))]
    fn to_any(self) -> AnySql {
        AnySql::Schema(self)
    }
}

impl PartialEq<Schema> for Schema {
    fn eq(&self, other: &Schema) -> bool {
        if self.tables.len() != other.tables.len() {
            return false;
        }
        for tables in self.tables.iter().zip(other.tables.iter()) {
            if tables.0 != tables.1 {
                return false;
            }
        }
        true
    }
}

// endregion Schema

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[cfg(feature = "rusqlite")]
    fn test_sql<S: SQLStatement>(stmt: &mut S) -> Result<()> {
        for if_exists in [true, false] {
            for transaction in [true, false] {
                let sql: String = stmt.build(transaction, if_exists)?;

                assert_eq!(sql.len(), stmt.len(transaction, if_exists)?);

                let conn: Connection = Connection::open_in_memory()?;
                let ret = conn.execute_batch(&sql);
                if ret.is_err() {
                    println!("Error SQL: '{}'", sql)
                }
                ret?
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "rusqlite"))]
    fn test_sql<S: SQLStatement>(_stmt: &mut S) -> Result<()> {
        // todo ???
        Ok(())
    }

    fn test_sql_part<P: SQLPart>(part: &P) -> Result<()> {
        let mut str: String = String::with_capacity(part.part_len()?);

        part.part_str(&mut str)?;
        assert_eq!(str.len(), part.part_len()?);

        Ok(())
    }

    #[test]
    fn test_sqlite_type() -> Result<()> {
        let mut str: String;

        str = String::new();
        SQLiteType::Blob.part_str(&mut str)?;
        assert_eq!(str, "BLOB");
        assert_eq!(str.len(), SQLiteType::Blob.part_len()?);

        str = String::new();
        SQLiteType::Numeric.part_str(&mut str)?;
        assert_eq!(str, "NUMERIC");
        assert_eq!(str.len(), SQLiteType::Numeric.part_len()?);

        str = String::new();
        SQLiteType::Integer.part_str(&mut str)?;
        assert_eq!(str, "INTEGER");
        assert_eq!(str.len(), SQLiteType::Integer.part_len()?);

        str = String::new();
        SQLiteType::Real.part_str(&mut str)?;
        assert_eq!(str, "REAL");
        assert_eq!(str.len(), SQLiteType::Real.part_len()?);

        str = String::new();
        SQLiteType::Text.part_str(&mut str)?;
        assert_eq!(str, "TEXT");
        assert_eq!(str.len(), SQLiteType::Text.part_len()?);

        Ok(())
    }

    #[test]
    fn test_order() -> Result<()> {
        let mut str: String;

        str = String::new();
        Order::Ascending.part_str(&mut str)?;
        assert_eq!(str, "ASC");
        assert_eq!(str.len(), Order::Ascending.part_len()?);

        str = String::new();
        Order::Descending.part_str(&mut str)?;
        assert_eq!(str, "DESC");
        assert_eq!(str.len(), Order::Descending.part_len()?);

        Ok(())
    }

    #[test]
    fn test_on_conflict() -> Result<()> {
        let mut str: String;

        str = String::new();
        OnConflict::Rollback.part_str(&mut str)?;
        assert_eq!(str, "ON CONFLICT ROLLBACK");
        assert_eq!(str.len(), OnConflict::Rollback.part_len()?);

        str = String::new();
        OnConflict::Abort.part_str(&mut str)?;
        assert_eq!(str, "ON CONFLICT ABORT");
        assert_eq!(str.len(), OnConflict::Abort.part_len()?);

        str = String::new();
        OnConflict::Fail.part_str(&mut str)?;
        assert_eq!(str, "ON CONFLICT FAIL");
        assert_eq!(str.len(), OnConflict::Fail.part_len()?);

        str = String::new();
        OnConflict::Ignore.part_str(&mut str)?;
        assert_eq!(str, "ON CONFLICT IGNORE");
        assert_eq!(str.len(), OnConflict::Ignore.part_len()?);

        str = String::new();
        OnConflict::Replace.part_str(&mut str)?;
        assert_eq!(str, "ON CONFLICT REPLACE");
        assert_eq!(str.len(), OnConflict::Replace.part_len()?);

        Ok(())
    }

    #[test]
    fn test_fk_on_action() -> Result<()> {
        let mut str: String;

        str = String::new();
        FKOnAction::SetNull.part_str(&mut str)?;
        assert_eq!(str, "SET NULL");
        assert_eq!(str.len(), FKOnAction::SetNull.part_len()?);

        str = String::new();
        FKOnAction::SetDefault.part_str(&mut str)?;
        assert_eq!(str, "SET DEFAULT");
        assert_eq!(str.len(), FKOnAction::SetDefault.part_len()?);

        str = String::new();
        FKOnAction::Cascade.part_str(&mut str)?;
        assert_eq!(str, "CASCADE");
        assert_eq!(str.len(), FKOnAction::Cascade.part_len()?);

        str = String::new();
        FKOnAction::Restrict.part_str(&mut str)?;
        assert_eq!(str, "RESTRICT");
        assert_eq!(str.len(), FKOnAction::Restrict.part_len()?);

        str = String::new();
        FKOnAction::NoAction.part_str(&mut str)?;
        assert_eq!(str, "NO ACTION");
        assert_eq!(str.len(), FKOnAction::NoAction.part_len()?);

        Ok(())
    }

    #[test]
    fn test_generated_as() -> Result<()> {
        let mut str: String;

        str = String::new();
        GeneratedAs::Virtual.part_str(&mut str)?;
        assert_eq!(str, "VIRTUAL");
        assert_eq!(str.len(), GeneratedAs::Virtual.part_len()?);

        str = String::new();
        GeneratedAs::Stored.part_str(&mut str)?;
        assert_eq!(str, "STORED");
        assert_eq!(str.len(), GeneratedAs::Stored.part_len()?);

        Ok(())
    }

    #[test]
    fn test_not_null() -> Result<()> {
        let mut str: String;

        str = String::new();
        NotNull::new(OnConflict::Rollback).part_str(&mut str)?;
        assert_eq!(str, "NOT NULL ON CONFLICT ROLLBACK");
        assert_eq!(str.len(), NotNull::new(OnConflict::Rollback).part_len()?);

        str = String::new();
        NotNull::new(OnConflict::Abort).part_str(&mut str)?;
        assert_eq!(str, "NOT NULL ON CONFLICT ABORT");
        assert_eq!(str.len(), NotNull::new(OnConflict::Abort).part_len()?);

        str = String::new();
        NotNull::new(OnConflict::Fail).part_str(&mut str)?;
        assert_eq!(str, "NOT NULL ON CONFLICT FAIL");
        assert_eq!(str.len(), NotNull::new(OnConflict::Fail).part_len()?);

        str = String::new();
        NotNull::new(OnConflict::Ignore).part_str(&mut str)?;
        assert_eq!(str, "NOT NULL ON CONFLICT IGNORE");
        assert_eq!(str.len(), NotNull::new(OnConflict::Ignore).part_len()?);

        str = String::new();
        NotNull::new(OnConflict::Replace).part_str(&mut str)?;
        assert_eq!(str, "NOT NULL ON CONFLICT REPLACE");
        assert_eq!(str.len(), NotNull::new(OnConflict::Replace).part_len()?);

        Ok(())
    }

    #[test]
    fn test_unique() -> Result<()> {
        let mut str: String;

        str = String::new();
        Unique::new(OnConflict::Rollback).part_str(&mut str)?;
        assert_eq!(str, "UNIQUE ON CONFLICT ROLLBACK");
        assert_eq!(str.len(), Unique::new(OnConflict::Rollback).part_len()?);

        str = String::new();
        Unique::new(OnConflict::Abort).part_str(&mut str)?;
        assert_eq!(str, "UNIQUE ON CONFLICT ABORT");
        assert_eq!(str.len(), Unique::new(OnConflict::Abort).part_len()?);

        str = String::new();
        Unique::new(OnConflict::Fail).part_str(&mut str)?;
        assert_eq!(str, "UNIQUE ON CONFLICT FAIL");
        assert_eq!(str.len(), Unique::new(OnConflict::Fail).part_len()?);

        str = String::new();
        Unique::new(OnConflict::Ignore).part_str(&mut str)?;
        assert_eq!(str, "UNIQUE ON CONFLICT IGNORE");
        assert_eq!(str.len(), Unique::new(OnConflict::Ignore).part_len()?);

        str = String::new();
        Unique::new(OnConflict::Replace).part_str(&mut str)?;
        assert_eq!(str, "UNIQUE ON CONFLICT REPLACE");
        assert_eq!(str.len(), Unique::new(OnConflict::Replace).part_len()?);

        Ok(())
    }

    #[test]
    fn test_primary_key() -> Result<()> {
        for so in [Order::Ascending, Order::Descending] {
            for conf in [OnConflict::Rollback, OnConflict::Abort, OnConflict::Fail, OnConflict::Ignore, OnConflict::Replace] {
                for auto_inc in [true, false] {
                    test_sql_part(&PrimaryKey::new(so, conf, auto_inc))?;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_foreign_key() -> Result<()> {
        for defer in [true, false] {
            for on_del in [None, Some(FKOnAction::SetNull), Some(FKOnAction::SetDefault), Some(FKOnAction::Cascade), Some(FKOnAction::Restrict), Some(FKOnAction::NoAction)] {
                for on_upd in [None, Some(FKOnAction::SetNull), Some(FKOnAction::SetDefault), Some(FKOnAction::Cascade), Some(FKOnAction::Restrict), Some(FKOnAction::NoAction)] {
                    // todo: test string params
                    assert_eq!(ForeignKey::new("".to_string(), "test".to_string(), on_del, on_upd, defer).part_len(), Err(Error::EmptyForeignTableName));
                    assert_eq!(ForeignKey::new("test".to_string(), "".to_string(), on_del, on_upd, defer).part_len(), Err(Error::EmptyForeignColumnName));

                    test_sql_part(&ForeignKey::new("test".to_string(), "test".to_string(), on_del, on_upd, defer))?;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_generated() -> Result<()> {
        for gen_as in [None, Some(GeneratedAs::Virtual), Some(GeneratedAs::Stored)] {
            // todo: test string params
            assert_eq!(Generated::new("".to_string(), gen_as).part_len(), Err(Error::EmptyGeneratorExpr));

            test_sql_part(&Generated::new("test".to_string(), gen_as))?;
        }

        Ok(())
    }

    #[test]
    fn test_column() -> Result<()> {
        for typ in [SQLiteType::Blob, SQLiteType::Numeric, SQLiteType::Integer, SQLiteType::Real, SQLiteType::Text] {
            for pk in [None, Some(PrimaryKey::default())] {
                for uniq in [None, Some(Unique::default())] {
                    for fk in [None, Some(ForeignKey::new_default("test".to_string(), "test".to_string()))] {
                        for nn in [None, Some(NotNull::default())] {
                            for gened in [None, Some(Generated::new_default("expr".to_string()))] {
                                assert_eq!(Column::new(typ, "".to_string(), pk.clone(), uniq, fk.clone(), nn, gened.clone()).part_len(), Err(Error::EmptyColumnName));

                                let col: Column = Column::new(typ, "test".to_string(), pk.clone(), uniq, fk.clone(), nn, gened);

                                if col.pk.is_some() && col.fk.is_some() {
                                    assert_eq!(col.part_len(), Err(Error::PrimaryKeyAndForeignKey));
                                } else if col.pk.is_some() && col.unique.is_some() {
                                    assert_eq!(col.part_len(), Err(Error::PrimaryKeyAndUnique));
                                } else {
                                    test_sql_part(&col)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_view() -> Result<()> {
        'poss: for mut possible in View::possibilities(false).into_iter().map(|boxed| *boxed) {
            
            for col in &possible.columns {
                if col.name.is_empty() {
                    assert_eq!(col.part_len(), Err(Error::EmptyColumnName));
                    continue 'poss;
                }
            }
            
            if possible.name.is_empty() {
                assert_eq!(possible.part_len(), Err(Error::EmptyViewName));
                continue;
            }
            if possible.select.is_empty() {
                assert_eq!(possible.part_len(), Err(Error::EmptyViewSelect));
                continue;
            }
            
            test_sql_part(&possible)?;
            test_sql(&mut possible)?
        }
        Ok(())
    }

    #[test]
    fn test_table() -> Result<()> {
        'poss: for mut possible in Table::possibilities(false).into_iter().map(|boxed| *boxed) {
            let mut has_pk: bool = false;

            for col in &possible.columns {
                if col.pk.is_some() && col.unique.is_some() {
                    assert_eq!(col.part_len(), Err(Error::PrimaryKeyAndUnique));
                    continue 'poss;
                }
                if col.pk.is_some() && col.fk.is_some() {
                    assert_eq!(col.part_len(), Err(Error::PrimaryKeyAndForeignKey));
                    continue 'poss;
                }
                if col.pk.is_some() {
                    has_pk = true;
                }
            }
            if !possible.without_rowid && has_pk {
                assert_eq!(possible.part_len(), Err(Error::WithoutRowidNoPrimaryKey));
                continue;
            }

            if possible.name.is_empty() {
                assert_eq!(possible.part_len(), Err(Error::EmptyTableName));
                continue;
            }

            if possible.columns.is_empty() {
                assert_eq!(possible.part_len(), Err(Error::NoColumns));
                continue;
            }

            test_sql_part(&possible)?;
            test_sql(&mut possible)?; // FUCK
        }
        Ok(())
    }

    #[test]
    fn test_schema() -> Result<()> {
        {
            let mut schema: Schema = Schema::new();
            assert_eq!(schema.len(false, false), Err(Error::EmptySchema));
        }
        for num_tbl in 1..3 {
            let mut schema: Schema = Schema::new();
            for tbl_idx in 0..num_tbl {
                let mut tbl = Table::new_default(format!("table{}", tbl_idx));
                tbl = tbl.add_column(Column::new_default("testcol".to_string()));
                schema = schema.add_table(tbl);
            }
            test_sql(&mut schema)?;
        }

        Ok(())
    }

    #[cfg(feature = "xml-config")]
    mod xml_tests {
        use super::*;

        #[allow(dead_code)]
        impl AnySql {

            fn serialized(&self) -> Result<&'static str> {
                Ok(match self {
                    AnySql::Schema(s) => {Box::leak(quick_xml::se::to_string(s)?.into_boxed_str())}
                    AnySql::Table(t) => {Box::leak(quick_xml::se::to_string(t)?.into_boxed_str())}
                    AnySql::View(v) => {Box::leak(quick_xml::se::to_string(v)?.into_boxed_str())}
                })
            }

            fn deserialize_schema(xml: &'static str) -> Result<Self> {
                Ok(Self::Schema(from_str(xml)?))
            }

            fn deserialize_table(xml: &'static str) -> Result<Self> {
                Ok(Self::Table(from_str(xml)?))
            }

            fn deserialize_view(xml: &'static str) -> Result<Self> {
                Ok(Self::View(from_str(xml)?))
            }

            fn serialize_deserialize(self) -> Result<&'static str> {
                let xml = self.serialized()?;

                let deserialized: Self = match self {
                    Self::Schema(_) => {Self::deserialize_schema(xml)?},
                    Self::Table(_) => {Self::deserialize_table(xml)?},
                    Self::View(_) => {Self::deserialize_view(xml)?},
                };

                assert_eq!(deserialized, self);
                Ok(xml)
            }

            fn deserialize_serialize_inner(self, original_xml: & 'static str) -> Result<Self> {
                let new_xml: &'static str = self.serialized()?;
                assert_eq!(new_xml, original_xml);
                Ok(self)
            }

            fn deserialize_serialize_schema(xml: &'static str) -> Result<Self> {
                Self::deserialize_serialize_inner(Self::deserialize_schema(xml)?, xml)
            }

            fn deserialize_serialize_table(xml: &'static str) -> Result<Self> {
                Self::deserialize_serialize_inner(Self::deserialize_table(xml)?, xml)
            }

            fn deserialize_serialize_view(xml: &'static str) -> Result<Self> {
                Self::deserialize_serialize_inner(Self::deserialize_view(xml)?, xml)
            }
        }

        #[test]
        fn test_serialize_deserialize() -> Result<()> {
            let tbl = Table::new_default("TestName".to_string()).add_column(Column::new_default("TestCol".to_string()));
            let tbl2 = tbl.clone().set_name("TestName2".to_string());
            let view = View::new_default("TestView".to_string(), "Select".to_string()).add_column(ViewColumn::new("TestCol".to_string()));
            let schema = Schema::new().add_table(tbl.clone()).add_table(tbl2.clone()).add_view(view.clone());
            schema.to_any().serialize_deserialize()?;
            tbl.to_any().serialize_deserialize()?;
            tbl2.to_any().serialize_deserialize()?;
            view.to_any().serialize_deserialize()?;
            Ok(())
        }

        #[test]
        fn test_deserialize_serialize() -> Result<()> {
            let table_xml: &'static str = r#"<table name="test" without_rowid="false" strict="false"><column type="integer" name="test"/></table>"#;
            let view_xml: &'static str = r#"<view name="test" temp="false" select="SelectStatement"><column name="test"/></view>"#;
            let schema_xml: &'static str = Box::leak(("<schema>".to_string() + table_xml + view_xml + "</schema>").into_boxed_str());
            AnySql::deserialize_serialize_table(table_xml)?;
            AnySql::deserialize_serialize_view(view_xml)?;
            AnySql::deserialize_serialize_schema(schema_xml)?;
            Ok(())
        }

        #[test]
        fn some_test() -> Result<()> {
            let raw: &str = r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>
<schema xmlns="https://crates.io/crates/sqlayout">

  <!-- Card data -->
  <table name="updates" strict="true">
    <column name="ID" type="integer">
      <pk/>
      <not_null/>
    </column>
    <column name="timestamp" type="integer">
      <not_null/>
    </column>
    <column name="guid" type="text">
      <not_null/>
      <unique/>
    </column>
  </table>

  <table name="migrations" strict="true">
    <column name="ID" type="integer">
      <pk/>
      <not_null/>
    </column>
    <column name="timestamp" type="integer">
      <not_null/>
    </column>
    <column name="GUID" type="text">
      <not_null/>
      <unique/>
    </column>
  </table>

  <table name="card_data" strict="true">
    <column name="ID" type="integer">
      <pk/>
      <not_null/>
    </column>
  </table>

  <!-- Collection Data -->
  <table name="card_location" strict="true">
    <column name="ID" type="integer">
      <pk/>
      <not_null/>
    </column>
    <column name="name" type="text">
      <not_null/>
    </column>
    <column name="description" type="text"/>
  </table>

  <table name="card_collection" strict="true">
    <column name="ID" type="integer">
      <pk/>
      <not_null/>
    </column>
    <column name="card_ID" type="integer">
      <fk foreign_table="card_data" foreign_column="ID"/>
      <not_null/>
    </column>
    <column name="count" type="integer">
      <not_null/>
    </column>
    <column name="finish" type="integer">
      <!-- enum -->
      <not_null/>
    </column>
    <column name="condition" type="integer">
      <!-- enum -->
    </column>
    <column name="location" type="integer">
      <fk foreign_table="card_location" foreign_column="ID"/>
      <not_null/>
    </column>
    <column name="location_page" type="integer"/>
  </table>
</schema>
"#;
            let _: Schema = quick_xml::de::from_str(raw)?;
            Ok(())
        }

        #[test]
        fn xmlns_test() -> Result<()> {
            let tbl: Table = Table::new_default("test".to_string()).add_column(Column::new_default("testcol".to_string()));
            let sch: Schema = Schema::new().add_table(tbl.clone());

            let xml: String = quick_xml::se::to_string(&sch)?;
            println!("Schema XML:\n{}", xml);
            let xml: String = quick_xml::se::to_string(&tbl)?;
            println!("Table XML:\n{}", xml);
            Ok(())
        }
    }

    #[cfg(feature = "rusqlite")]
    mod rusqlite {
        // todo
    }
}
