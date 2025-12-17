// Update NativeQueryPlan's operation_document field to be pretty-printed.

use std::sync::Arc;

use apollo_compiler::ExecutableDocument;
use apollo_compiler::collections::IndexMap;
use apollo_federation::query_plan::FetchNode;
use apollo_federation::query_plan::PlanNode;
use apollo_federation::query_plan::QueryPlan;
use apollo_federation::query_plan::TopLevelPlanNode;
use apollo_federation::query_plan::query_planner;
use apollo_federation::query_plan::serializable_document::SerializableDocument;
use apollo_federation::schema::ValidFederationSchema;

pub fn pretty_query_plan(schema_str: &str, plan: &QueryPlan) -> QueryPlan {
    let node = plan
        .node
        .as_ref()
        .map(|node| pretty_query_plan_node(schema_str, node));
    let statistics = plan.statistics.clone();
    QueryPlan { node, statistics }
}

pub fn pretty_query_plan_node(schema_str: &str, node: &TopLevelPlanNode) -> TopLevelPlanNode {
    let mut node = node.clone();
    traverse_top_level_plan_node(&subgraph_schemas(schema_str), &mut node);
    node
}

fn subgraph_schemas(schema_str: &str) -> IndexMap<Arc<str>, ValidFederationSchema> {
    let supergraph = apollo_federation::Supergraph::new_with_router_specs(schema_str).unwrap();
    let planner = query_planner::QueryPlanner::new(&supergraph, Default::default()).unwrap();
    planner.subgraph_schemas().clone()
}

fn pretty_fetch_node(schema: &ValidFederationSchema, fetch: &mut FetchNode) {
    // Update `operation_document` with a pretty-printed version.
    // - Note: `SerializableDocument::from_parsed` can't be used, since it uses unindented text.
    // - Note: This loses the parsed AST.
    if let Ok(doc) = fetch.operation_document.as_parsed() {
        fetch.operation_document = SerializableDocument::from_string(doc.to_string());
    } else {
        // Parse the serialized document
        let doc = ExecutableDocument::parse_and_validate(
            schema.schema(),
            fetch.operation_document.as_serialized(),
            "operation.graphql",
        )
        .unwrap();
        fetch.operation_document = SerializableDocument::from_string(doc.to_string());
    }
    // Re-parse the operation_document
    // - This enables to use the `Display` implementation of `FetchNode`.
    fetch
        .operation_document
        .init_parsed(schema.schema())
        .unwrap();
}

fn traverse_plan_node(
    subgraph_schemas: &IndexMap<Arc<str>, ValidFederationSchema>,
    node: &mut PlanNode,
) {
    match node {
        PlanNode::Fetch(fetch_node) => {
            let schema = subgraph_schemas.get(&fetch_node.subgraph_name).unwrap();
            pretty_fetch_node(schema, fetch_node);
        }
        PlanNode::Sequence(seq_node) => {
            for child in &mut seq_node.nodes {
                traverse_plan_node(subgraph_schemas, child);
            }
        }
        PlanNode::Parallel(par_node) => {
            for child in &mut par_node.nodes {
                traverse_plan_node(subgraph_schemas, child);
            }
        }
        PlanNode::Flatten(flat_node) => {
            traverse_plan_node(subgraph_schemas, &mut flat_node.node);
        }
        PlanNode::Defer(defer_node) => {
            if let Some(node) = &mut defer_node.primary.node {
                traverse_plan_node(subgraph_schemas, node);
            }
            for deferred in &mut defer_node.deferred {
                if let Some(node) = &mut deferred.node {
                    traverse_plan_node(subgraph_schemas, node);
                }
            }
        }
        PlanNode::Condition(cond_node) => {
            if let Some(if_clause) = &mut cond_node.if_clause {
                traverse_plan_node(subgraph_schemas, if_clause);
            }
            if let Some(else_clause) = &mut cond_node.else_clause {
                traverse_plan_node(subgraph_schemas, else_clause);
            }
        }
    }
}

fn traverse_top_level_plan_node(
    subgraph_schemas: &IndexMap<Arc<str>, ValidFederationSchema>,
    node: &mut TopLevelPlanNode,
) {
    match node {
        TopLevelPlanNode::Fetch(fetch_node) => {
            let schema = subgraph_schemas.get(&fetch_node.subgraph_name).unwrap();
            pretty_fetch_node(schema, fetch_node);
        }
        TopLevelPlanNode::Sequence(seq_node) => {
            for child in &mut seq_node.nodes {
                traverse_plan_node(subgraph_schemas, child);
            }
        }
        TopLevelPlanNode::Parallel(par_node) => {
            for child in &mut par_node.nodes {
                traverse_plan_node(subgraph_schemas, child);
            }
        }
        TopLevelPlanNode::Flatten(flat_node) => {
            traverse_plan_node(subgraph_schemas, &mut flat_node.node);
        }
        TopLevelPlanNode::Defer(defer_node) => {
            if let Some(node) = &mut defer_node.primary.node {
                traverse_plan_node(subgraph_schemas, node);
            }
            for deferred in &mut defer_node.deferred {
                if let Some(node) = &mut deferred.node {
                    traverse_plan_node(subgraph_schemas, node);
                }
            }
        }
        TopLevelPlanNode::Condition(cond_node) => {
            if let Some(if_clause) = &mut cond_node.if_clause {
                traverse_plan_node(subgraph_schemas, if_clause);
            }
            if let Some(else_clause) = &mut cond_node.else_clause {
                traverse_plan_node(subgraph_schemas, else_clause);
            }
        }
        TopLevelPlanNode::Subscription(sub_node) => {
            let schema = subgraph_schemas
                .get(&sub_node.primary.subgraph_name)
                .unwrap();
            pretty_fetch_node(schema, sub_node.primary.as_mut());
            if let Some(rest) = &mut sub_node.rest {
                traverse_plan_node(subgraph_schemas, rest);
            }
        }
    }
}
