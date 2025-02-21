// Copied from `apollo-router/src/query_planner/plan.rs`.

use std::sync::Arc;

use apollo_compiler::ExecutableDocument;
use apollo_compiler::Name;
use apollo_compiler::ast;
use apollo_compiler::validation::Valid;
use serde::Deserialize;
use serde::Serialize;
use serde_json_bytes::Value;

use crate::router::path::Path;
use crate::router::selection::Selection;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The root query plan container.
pub(super) struct QueryPlan {
    /// The hierarchical nodes that make up the query plan
    pub(super) node: Option<Arc<PlanNode>>,
}

/// Query plans are composed of a set of nodes.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", tag = "kind")]
pub(crate) enum PlanNode {
    /// These nodes must be executed in order.
    Sequence {
        /// The plan nodes that make up the sequence execution.
        nodes: Vec<PlanNode>,
    },

    /// These nodes may be executed in parallel.
    Parallel {
        /// The plan nodes that make up the parallel execution.
        nodes: Vec<PlanNode>,
    },

    /// Fetch some data from a subgraph.
    Fetch(FetchNode),

    /// Merge the current resultset with the response.
    Flatten(FlattenNode),

    Defer {
        primary: Primary,
        deferred: Vec<DeferredNode>,
    },

    Subscription {
        primary: SubscriptionNode,
        rest: Option<Box<PlanNode>>,
    },

    #[serde(rename_all = "camelCase")]
    Condition {
        condition: String,
        if_clause: Option<Box<PlanNode>>,
        else_clause: Option<Box<PlanNode>>,
    },
}

/// A flatten node.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FlattenNode {
    /// The path when result should be merged.
    pub(crate) path: Path,

    /// The child execution plan.
    pub(crate) node: Box<PlanNode>,
}

/// A primary query for a Defer node, the non deferred part
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Primary {
    /// The part of the original query that "selects" the data to
    /// send in that primary response (once the plan in `node` completes).
    pub(crate) subselection: Option<String>,

    // The plan to get all the data for that primary part
    pub(crate) node: Option<Box<PlanNode>>,
}

/// The "deferred" parts of the defer (note that it's an array). Each
/// of those deferred elements will correspond to a different chunk of
/// the response to the client (after the initial non-deferred one that is).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeferredNode {
    /// References one or more fetch node(s) (by `id`) within
    /// `primary.node`. The plan of this deferred part should not
    /// be started before all those fetches returns.
    pub(crate) depends: Vec<Depends>,

    /// The optional defer label.
    pub(crate) label: Option<String>,
    /// Path to the @defer this correspond to. `subselection` start at that `path`.
    pub(crate) query_path: Path,
    /// The part of the original query that "selects" the data to send
    /// in that deferred response (once the plan in `node` completes).
    /// Will be set _unless_ `node` is a `DeferNode` itself.
    pub(crate) subselection: Option<String>,
    /// The plan to get all the data for that deferred part
    pub(crate) node: Option<Arc<PlanNode>>,
}

/// A deferred node.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Depends {
    pub(crate) id: String,
}

/// GraphQL operation type.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum OperationKind {
    #[default]
    Query,
    Mutation,
    Subscription,
}

impl From<OperationKind> for ast::OperationType {
    fn from(value: OperationKind) -> Self {
        match value {
            OperationKind::Query => ast::OperationType::Query,
            OperationKind::Mutation => ast::OperationType::Mutation,
            OperationKind::Subscription => ast::OperationType::Subscription,
        }
    }
}

impl From<ast::OperationType> for OperationKind {
    fn from(value: ast::OperationType) -> Self {
        match value {
            ast::OperationType::Query => OperationKind::Query,
            ast::OperationType::Mutation => OperationKind::Mutation,
            ast::OperationType::Subscription => OperationKind::Subscription,
        }
    }
}

/// A fetch node.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FetchNode {
    /// The name of the service or subgraph that the fetch is querying.
    pub(crate) service_name: Arc<str>,

    /// The data that is required for the subgraph fetch.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) requires: Vec<Selection>,

    /// The variables that are used for the subgraph fetch.
    pub(crate) variable_usages: Vec<Arc<str>>,

    /// The GraphQL subquery that is used for the fetch.
    pub(crate) operation: SubgraphOperation,

    /// The GraphQL subquery operation name.
    pub(crate) operation_name: Option<Arc<str>>,

    /// The GraphQL operation kind that is used for the fetch.
    pub(crate) operation_kind: OperationKind,

    /// Optional id used by Deferred nodes
    pub(crate) id: Option<String>,

    // Optionally describes a number of "rewrites" that query plan executors should apply to the data that is sent as input of this fetch.
    pub(crate) input_rewrites: Option<Vec<DataRewrite>>,

    // Optionally describes a number of "rewrites" to apply to the data that received from a fetch (and before it is applied to the current in-memory results).
    pub(crate) output_rewrites: Option<Vec<DataRewrite>>,

    // Optionally describes a number of "rewrites" to apply to the data that has already been received further up the tree
    pub(crate) context_rewrites: Option<Vec<DataRewrite>>,
}

#[derive(Clone)]
pub(crate) struct SubgraphOperation {
    serialized: String,
    // /// Ideally this would be always present, but we don’t have access to the subgraph schemas
    // /// during `Deserialize`.
    // parsed: Option<Arc<Valid<ExecutableDocument>>>,
}

impl SubgraphOperation {
    pub(crate) fn from_string(serialized: impl Into<String>) -> Self {
        Self {
            serialized: serialized.into(),
            // parsed: None,
        }
    }

    pub(crate) fn from_parsed(parsed: impl Into<Arc<Valid<ExecutableDocument>>>) -> Self {
        let parsed = parsed.into();
        Self {
            serialized: parsed.serialize().no_indent().to_string(),
            // parsed: Some(parsed),
        }
    }

    pub(crate) fn as_serialized(&self) -> &str {
        &self.serialized
    }
}

impl Serialize for SubgraphOperation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_serialized().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SubgraphOperation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_string(String::deserialize(deserializer)?))
    }
}

impl PartialEq for SubgraphOperation {
    fn eq(&self, other: &Self) -> bool {
        self.as_serialized() == other.as_serialized()
    }
}

impl std::fmt::Debug for SubgraphOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_serialized(), f)
    }
}

impl std::fmt::Display for SubgraphOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_serialized(), f)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", tag = "kind")]
pub(crate) enum DataRewrite {
    ValueSetter(DataValueSetter),
    KeyRenamer(DataKeyRenamer),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataValueSetter {
    pub(crate) path: Path,
    pub(crate) set_value_to: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataKeyRenamer {
    pub(crate) path: Path,
    pub(crate) rename_key_to: Name,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionNode {
    /// The name of the service or subgraph that the subscription is querying.
    pub(crate) service_name: Arc<str>,

    /// The variables that are used for the subgraph subscription.
    pub(crate) variable_usages: Vec<Arc<str>>,

    /// The GraphQL subquery that is used for the subscription.
    pub(crate) operation: SubgraphOperation,

    /// The GraphQL subquery operation name.
    pub(crate) operation_name: Option<Arc<str>>,

    /// The GraphQL operation kind that is used for the fetch.
    pub(crate) operation_kind: OperationKind,

    // Optionally describes a number of "rewrites" that query plan executors should apply to the data that is sent as input of this subscription.
    pub(crate) input_rewrites: Option<Vec<DataRewrite>>,

    // Optionally describes a number of "rewrites" to apply to the data that received from a subscription (and before it is applied to the current in-memory results).
    pub(crate) output_rewrites: Option<Vec<DataRewrite>>,
}
