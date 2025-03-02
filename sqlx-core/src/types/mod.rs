//! Conversions between Rust and SQL types.
//!
//! To see how each SQL type maps to a Rust type, see the corresponding `types` module for each
//! database:
//!
//!  * [PostgreSQL](../postgres/types/index.html)
//!  * [MySQL](../mysql/types/index.html)
//!  * [SQLite](../sqlite/types/index.html)
//!  * [MSSQL](../mssql/types/index.html)
//!
//! Any external types that have had [`Type`] implemented for, are re-exported in this module
//! for convenience as downstream users need to use a compatible version of the external crate
//! to take advantage of the implementation.
//!
//! # Nullable
//!
//! To represents nullable SQL types, `Option<T>` is supported where `T` implements `Type`.
//! An `Option<T>` represents a potentially `NULL` value from SQL.
//!

use crate::database::Database;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
mod json;

#[cfg(feature = "uuid")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
pub use uuid::Uuid;

#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub mod chrono {
    pub use chrono::{
        DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc,
    };
}

#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub mod time {
    pub use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};
}

#[cfg(feature = "bigdecimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigdecimal")))]
pub use bigdecimal::BigDecimal;

#[cfg(feature = "decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "decimal")))]
pub use rust_decimal::Decimal;

#[cfg(feature = "ipnetwork")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipnetwork")))]
pub mod ipnetwork {
    pub use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
}

#[cfg(feature = "json")]
pub use json::Json;

/// Indicates that a SQL type is supported for a database.
///
/// ## Compile-time verification
///
/// With compile-time verification, the use of type overrides is currently required to make
/// use of any user-defined types.
///
/// ```rust,ignore
/// struct MyUser { id: UserId, name: String }
///
/// // fetch all properties from user and override the type in Rust for `id`
/// let user = query_as!(MyUser, r#"SELECT users.*, id as "id: UserId" FROM users"#)
///     .fetch_one(&pool).await?;
/// ```
///
/// ## Derivable
///
/// This trait can be derived by SQLx to support Rust-only wrapper types, enumerations, and (for
/// postgres) structured records. Additionally, an implementation of [`Encode`] and [`Decode`] is
/// generated.
///
/// [`Encode`]: trait.Encode.html
/// [`Decode`]: trait.Decode.html
///
/// ### Transparent
///
/// Rust-only domain or wrappers around SQL types. The generated implementations directly delegate
/// to the implementation of the inner type.
///
/// ```rust,ignore
/// #[derive(sqlx::Type)]
/// #[sqlx(transparent)]
/// struct UserId(i64);
/// ```
///
/// ### Enumeration
///
/// Enumerations may be defined in Rust and can match SQL by
/// integer discriminant or variant name.
///
/// With `#[repr(_)]` the integer representation is used when converting from/to SQL and expects
/// that SQL type (e.g., `INT`). Without, the names of the variants are used instead and
/// expects a textual SQL type (e.g., `VARCHAR`, `TEXT`).
///
/// ```rust,ignore
/// #[derive(sqlx::Type)]
/// #[repr(i32)]
/// enum Color { Red = 1, Green = 2, Blue = 3 }
/// ```
///
/// ```rust,ignore
/// #[derive(sqlx::Type)]
/// #[sqlx(rename = "color")] // only for PostgreSQL to match a type definition
/// #[sqlx(rename_all = "lowercase")]
/// enum Color { Red, Green, Blue }
/// ```
///
/// ### Records
///
/// User-defined composite types are supported through deriving a `struct`.
///
/// This is only supported for PostgreSQL.
///
/// ```rust,ignore
/// #[derive(sqlx::Type)]
/// #[sqlx(rename = "interface_type")]
/// struct InterfaceType {
///     name: String,
///     supplier_id: i32,
///     price: f64
/// }
/// ```
///
pub trait Type<DB: Database> {
    /// Returns the canonical SQL type for this Rust type.
    ///
    /// When binding arguments, this is used to tell the database what is about to be sent; which,
    /// the database then uses to guide query plans. This can be overridden by `Encode::produces`.
    ///
    /// A map of SQL types to Rust types is populated with this and used
    /// to determine the type that is returned from the anonymous struct type from `query!`.
    fn type_info() -> DB::TypeInfo;

    /// Determines if this Rust type is compatible with the given SQL type.
    ///
    /// When decoding values from a row, this method is checked to determine if we should continue
    /// or raise a runtime type mismatch error.
    ///
    /// When binding arguments with `query!` or `query_as!`, this method is consulted to determine
    /// if the Rust type is acceptable.
    fn compatible(ty: &DB::TypeInfo) -> bool {
        *ty == Self::type_info()
    }
}

// for references, the underlying SQL type is identical
impl<T: ?Sized + Type<DB>, DB: Database> Type<DB> for &'_ T {
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <T as Type<DB>>::compatible(ty)
    }
}

// for optionals, the underlying SQL type is identical
impl<T: Type<DB>, DB: Database> Type<DB> for Option<T> {
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <T as Type<DB>>::compatible(ty)
    }
}
