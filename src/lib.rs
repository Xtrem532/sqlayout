//! A Library for generating SQLite-specific SQL to Initialize Databases (as in `CREATE TABLE...`).
//! SQLite Interface agnostic, e.g. can be used with [rusqlite](https://github.com/rusqlite/rusqlite), [sqlite](https://github.com/stainless-steel/sqlite) or any other SQLite Interface.
//!
//! # xml-config
//!
//! todo

#![warn(missing_docs)]
mod error;

#[cfg(feature = "xml-config")]
use serde::{Serialize, Deserialize};

pub use error::{Error, Result};

// this cannot be in the test mod b/c it is needed for the test trait impls (SQLPart::possibilities)
#[cfg(test)]
fn option_iter<T: Clone>(input: Vec<Box<T>>) -> Vec<Option<T>> {
    let mut ret: Vec<Option<T>> = input.iter().map(|boxed| Some(*boxed.clone())).collect::<Vec<Option<T>>>();
    ret.push(None);
    ret
}

// region Traits

trait SQLPart {
    fn part_len(&self) -> Result<usize>;

    fn part_str(&self, sql: &mut String) -> Result<()>;

    // todo: for no-std
    // fn part_arr(&self, sql: &mut [u8]) -> Result<()>;

    #[cfg(test)]
    fn possibilities(illegal_variants: bool) -> Vec<Box<Self>>;
}

/// Any struct Implementing this trait can be converted into a SQL statement [String].
/// Optionally, the statement can be wrapped in a SQL Transaction and/or guarded against already existing Tables with a `...IF NOT EXISTS...` guard.
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
    /// * `if_exists`: Weather the `CREATE TABLE...` Statement should include a `...IF NOT EXISTS...` guard
    fn build(&mut self, transaction: bool, if_exists: bool) -> Result<String>;

    // todo: for no-std
    // fn build_arr(&self, arr: &mut [u8], transaction: bool) -> Result<()>;
}

// endregion

// region SQLiteType

/// Encodes all Column-Datatypes available in SQLite, see [here](https://www.sqlite.org/datatype3.html#type_affinity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
#[allow(missing_docs)]
pub enum SQLiteType {
    // ref. https://www.sqlite.org/datatype3.html#type_affinity
    Blob,
    Numeric,
    Integer,
    Real,
    Text
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
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
#[allow(missing_docs)]
pub enum Order {
    Ascending,
    Descending
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
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
#[allow(missing_docs)]
pub enum OnConflict {
    Rollback,
    Abort,
    Fail,
    Ignore,
    Replace
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

    pub fn set_autoincrement(mut self, autoinc: bool) -> Self {
        self.autoincrement = autoinc;
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
                for autoinc in [true, false] {
                    ret.push(Box::new(Self::new(*so, *conf, autoinc)))
                }
            }
        }
        ret
    }
}

// endregion

// region Not Null

/// Marks a [Column] as `NOT NULL`, e.g. the Column cannot contain `NULL` values and trying to insert `NULL` values is a Error.
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

/// Marks a [Column] as "Unique", e.g. the Column cannot contain the same value twice and trying to insert a value for the second time is a Error.
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

/// Defines a Foreign Key for a [Column]. It is a Error for the `foreign_table` and `foreign_column` [String]s to be Empty ([Error::EmptyForeignTableName], [Error::EmptyForeignColumnName]).
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
    #[cfg_attr(feature = "xml-config", serde(rename = "@deferrable"))]
    deferrable: bool,
}

impl ForeignKey {
    pub fn check(&self) -> Result<()> {
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
        for tbl in [if illegal { "".to_string() } else { "test".to_string() } , "test".to_string()] {
            for col in [if illegal { "".to_string() } else { "test".to_string() } , "test".to_string()] {
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

// region Column

/// This struct Represents a Column in a [Table]. It is a Error for the `name` to be Empty ([Error::EmptyColumnName]).
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
    // todo Generated Column
}

impl Column {
    fn check(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::EmptyColumnName)
        }

        if self.pk.is_some() && self.fk.is_some() {
            return Err(Error::PrimaryKeyAndForeignKey)
        }

        if self.pk.is_some() && self.unique.is_some() {
            return Err(Error::PrimaryKeyAndUnique)
        }

        Ok(())
    }

    pub fn new(typ: SQLiteType, name: String, pk: Option<PrimaryKey>, unique: Option<Unique>, fk: Option<ForeignKey>) -> Self {
        Self {
            typ,
            name,
            pk,
            unique,
            fk,
        }
    }

    pub fn new_default(name: String) -> Self {
        Self {
            typ: Default::default(),
            name,
            pk: Default::default(),
            unique: Default::default(),
            fk: Default::default(),
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
            for name in [if illegal { "".to_string() } else { "test".to_string() } , "test".to_string()] {
                for pk in option_iter(PrimaryKey::possibilities(false)) {
                    for unique in option_iter(Unique::possibilities(false)) {
                        for fk in option_iter(ForeignKey::possibilities(false)) {
                            if !illegal && pk.is_some() && (fk.is_some() || unique.is_some()) {
                                continue
                            } 
                            ret.push(Box::new(Self::new(*typ.clone(), name.clone(), pk.clone(), unique, fk)));
                        }
                    }
                }
            }
        }
        ret
    }
}

// endregion

// region Table

/// Represents an entire Table, which may be Part of a wider [Schema] or used standalone.
/// Can be converted into an SQL Statement via the [SQLStatement] Methods.
/// It is a Error for the `name` to be empty ([Error::EmptyTableName]) or the Table itself to be empty ([Error::NoColumns]).
#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize))]
pub struct Table {
    #[cfg_attr(feature = "xml-config", serde(rename = "@name"))]
    name: String,
    #[cfg_attr(feature = "xml-config", serde(rename = "column"))]
    columns: Vec<Column>,
    #[cfg_attr(feature = "xml-config", serde(rename = "@without_rowid"))]
    without_rowid: bool,
    #[cfg_attr(feature = "xml-config", serde(rename = "@strict"))]
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
            return Err(Error::NoColumns)
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
            without_rowid: false,
            strict: false,
            if_exists: false
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
            + (self.without_rowid && self.strict) as usize * 1 // ','
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
        if self.without_rowid && self.strict  {
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
        for name in [if illegal { "".to_string() } else { "test".to_string() } , "test".to_string()] {
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

impl SQLStatement for Table {
    fn len(&mut self, transaction: bool, if_exists: bool) -> Result<usize> {
        self.if_exists = if_exists;
        Ok(transaction as usize * 7 + self.part_len()? + 1 + transaction as usize * 5)
    }

    fn build(&mut self, transaction: bool, if_exist: bool) -> Result<String> {
        let mut str = String::with_capacity(self.len(transaction, if_exist)?);
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
}

impl PartialEq<Table> for Table {
    fn eq(&self, other: &Table) -> bool {
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
/// It is a Error for the Schema to be empty ([Error::SchemaWithoutTables]).
#[derive(Debug, Clone, Default, Eq)]
#[cfg_attr(feature = "xml-config", derive(Serialize, Deserialize), serde(rename = "schema"))]
pub struct Schema {
    #[cfg_attr(feature = "xml-config", serde(rename = "table"))]
    tables: Vec<Table>,
    #[cfg(feature = "xml-config")]
    #[cfg_attr(feature = "xml-config", serde(rename = "@xmlns"))]
    xmlns: &'static str,
}

impl Schema {
    fn check(&self) -> Result<()> {
        if self.tables.is_empty() {
            return Err(Error::SchemaWithoutTables);
        }
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            #[cfg(feature = "xml-config")]
            xmlns: "https://crates.io/crates/sqlayout"
        }
    }

    pub fn add_table(mut self, new_table: Table) -> Self {
        self.tables.push(new_table);
        self
    }
}

impl SQLStatement for Schema {
    fn len(&mut self, transaction: bool, if_exists: bool) -> Result<usize> {
        self.check()?;
        let mut tbls_len: usize = 0;
        for tbl in &mut self.tables {
            tbl.if_exists = if_exists;
            tbls_len += tbl.part_len()?;
        }
        Ok(transaction as usize * 7 + tbls_len + self.tables.len() + transaction as usize * 5)
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
    use rusqlite::Connection;

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
                for autoinc in [true, false] {
                    test_sql_part(&PrimaryKey::new(so, conf, autoinc))?;
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
    fn test_column() -> Result<()> {
        for typ in [SQLiteType::Blob, SQLiteType::Numeric, SQLiteType::Integer, SQLiteType::Real, SQLiteType::Text] {
            for pk in [None, Some(PrimaryKey::default())] {
                for uniq in [None, Some(Unique::default())] {
                    for fk in [None, Some(ForeignKey::new_default("test".to_string(), "test".to_string()))] {
                        assert_eq!(Column::new(typ, "".to_string(),Clone::clone(&pk), uniq, Clone::clone(&fk)).part_len(), Err(Error::EmptyColumnName));

                        let col: Column = Column::new(typ, "test".to_string(), Clone::clone(&pk), uniq, Clone::clone(&fk));

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
            assert_eq!(schema.len(false, false), Err(Error::SchemaWithoutTables));
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

        #[test]
        fn test_serialize() -> Result<()> {
            let tbl = Table::new_default("TestName".to_string()).add_column(Column::new_default("TestCol".to_string()));
            let tbl2  = tbl.clone().set_name("TestName2".to_string());
            let schema = Schema::new().add_table(tbl).add_table(tbl2);
            // todo: this is bullshit
            let serialized: &'static str = Box::leak(quick_xml::se::to_string(&schema)?.into_boxed_str());
            println!("Serialized XML: \n{}", serialized);
            let deserialized: Schema = quick_xml::de::from_str(serialized)?;
            assert_eq!(schema, deserialized);
            Ok(())
        }
    }
}
