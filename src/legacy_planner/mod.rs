mod convert;
mod router;

use apollo_federation::query_plan::QueryPlan as NativeQueryPlan;

// Re-exports
pub use router::QueryPlanResult;
pub use router::path::Path;
pub use router_bridge;
pub use router_bridge::planner::IncrementalDeliverySupport;
pub use router_bridge::planner::PlanOptions;
pub use router_bridge::planner::Planner;
pub use router_bridge::planner::QueryPlannerConfig;

pub fn run_legacy_planner(
    schema_str: &str,
    query_str: &str,
    query_name: Option<String>,
    config: QueryPlannerConfig,
    plan_options: PlanOptions,
) -> Result<QueryPlanResult, Vec<String>> {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let planner = runtime
        .block_on(Planner::new(schema_str.to_string(), config))
        .unwrap();
    let result = runtime
        .block_on(planner.plan(query_str.to_string(), query_name, plan_options))
        .unwrap();
    if let Some(errors) = result.errors {
        return Err(errors.iter().map(|e| e.to_string()).collect());
    }
    Ok(result.data.unwrap())
}

pub fn convert_legacy_query_plan(js_plan: &QueryPlanResult) -> NativeQueryPlan {
    convert::convert_root_query_plan_node(&js_plan.query_plan)
}
