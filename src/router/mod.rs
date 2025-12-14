//! In order to avoid importing the `apollo-router` crate, some of its code is duplicated here.

mod convert;
pub(crate) mod path;
pub(crate) mod plan;
#[allow(dead_code)]
pub(crate) mod plan_compare;

use std::sync::Arc;

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
    pub(crate) query_plan: self::plan::QueryPlan,
}
