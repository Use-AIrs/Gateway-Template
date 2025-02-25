use crate::ctx::Ctx;
use crate::model::base::{create, delete, get, list, update, DbBmc};
use crate::model::ModelManager;
use crate::model::{Error, Result};
use lib_auth::pwd;
use lib_auth::pwd::ContentToHash;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use time::Date;
use uuid::Uuid;

/// --- Q-User Types

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QUser {
	#[serde(rename = "_id")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<String>,
	pub active: Option<bool>,
	pub login: Option<QUserLogin>,
	pub metadata: Option<QUserMeta>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QUserLogin {
	pub username: Option<String>,
	pub pw: Option<String>,
	pub pwd_salt: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QUserMeta {
	pub name: Option<String>,
	pub surname: Option<String>,
	pub birth: Option<Date>,
	pub mail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QUserForCreate {
	pub username: String,
	pub pwd_clear: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QUserForLogin {
	pub id: String,
	pub username: String,
	pub pwd: Option<String>,
	pub pwd_salt: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QUserForAuth {
	pub id: String,
	pub username: String,
	pub pwd_salt: Uuid,
}
impl QUser {
	pub fn filter_id(id_string: &String) -> Result<Self> {
		Ok(
			QUser {
				id: Some(id_string.to_string()),
				active: None,
				login: None,
				metadata: None,
			},
		)
	}

	pub fn filter_username(username: String) -> Result<Self> {
		let login = QUserLogin {
			username: Some(username),
			pw: None,
			pwd_salt: None,
		};
		Ok(
			QUser {
				id: None,
				active: None,
				login: Some(login),
				metadata: None,
			},
		)
	}

	pub async fn create_user(userc: QUserForCreate) -> Result<Self> {
		let salt = Uuid::new_v4();
		let pwd = pwd::hash_pwd(
			ContentToHash {
				content: userc.pwd_clear,
				salt,
			},
		)
		.await?;

		let login = QUserLogin {
			username: Some(userc.username),
			pw: Some(pwd),
			pwd_salt: Some(salt),
		};

		Ok(
			QUser {
				id: None,
				active: Some(true),
				login: Some(login),
				metadata: None,
			},
		)
	}

	pub fn update_pwd(
		mut self,
		new_pwd: String,
	) -> Result<Self> {
		if let Some(ref mut login) = self.login {
			login.pw = Some(new_pwd);
			Ok(self)
		} else {
			Err(Error::UpdateError)
		}
	}
}

pub struct UserBmc;

impl DbBmc for UserBmc {
	const TABLE: &'static str = "Users";
}

impl UserBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		input: QUserForCreate,
	) -> Result<String> {
		let new_user = QUser::create_user(input).await?;
		let id = create::<UserBmc, QUser>(
			ctx, mm, new_user,
		)
		.await?;
		Ok(id)
	}

	pub async fn update_pwd(
		ctx: &Ctx,
		mm: &ModelManager,
		id: &String,
		pwd: &str,
	) -> Result<()> {
		let user = Self::get(
			ctx, mm, id,
		)
		.await?;
		if let Some(login) = &user.login {
			let salt = login.pwd_salt.ok_or(Error::UpdateError)?;
			let hashed_pwd = pwd::hash_pwd(
				ContentToHash {
					content: pwd.to_string(),
					salt,
				},
			)
			.await?;
			let updated_user = user.update_pwd(hashed_pwd)?;
			update::<UserBmc, QUser>(
				ctx,
				mm,
				id,
				updated_user,
			)
			.await?;
			Ok(())
		} else {
			Err(Error::UpdateError)
		}
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: &String,
	) -> Result<QUser> {
		let filter = QUser::filter_id(id)?;
		get::<UserBmc, QUser>(
			ctx, mm, filter,
		)
		.await
	}

	pub async fn get_user_by_name(
		ctx: &Ctx,
		mm: &ModelManager,
		username: &String,
	) -> Result<QUser> {
		let filter = QUser::filter_username(username.clone())?;
		println!(
			"{:?}",
			filter
		);
		get::<UserBmc, QUser>(
			ctx, mm, filter,
		)
		.await
	}

	pub async fn get_user_for_login(
		ctx: &Ctx,
		mm: &ModelManager,
		username: &String,
	) -> Result<QUserForLogin> {
		let filter = QUser::filter_username(username.clone())?;
		let user = get::<UserBmc, QUser>(
			ctx, mm, filter,
		)
		.await?;

		let id = user.id.ok_or(Error::ReadError)?.to_string();
		let login = user.login.ok_or(Error::ReadError)?;
		let username = login.username.ok_or(Error::ReadError)?;
		let pwd = login.pw.ok_or(Error::ReadError)?;
		let pwd_salt = login.pwd_salt.ok_or(Error::ReadError)?;

		let out = QUserForLogin {
			id,
			username,
			pwd: Some(pwd),
			pwd_salt,
		};
		Ok(out)
	}

	pub async fn get_user_for_auth(
		ctx: &Ctx,
		mm: &ModelManager,
		username: &String,
	) -> Result<QUserForAuth> {
		let filter = QUser::filter_username(username.clone())?;
		let user = get::<UserBmc, QUser>(
			ctx, mm, filter,
		)
		.await?;

		let id = user.id.ok_or(Error::ReadError)?.to_string();
		let login = user.login.ok_or(Error::ReadError)?;
		let username = login.username.ok_or(Error::ReadError)?;
		let pwd_salt = login.pwd_salt.ok_or(Error::ReadError)?;

		let out = QUserForAuth {
			id,
			username,
			pwd_salt,
		};
		Ok(out)
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filter: Option<QUser>,
	) -> Result<Vec<QUser>> {
		let filter_doc = filter.unwrap_or(
			QUser {
				id: None,
				active: None,
				login: None,
				metadata: None,
			},
		);
		list::<UserBmc, QUser>(
			ctx, mm, filter_doc,
		)
		.await
	}

	pub async fn delete(
		ctx: &Ctx,
		mm: &ModelManager,
		id: &String,
	) -> Result<()> {
		let filter = QUser::filter_id(id)?;
		delete::<UserBmc, QUser>(
			ctx, mm, filter,
		)
		.await
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ctx::Ctx;
	use crate::model::error::Error;
	use crate::model::ModelManager;
	use mongodb::bson::{doc, Document};
	use serde::{Deserialize, Serialize};
	use serde_with::skip_serializing_none;
	use tokio;

	#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
	#[skip_serializing_none]
	struct DummyData {
		pub id: Option<String>,
		pub name: String,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub age: Option<u32>,
	}

	struct DummyMC;

	impl DbBmc for DummyMC {
		fn has_owner_id() -> bool {
			true
		}
		const TABLE: &'static str = "dummy_collection_for_testing";
	}

	/// Hilfsfunktion, um die Collection zu leeren
	async fn clear_collection(
		mm: &ModelManager,
		tenant: &String,
	) -> Result<()> {
		let collection = mm
			.client
			.database(tenant)
			.collection::<Document>(DummyMC::TABLE);
		collection
			.delete_many(doc! {})
			.await
			.map_err(|_| Error::DeleteError)?;
		Ok(())
	}

	/// Beispiel: Vor und nach dem Test die Collection leeren
	#[tokio::test]
	async fn test_create_and_get() -> Result<()> {
		// Erzeuge Kontext und ModelManager
		let ctx = Ctx::root_ctx();
		let tenant = ctx.tenant_id();
		let mm = ModelManager::new().await?;
		// Sicherstellen, dass die Collection leer ist
		clear_collection(
			&mm, &tenant,
		)
		.await?;

		let data = DummyData {
			id: None,
			name: "Alice".to_string(),
			age: Some(30),
		};

		// Dokument erstellen
		let id = create::<DummyMC, _>(
			&ctx, &mm, data,
		)
		.await
		.expect("create failed");
		assert!(
			!id.is_empty(),
			"Die zurückgegebene ID darf nicht leer sein"
		);

		let filter = DummyData {
			id: None,
			name: "Alice".to_string(),
			age: None,
		};
		let fetched: DummyData = get::<DummyMC, _>(
			&ctx, &mm, filter,
		)
		.await
		.expect("get failed");
		assert_eq!(
			fetched.name,
			"Alice"
		);
		assert_eq!(
			fetched.age,
			Some(30)
		);

		// Cleanup: Collection leeren
		clear_collection(
			&mm, &tenant,
		)
		.await?;
		Ok(())
	}

	#[tokio::test]
	async fn test_update() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let tenant = ctx.tenant_id().clone();
		let mm = ModelManager::new().await?;
		clear_collection(
			&mm, &tenant,
		)
		.await?;

		let data = DummyData {
			id: None,
			name: "Bob".to_string(),
			age: Some(25),
		};

		let id = create::<DummyMC, _>(
			&ctx, &mm, data,
		)
		.await
		.expect("create failed");

		let update_data = DummyData {
			id: None,
			name: "Robert".to_string(),
			age: Some(26),
		};

		update::<DummyMC, _>(
			&ctx,
			&mm,
			&id,
			update_data,
		)
		.await
		.expect("update failed");

		let filter = DummyData {
			id: None,
			name: "Robert".to_string(),
			age: None,
		};

		let fetched: DummyData = get::<DummyMC, _>(
			&ctx, &mm, filter,
		)
		.await
		.expect("get after update failed");
		assert_eq!(
			fetched.name,
			"Robert"
		);
		assert_eq!(
			fetched.age,
			Some(26)
		);

		clear_collection(
			&mm, &tenant,
		)
		.await?;
		Ok(())
	}

	#[tokio::test]
	async fn test_delete() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let tenant = ctx.tenant_id();
		let mm = ModelManager::new().await?;
		clear_collection(
			&mm, &tenant,
		)
		.await?;

		let data = DummyData {
			id: None,
			name: "Charlie".to_string(),
			age: Some(40),
		};

		let _id = create::<DummyMC, _>(
			&ctx, &mm, data,
		)
		.await
		.expect("create failed");

		let filter = DummyData {
			id: None,
			name: "Charlie".to_string(),
			age: None,
		};

		delete::<DummyMC, _>(
			&ctx, &mm, filter,
		)
		.await
		.expect("delete failed");

		let filter = DummyData {
			id: None,
			name: "Charlie".to_string(),
			age: None,
		};
		let result: Result<DummyData> = get::<DummyMC, _>(
			&ctx, &mm, filter,
		)
		.await;
		assert!(
			result.is_err(),
			"Das Dokument sollte nicht mehr existieren"
		);

		clear_collection(
			&mm, &tenant,
		)
		.await?;
		Ok(())
	}

	#[tokio::test]
	async fn test_list() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let tenant = ctx.tenant_id().clone();
		let mm = ModelManager::new().await?;
		clear_collection(
			&mm, &tenant,
		)
		.await?;

		let data1 = DummyData {
			id: None,
			name: "Dave".to_string(),
			age: Some(20),
		};
		let data2 = DummyData {
			id: None,
			name: "Eve".to_string(),
			age: Some(22),
		};

		create::<DummyMC, _>(
			&ctx, &mm, data1,
		)
		.await
		.expect("create for data1 failed");
		create::<DummyMC, _>(
			&ctx, &mm, data2,
		)
		.await
		.expect("create for data2 failed");

		let list_result = list::<DummyMC, DummyData>(
			&ctx,
			&mm,
			Default::default(),
		)
		.await
		.expect("list failed");
		println!(
			"{:?}",
			list_result
		);
		Ok(())
	}
}
