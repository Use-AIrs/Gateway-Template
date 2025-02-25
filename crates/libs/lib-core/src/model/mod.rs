//! Model Layer
//!
//! Design:
//!
//! - The Model layer normalizes the application's data type
//!   structures and access.
//! - All application code data access must go through the Model layer.
//! - The `ModelManager` holds the internal states/resources
//!   needed by ModelControllers to access data.
//!   (e.g., db_pool, S3 client, redis client).
//! - Model Controllers (e.g., `ConvBmc`, `AgentBmc`) implement
//!   CRUD and other data access methods on a given "entity"
//!   (e.g., `Conv`, `Agent`).
//!   (`Bmc` is short for Backend Model Controller).
//! - In frameworks like Axum, Tauri, `ModelManager` are typically used as App State.
//! - ModelManager are designed to be passed as an argument
//!   to all Model Controllers functions.
//!

// region:    --- Modules

mod acs;
mod base;
mod error;
mod store;

pub mod example;
pub mod user;

pub use self::error::{Error, Result};
use mongodb::{Client, ClientSession};
use std::cell::RefCell;
use std::rc::Rc;

use crate::model::store::new_client;

// endregion: --- Modules

// region:    --- ModelManager

#[cfg_attr(feature = "with-rpc", derive(rpc_router::RpcResource))]
#[derive(Debug, Clone)]
pub struct ModelManager {
	client: Client,
	// session: Rc<RefCell<Option<ClientSession>>>, // Have to think about sessions, clone is needed
}

impl ModelManager {
	/// Constructor
	pub async fn new() -> Result<Self> {
		let client = new_client()
			.await
			.map_err(|e| Error::CantCreateModelManagerProvider(e.to_string()))?;

		Ok(
			ModelManager {
				client,
				// session: Rc::new(RefCell::new(None)),
			},
		)
	}

	pub async fn new_with_txn(&self) -> Result<&Self> {
		todo!()
	}
}

// endregion: --- ModelManager
