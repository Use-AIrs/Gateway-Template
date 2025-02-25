use proc_mac::crud;
use serde::{Deserialize, Serialize};

/// Generates RPC conversion types and CRUD functions. The macro creates:
/// - ForCreate: All fields (except "id" and those marked with #[skip_create]) as Option types.
/// - ForUpdate: All fields (except "id" and those marked with #[skip_update]) as Option types.
/// - Filter: All fields (except those marked with #[skip_filter]) as Option types.
/// - A BMC struct implementing DbBmc with TABLE set to "<StructName>Bmc".
/// - CRUD functions in the BMC impl that convert the provided types into the main struct,
///   filling missing fields with None.

#[derive(Debug, Serialize, Deserialize, Default)]
#[crud]
pub struct Example {
	pub id: Option<String>,
	pub name: Option<String>,
	pub description: Option<String>,
	pub skills: Option<Vec<String>>,
}
