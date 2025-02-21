pub mod router;

//=================================================================================================
// Re-export underlying crates

pub use apollo_compiler;
pub use apollo_federation;
pub use router_bridge;

//=================================================================================================
// Export semantic diff functions

pub use crate::router::plan_compare::diff_plan;
pub use crate::router::plan_compare::plan_matches;
pub use crate::router::plan_compare::render_diff;
pub use crate::router::render_legacy_plan;
pub use crate::router::render_native_plan;

//=================================================================================================
// Helper functions for running query planners

pub use crate::router::QueryPlanResult as LegacyQueryPlanResult;
pub use apollo_federation::error::FederationError;
pub use apollo_federation::query_plan::QueryPlan as NativeQueryPlan;
pub use apollo_federation::query_plan::query_planner as native_planner;
pub use router_bridge::planner as legacy_planner;

pub fn run_native_planner(
    schema_str: &str,
    query_str: &str,
    query_name: Option<apollo_compiler::Name>,
    query_path: impl AsRef<std::path::Path>,
    config: native_planner::QueryPlannerConfig,
    plan_options: native_planner::QueryPlanOptions,
) -> Result<NativeQueryPlan, FederationError> {
    let supergraph = apollo_federation::Supergraph::new(schema_str).unwrap();
    let planner = native_planner::QueryPlanner::new(&supergraph, config)?;
    let query_doc = apollo_compiler::ExecutableDocument::parse_and_validate(
        planner.api_schema().schema(),
        query_str,
        query_path,
    )?;
    let plan = planner.build_query_plan(&query_doc, query_name, plan_options)?;
    Ok(plan)
}

pub fn run_legacy_planner(
    schema_str: &str,
    query_str: &str,
    query_name: Option<String>,
    config: legacy_planner::QueryPlannerConfig,
    plan_options: legacy_planner::PlanOptions,
) -> Result<LegacyQueryPlanResult, Vec<String>> {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let planner = runtime
        .block_on(legacy_planner::Planner::new(schema_str.to_string(), config))
        .unwrap();
    let result = runtime
        .block_on(planner.plan(query_str.to_string(), query_name, plan_options))
        .unwrap();
    if let Some(errors) = result.errors {
        return Err(errors.iter().map(|e| e.to_string()).collect());
    }
    Ok(result.data.unwrap())
}
