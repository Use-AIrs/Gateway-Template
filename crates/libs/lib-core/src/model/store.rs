use crate::core_config;

use mongodb::error::Result;
use mongodb::Client;

pub async fn new_client() -> Result<Client> {
	Client::with_uri_str(&core_config().DB_URL).await
}
