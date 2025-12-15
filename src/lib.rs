mod compare;
#[cfg(feature = "js")]
pub mod legacy_planner;
mod pretty_plan;

//=================================================================================================
// Re-export underlying crates

pub use apollo_compiler;
pub use apollo_federation;

//=================================================================================================
// Export semantic diff functions

pub use compare::diff_plan;
pub use compare::plan_matches;
pub use pretty_plan::pretty_query_plan;

//=================================================================================================
// Helper function for running the native query planner

pub use apollo_federation::error::FederationError;
pub use apollo_federation::query_plan::QueryPlan as NativeQueryPlan;
pub use apollo_federation::query_plan::query_planner as native_planner;

pub fn run_native_planner(
    schema_str: &str,
    query_str: &str,
    query_name: Option<apollo_compiler::Name>,
    query_path: impl AsRef<std::path::Path>,
    config: native_planner::QueryPlannerConfig,
    plan_options: native_planner::QueryPlanOptions,
) -> Result<NativeQueryPlan, FederationError> {
    let supergraph = apollo_federation::Supergraph::new_with_router_specs(schema_str).unwrap();
    let planner = native_planner::QueryPlanner::new(&supergraph, config)?;
    let query_doc = apollo_compiler::ExecutableDocument::parse_and_validate(
        planner.api_schema().schema(),
        query_str,
        query_path,
    )?;
    planner.build_query_plan(&query_doc, query_name, plan_options)
}
