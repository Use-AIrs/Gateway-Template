/// Create the base crud rpc functions following the common pattern.
/// - `create_...`
/// - `get_...`
///
/// NOTE: Make sure to import the Ctx, ModelManager, ... in the model that uses this macro.
#[macro_export]
macro_rules! generate_common_rpc_fns {
	(
        Bmc: $bmc:ident,
        Entity: $entity:ty,
        ForCreate: $for_create:ty,
        ForUpdate: $for_update:ty,
        Filter: $filter:ty,
        Suffix: $suffix:ident
    ) => {
		paste! {
			pub async fn [<create_ $suffix>](
				ctx: lib_core::ctx::Ctx,
				mm: lib_core::model::ModelManager,
				params: ParamsForCreate<$for_create>,
			) -> Result<DataRpcResult<$entity>> {
				let ParamsForCreate { data } = params;
				let id = $bmc::create(&ctx, &mm, data).await?;
				let mut filter: $filter = Default::default();
				filter.id = Some(id);
				let entity = $bmc::get(&ctx, &mm, Some(filter)).await?;
				Ok(entity.into())
			}

			pub async fn [<get_ $suffix>](
				ctx: lib_core::ctx::Ctx,
				mm: lib_core::model::ModelManager,
				params: ParamsIded,
			) -> Result<DataRpcResult<$entity>> {
				let ParamsIded { id } = params;
				let mut filter: $filter = Default::default();
				filter.id = Some(id);
				let entity = $bmc::get(&ctx, &mm, Some(filter)).await?;
				Ok(entity.into())
			}

			pub async fn [<list_ $suffix s>](
				ctx: lib_core::ctx::Ctx,
				mm: lib_core::model::ModelManager,
				params: ParamsList<$filter>,
			) -> Result<DataRpcResult<Vec<$entity>>> {
				let ParamsList { filter } = params;
				let entities = $bmc::list(&ctx, &mm, filter).await?;
				Ok(entities.into())
			}

			pub async fn [<update_ $suffix>](
				ctx: lib_core::ctx::Ctx,
				mm: lib_core::model::ModelManager,
				params: ParamsForUpdate<$for_update>,
			) -> Result<DataRpcResult<$entity>> {
				let ParamsForUpdate { id, data } = params;
				$bmc::update(&ctx, &mm, &id, data).await?;
				let mut filter: $filter = Default::default();
				filter.id = Some(id);
				let entity = $bmc::get(&ctx, &mm, Some(filter)).await?;
				Ok(entity.into())
			}

			pub async fn [<delete_ $suffix>](
				ctx: lib_core::ctx::Ctx,
				mm: lib_core::model::ModelManager,
				params: ParamsIded,
			) -> Result<DataRpcResult<$entity>> {
				let ParamsIded { id } = params;
				let mut filter: $filter = Default::default();
				filter.id = Some(id);
				let entity = $bmc::get(&ctx, &mm, Some(filter.clone())).await?;
				$bmc::delete(&ctx, &mm, Some(filter)).await?;
				Ok(entity.into())
			}
		}
	};
}
