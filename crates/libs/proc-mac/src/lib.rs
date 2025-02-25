extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident, LitStr, Type};

fn is_option(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty {
		if let Some(segment) = type_path.path.segments.first() {
			return segment.ident == "Option";
		}
	}
	false
}

fn wrap_in_option(ty: &Type) -> proc_macro2::TokenStream {
	if is_option(ty) {
		quote! { #ty }
	} else {
		quote! { Option<#ty> }
	}
}

fn has_skip_attribute(
	field: &Field,
	attr_name: &str,
) -> bool {
	field.attrs.iter().any(|attr| attr.path.is_ident(attr_name))
}

/// Generates RPC conversion types and CRUD functions. The macro creates:
/// - ForCreate: All fields (except "id" and those marked with #[skip_create]) as Option types.
/// - ForUpdate: All fields (except "id" and those marked with #[skip_update]) as Option types.
/// - Filter: All fields (except those marked with #[skip_filter]) as Option types.
/// - A BMC struct implementing DbBmc with TABLE set to "<StructName>Bmc".
/// - CRUD functions in the BMC impl that convert the provided types into the main struct,
///   filling missing fields with None.
#[proc_macro_attribute]
pub fn crud(
	_attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	let input_ast = parse_macro_input!(item as DeriveInput);
	let struct_ident = &input_ast.ident;

	let for_create_ident = Ident::new(
		&format!(
			"{}ForCreate",
			struct_ident
		),
		struct_ident.span(),
	);
	let for_update_ident = Ident::new(
		&format!(
			"{}ForUpdate",
			struct_ident
		),
		struct_ident.span(),
	);
	let filter_ident = Ident::new(
		&format!(
			"{}Filter",
			struct_ident
		),
		struct_ident.span(),
	);
	let bmc_ident = Ident::new(
		&format!(
			"{}Bmc",
			struct_ident
		),
		struct_ident.span(),
	);

	let table_name = format!(
		"{}Bmc",
		struct_ident
	);
	let table_name_literal = LitStr::new(
		&table_name,
		struct_ident.span(),
	);

	let fields = if let Data::Struct(data_struct) = &input_ast.data {
		if let Fields::Named(fields_named) = &data_struct.fields {
			fields_named.named.iter().collect::<Vec<&Field>>()
		} else {
			panic!("Only structs with named fields are supported!");
		}
	} else {
		panic!("Only structs are supported!");
	};

	let create_fields = fields
		.iter()
		.filter(
			|field| {
				if let Some(ident) = &field.ident {
					if ident == "id"
						|| has_skip_attribute(
							field,
							"skip_create",
						) {
						return false;
					}
				}
				true
			},
		)
		.map(
			|field| {
				let ident = &field.ident;
				let ty = &field.ty;
				let new_ty = wrap_in_option(ty);
				quote! { pub #ident: #new_ty, }
			},
		);

	let update_fields = fields
		.iter()
		.filter(
			|field| {
				if let Some(ident) = &field.ident {
					if ident == "id"
						|| has_skip_attribute(
							field,
							"skip_update",
						) {
						return false;
					}
				}
				true
			},
		)
		.map(
			|field| {
				let ident = &field.ident;
				let ty = &field.ty;
				let new_ty = wrap_in_option(ty);
				quote! { pub #ident: #new_ty, }
			},
		);

	let filter_fields = fields
		.iter()
		.filter(
			|field| {
				if has_skip_attribute(
					field,
					"skip_filter",
				) {
					return false;
				}
				true
			},
		)
		.map(
			|field| {
				let ident = &field.ident;
				let ty = &field.ty;
				let new_ty = wrap_in_option(ty);
				quote! { pub #ident: #new_ty, }
			},
		);

	let create_assignments = fields.iter().map(
		|field| {
			let ident = field.ident.as_ref().unwrap();
			if ident == "id"
				|| has_skip_attribute(
					field,
					"skip_create",
				) {
				quote! { #ident: None, }
			} else {
				quote! { #ident: input.#ident, }
			}
		},
	);

	let update_assignments = fields.iter().map(
		|field| {
			let ident = field.ident.as_ref().unwrap();
			if ident == "id"
				|| has_skip_attribute(
					field,
					"skip_update",
				) {
				quote! { #ident: None, }
			} else {
				quote! { #ident: input.#ident, }
			}
		},
	);

	let filter_assignments = fields.iter().map(
		|field| {
			let ident = field.ident.as_ref().unwrap();
			if has_skip_attribute(
				field,
				"skip_filter",
			) {
				quote! { #ident: None, }
			} else {
				quote! { #ident: input.#ident, }
			}
		},
	);

	let expanded = quote! {
		#input_ast

		#[derive(Debug, Serialize, Deserialize, Default, Clone)]
		pub struct #for_create_ident {
			#(#create_fields)*
		}

		#[derive(Debug, Serialize, Deserialize, Default, Clone)]
		pub struct #for_update_ident {
			#(#update_fields)*
		}

		#[derive(Debug, Serialize, Deserialize, Default, Clone)]
		pub struct #filter_ident {
			#(#filter_fields)*
		}

		pub struct #bmc_ident;
		impl crate::model::base::DbBmc for #bmc_ident {
			const TABLE: &'static str = #table_name_literal;
		}

		impl #bmc_ident {
			pub async fn create(ctx: &crate::ctx::Ctx, mm: &crate::model::ModelManager, input: #for_create_ident) -> crate::model::Result<String> {
				let entity: #struct_ident = input.into();
				crate::model::base::create::<Self, #struct_ident>(ctx, mm, entity).await
			}

			pub async fn update(ctx: &crate::ctx::Ctx, mm: &crate::model::ModelManager, id: &String, input: #for_update_ident) -> crate::model::Result<()> {
				let entity: #struct_ident = input.into();
				crate::model::base::update::<Self, #struct_ident>(ctx, mm, id, entity).await
			}

			pub async fn get(ctx: &crate::ctx::Ctx, mm: &crate::model::ModelManager, filter: Option<#filter_ident>) -> crate::model::Result<#struct_ident> {
				let stru = filter.unwrap_or_default();
				let entity: #struct_ident = stru.into();
				crate::model::base::get::<Self, #struct_ident>(ctx, mm, entity).await

			}

			pub async fn list(ctx: &crate::ctx::Ctx, mm: &crate::model::ModelManager, filter: Option<#filter_ident>) -> crate::model::Result<Vec<#struct_ident>> {
				let stru = filter.unwrap_or_default();
				let entity: #struct_ident = stru.into();
				crate::model::base::list::<Self, #struct_ident>(ctx, mm, entity).await
			}

			pub async fn delete(ctx: &crate::ctx::Ctx, mm: &crate::model::ModelManager, filter: Option<#filter_ident>) -> crate::model::Result<()> {
				let stru = filter.unwrap_or_default();
				let entity: #struct_ident = stru.into();
				crate::model::base::delete::<Self, #struct_ident>(ctx, mm, entity).await
			}
		}

		impl From<#for_create_ident> for #struct_ident {
			fn from(input: #for_create_ident) -> Self {
				Self { #(#create_assignments)* }
			}
		}

		impl From<#for_update_ident> for #struct_ident {
			fn from(input: #for_update_ident) -> Self {
				Self { #(#update_assignments)* }
			}
		}

		impl From<#filter_ident> for #struct_ident {
			fn from(input: #filter_ident) -> Self {
				Self { #(#filter_assignments)* }
			}
		}
	};

	TokenStream::from(expanded)
}
