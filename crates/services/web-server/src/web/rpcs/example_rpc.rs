use lib_rpc_core::prelude::*;

use lib_core::model::example::{
	Example, ExampleBmc, ExampleFilter, ExampleForCreate, ExampleForUpdate,
};

pub fn rpc_router_builder() -> RouterBuilder {
	router_builder!(
		create_example,
		get_example,
		list_examples,
		update_example,
		delete_example,
	)
}

generate_common_rpc_fns!(
	Bmc: ExampleBmc,
	Entity: Example,
	ForCreate: ExampleForCreate,
	ForUpdate: ExampleForUpdate,
	Filter: ExampleFilter,
	Suffix: example
);
