use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use futures::stream::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, from_document, to_document, Bson, Document};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

#[derive(Serialize)]
#[skip_serializing_none]
pub struct Update<S: Serialize> {
	#[serde(rename = "$set")]
	set: S,
}

impl<S: Serialize> Update<S> {
	pub fn doc(set: S) -> Document {
		let input = Self { set };
		to_document(&input).unwrap()
	}
}

#[derive(Serialize, Debug)]
#[skip_serializing_none]
pub struct Query<S: Serialize> {
	set: S,
}

impl<S: Serialize> Query<S> {
	pub fn doc(set: S) -> Self {
		Self { set }
	}
}

#[derive(Serialize)]
#[skip_serializing_none]
pub struct Filter<S: Serialize> {
	set: S,
}

impl<S: Serialize> Filter<S> {
	pub fn doc(set: S) -> Document {
		let input = Self { set };
		let doc = to_document(&input.set).unwrap();
		let filter = make_filter_doc(doc);
		let filter = convert_filter_ids(filter);
		filter
	}
}

pub fn convert_filter_ids(mut doc: Document) -> Document {
	for (key, value) in doc.clone().into_iter() {
		if key == "id" {
			if let Bson::String(s) = value {
				if let Ok(oid) = ObjectId::parse_str(&s) {
					doc.remove("id");
					doc.insert(
						"_id",
						Bson::ObjectId(oid),
					);
				}
			}
		}
	}
	doc
}

pub fn make_filter_doc(doc: Document) -> Document {
	fn flatten(
		doc: Document,
		prefix: Option<String>,
	) -> Document {
		let mut flat = Document::new();
		for (key, value) in doc.into_iter() {
			// Neuer Schlüssel mit Prefix (falls vorhanden)
			let new_key = if let Some(prefix_str) = prefix.as_ref() {
				format!(
					"{}.{}",
					prefix_str, key
				)
			} else {
				key
			};

			match value {
				Bson::Document(inner_doc) => {
					// Rekursiv flatten
					let inner_flat = flatten(
						inner_doc,
						Some(new_key),
					);
					// Nur wenn das innere Dokument nicht leer ist, übernehmen wir die Felder
					for (k, v) in inner_flat.into_iter() {
						flat.insert(
							k, v,
						);
					}
				},
				Bson::Null => {
					// Ignoriere Felder, die Null sind
				},
				other => {
					flat.insert(
						new_key, other,
					);
				},
			}
		}
		flat
	}
	flatten(
		doc, None,
	)
}

pub async fn create<MC, D>(
	ctx: &Ctx,
	mm: &ModelManager,
	data: D,
) -> Result<String>
where
	MC: DbBmc,
	D: Serialize + Send + Sync + Debug,
{
	let q = Query::doc(data);
	let collection = mm
		.client
		.database(ctx.tenant_id().as_str())
		.collection::<D>(MC::TABLE);
	let result = collection.insert_one(q.set).await;
	let res = result.map_err(|e| Error::CreateError)?;
	let out = res.inserted_id.as_object_id().to_owned();
	let st = out.unwrap().to_hex();
	Ok(st)
}

pub async fn update<MC, D>(
	ctx: &Ctx,
	mm: &ModelManager,
	id: &String,
	update: D,
) -> Result<()>
where
	MC: DbBmc,
	D: Serialize + Send + Sync,
{
	let object_id = ObjectId::parse_str(id).map_err(|_| Error::ObIdError)?;
	let filter = doc! { "_id": object_id };
	let update_doc = Update::doc(update);
	let doc = update_doc;
	let collection = mm
		.client
		.database(ctx.tenant_id().as_str())
		.collection::<D>(MC::TABLE);
	collection
		.update_one(
			filter, doc,
		)
		.await
		.map_err(|_| Error::UpdateError)?;
	Ok(())
}

pub async fn delete<MC, T>(
	ctx: &Ctx,
	mm: &ModelManager,
	filter: T,
) -> Result<()>
where
	MC: DbBmc,
	T: Serialize + DeserializeOwned + Send + Sync,
{
	let doc = Filter::doc(filter);
	let collection = mm
		.client
		.database(ctx.tenant_id().as_str())
		.collection::<T>(MC::TABLE);
	collection
		.delete_one(doc)
		.await
		.map_err(|_| Error::DeleteError)?;
	Ok(())
}

pub async fn get<MC, T>(
	ctx: &Ctx,
	mm: &ModelManager,
	filter: T,
) -> Result<T>
where
	MC: DbBmc,
	T: Serialize + DeserializeOwned + Send + Sync + Debug,
{
	let filter_doc = Filter::doc(filter);
	let collection = mm
		.client
		.database(ctx.tenant_id().as_str())
		.collection::<Document>(MC::TABLE);
	let result_doc = collection
		.find_one(filter_doc)
		.await
		.map_err(|_| Error::QueryError)?;
	let mut result_doc = result_doc.ok_or(Error::ReadError)?;

	if let Some(bson) = result_doc.get("_id") {
		if let Bson::ObjectId(oid) = bson {
			result_doc.insert(
				"_id".to_string(),
				Bson::String(oid.to_hex()),
			);
		}
	}

	let res: T = from_document(result_doc).map_err(|e| Error::ReadError)?;
	Ok(res)
}

pub async fn list<MC, T>(
	ctx: &Ctx,
	mm: &ModelManager,
	filter: T,
) -> Result<Vec<T>>
where
	MC: DbBmc,
	T: Serialize + DeserializeOwned + Send + Sync + Default,
{
	let doc = Filter::doc(filter);

	let collection = mm
		.client
		.database(ctx.tenant_id().as_str())
		.collection::<Document>(MC::TABLE);
	let cursor = collection.find(doc).await.map_err(|_| Error::QueryError)?;
	let docs: Vec<Document> = cursor.try_collect().await.map_err(|_| Error::QueryError)?;

	let mut res = Vec::new();
	for mut d in docs {
		if let Some(bson) = d.get("_id") {
			if let Bson::ObjectId(oid) = bson {
				d.insert(
					"_id".to_string(),
					Bson::String(oid.to_hex()),
				);
			}
		}
		let item: T = from_document(d).map_err(|_| Error::QueryError)?;
		res.push(item);
	}
	Ok(res)
}

#[cfg(test)]
mod tests {
	use super::*;
	use futures::stream::TryStreamExt;
	use mongodb::bson::{doc, Document};
	use serde::{Deserialize, Serialize};
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

	async fn clear_collection(
		mm: &ModelManager,
		tenant: &String,
	) -> Result<()> {
		let collection = mm
			.client
			.database(tenant)
			.collection::<Document>(DummyMC::TABLE);
		let col = collection
			.delete_many(doc! {})
			.await
			.map_err(|_| Error::DeleteError)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_create_and_get() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let tenant = ctx.tenant_id();
		let mm = ModelManager::new().await?;
		clear_collection(
			&mm, &tenant,
		)
		.await?;

		let data = DummyData {
			id: None,
			name: "Alice".to_string(),
			age: Some(30),
		};

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

		// Überprüfen, ob das Update erfolgreich war
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
		.await;

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

		// Zwei Dokumente erstellen
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

		// Alle Dokumente abrufen
		let l = list::<DummyMC, DummyData>(
			&ctx,
			&mm,
			Default::default(),
		)
		.await?;
		println!(
			"{:?}",
			l
		);
		Ok(())
	}
}
