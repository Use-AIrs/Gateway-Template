use crate::config::core_config;
use crate::ctx::Ctx;
use crate::model::user::{QUser, QUserForCreate, UserBmc};
use crate::model::{Error, ModelManager, Result};
use mongodb::Client;

pub async fn init_dev() -> Result<()> {
	let client = Client::with_uri_str(&core_config().DB_URL)
		.await
		.map_err(|_| Error::NoSession)?;
	let d = QUserForCreate {
		username: "test".to_string(),
		pwd_clear: "admin".to_string(),
	};
	let _ = UserBmc::create(
		&Ctx::root_ctx(),
		&ModelManager::new().await?,
		d,
	)
	.await?;
	Ok(())
}
