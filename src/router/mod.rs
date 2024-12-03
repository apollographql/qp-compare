//! In order to avoid importing the `apollo-router` crate, some of its code is duplicated here.

mod convert;
mod path;
mod plan;
pub(crate) mod plan_compare;
mod selection;

use std::sync::Arc;

use apollo_federation::query_plan::QueryPlan as NativeQueryPlan;
pub(crate) use plan::*;
use serde::Deserialize;

//=================================================================================================
// This section is copied from `apollo-router/src/query_planner/bridge_query_planner.rs`.

/// Data coming from the `plan` method on the router_bridge
// Note: Reexported under `apollo_compiler::_private`
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryPlanResult {
    pub formatted_query_plan: Option<Arc<String>>,
    query_plan: self::plan::QueryPlan,
}

//=================================================================================================
// Render plans in the same formatting used by `diff_plan`.

type LegacyQueryPlanResult = QueryPlanResult;

pub fn render_legacy_plan(js_plan: &LegacyQueryPlanResult) -> String {
    let js_root_node = &js_plan.query_plan.node;
    match js_root_node {
        None => String::from(""),
        Some(js) => format!("{js:#?}"),
    }
}

pub fn render_native_plan(rust_plan: &NativeQueryPlan) -> String {
    let rust_root_node = convert::convert_root_query_plan_node(rust_plan);

    match rust_root_node {
        None => String::from(""),
        Some(rust) => format!("{rust:#?}"),
    }
}
