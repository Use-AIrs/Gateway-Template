// region:    --- Modules

pub(crate) mod crud_fns;

// -- Flatten hierarchy for user code.
pub use crud_fns::*;

// endregion: --- Modules

/// The DbBmc trait must be implemented for the Bmc struct of an entity.
/// It specifies meta information such as the table name,
/// whether the table has timestamp columns (cid, ctime, mid, mtime), and more as the
/// code evolves.
///
/// Note: This trait should not be confused with the BaseCrudBmc trait, which provides
///       common default CRUD BMC functions for a given Bmc/Entity.
pub trait DbBmc {
	const TABLE: &'static str;

	fn table_ref() -> &'static str {
		Self::TABLE
	}

	/// Specifies if the entity table managed by this BMC
	/// has an `owner_id` column that needs to be set on create (by default ctx.user_id).
	///
	/// default: false
	fn has_owner_id() -> bool {
		false
	}
}
