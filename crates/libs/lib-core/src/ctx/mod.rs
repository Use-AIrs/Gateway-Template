// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

// endregion: --- Modules

#[cfg_attr(feature = "with-rpc", derive(rpc_router::RpcResource))]
#[derive(Clone, Debug)]
pub struct Ctx {
	user_id: String,
	tenant_id: String,
	conv_id: Option<String>,
}

// Constructors.
impl Ctx {
	pub fn root_ctx() -> Self {
		Ctx {
			user_id: "0000".to_string(),
			conv_id: None,
			tenant_id: "d1UuaOAUBRL2glq1eawbyKHqBgc".to_string(),
		}
	}
	pub fn new(user_id: String) -> Result<Self> {
		if user_id == "".to_string() {
			Err(Error::CtxCannotNewRootCtx)
		} else {
			Ok(
				Self {
					user_id,
					conv_id: None,
					tenant_id: "d1UuaOAUBRL2glq1eawbyKHqBgc".to_string(),
				},
			)
		}
	}
	pub fn add_conv_id(
		&self,
		conv_id: String,
	) -> Ctx {
		let mut ctx = self.clone();
		ctx.conv_id = Some(conv_id);
		ctx
	}
	pub fn add_tenant_id(
		&self,
		tenant_id: String,
	) -> Ctx {
		let mut ctx = self.clone();
		ctx.tenant_id = tenant_id;
		ctx
	}
}

// Property Accessors.
impl Ctx {
	pub fn user_id(&self) -> String {
		self.user_id.clone()
	}
	pub fn conv_id(&self) -> Option<String> {
		self.conv_id.clone()
	}
	pub fn tenant_id(&self) -> String {
		self.tenant_id.clone()
	}
}
