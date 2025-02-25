use derive_more::From;
use lib_auth::pwd;
use serde::Serialize;
use serde_with::serde_as;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
	EntityNotFound {
		entity: &'static str,
		id: i64,
	},
	ListLimitOverMax {
		max: i64,
		actual: i64,
	},

	CountFail,

	// -- DB
	UserAlreadyExists {
		username: String,
	},
	UniqueViolation {
		table: String,
		constraint: String,
	},

	// -- CRUD
	CrudDocumentError,
	CreateError,
	ReadError,
	UpdateError,
	DeleteError,
	FilterError,
	ObIdError,
	QueryError,
	SessionError,
	CrudSessionError(String),

	// -- ModelManager
	CantCreateModelManagerProvider(String),
	NoSession,

	// -- Modules
	#[from]
	Pwd(pwd::Error),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(
			fmt,
			"{self:?}"
		)
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
