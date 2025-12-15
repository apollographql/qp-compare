// Convert the router QueryPlan into the native QueryPlan from apollo-federation
use std::sync::Arc;

use apollo_compiler::Name;
use apollo_federation::query_plan as next;

use super::router as legacy;
use legacy::PathElement;

pub(crate) fn convert_root_query_plan_node(plan: &legacy::QueryPlan) -> next::QueryPlan {
    let legacy::QueryPlan { node } = plan;
    next::QueryPlan {
        node: node.as_ref().map(|n| n.as_ref().into()),
        statistics: Default::default(),
    }
}

impl From<&'_ legacy::PlanNode> for next::TopLevelPlanNode {
    fn from(value: &'_ legacy::PlanNode) -> Self {
        match value {
            legacy::PlanNode::Fetch(node) => Self::Fetch(into_box(node)),
            legacy::PlanNode::Sequence { nodes } => Self::Sequence(next::SequenceNode {
                nodes: into_vec(nodes),
            }),
            legacy::PlanNode::Parallel { nodes } => Self::Parallel(next::ParallelNode {
                nodes: into_vec(nodes),
            }),
            legacy::PlanNode::Flatten(node) => Self::Flatten(node.into()),
            legacy::PlanNode::Defer { primary, deferred } => Self::Defer(next::DeferNode {
                primary: primary.into(),
                deferred: into_vec(deferred),
            }),
            legacy::PlanNode::Condition {
                condition,
                if_clause,
                else_clause,
            } => Self::Condition(from_legacy_condition_node(
                condition,
                if_clause,
                else_clause,
            )),
            legacy::PlanNode::Subscription { primary, rest } => {
                Self::Subscription(from_legacy_subscription_node(primary, rest))
            }
        }
    }
}

impl From<&'_ legacy::PlanNode> for next::PlanNode {
    fn from(value: &'_ legacy::PlanNode) -> Self {
        match value {
            legacy::PlanNode::Fetch(node) => Self::Fetch(into_box(node)),
            legacy::PlanNode::Sequence { nodes } => Self::Sequence(next::SequenceNode {
                nodes: into_vec(nodes),
            }),
            legacy::PlanNode::Parallel { nodes } => Self::Parallel(next::ParallelNode {
                nodes: into_vec(nodes),
            }),
            legacy::PlanNode::Flatten(node) => Self::Flatten(node.into()),
            legacy::PlanNode::Defer { primary, deferred } => Self::Defer(next::DeferNode {
                primary: primary.into(),
                deferred: into_vec(deferred),
            }),
            legacy::PlanNode::Condition {
                condition,
                if_clause,
                else_clause,
            } => Self::Condition(from_legacy_condition_node(
                condition,
                if_clause,
                else_clause,
            )),
            legacy::PlanNode::Subscription { .. } => {
                panic!("Subscription nodes should only appear at the top level")
            }
        }
    }
}

fn from_legacy_condition_node(
    condition: &str,
    if_clause: &Option<Box<legacy::PlanNode>>,
    else_clause: &Option<Box<legacy::PlanNode>>,
) -> Box<next::ConditionNode> {
    Box::new(next::ConditionNode {
        condition_variable: Name::new(condition).expect("valid condition variable name"),
        if_clause: into_box_option(if_clause),
        else_clause: into_box_option(else_clause),
    })
}

impl From<&'_ legacy::FetchNode> for next::FetchNode {
    fn from(value: &'_ legacy::FetchNode) -> Self {
        let legacy::FetchNode {
            service_name,
            id,
            requires,
            variable_usages,
            operation,
            operation_name,
            operation_kind,
            input_rewrites,
            output_rewrites,
            context_rewrites,
        } = value;
        Self {
            subgraph_name: service_name.clone(),
            id: id.as_ref().and_then(|s| s.parse().ok()),
            variable_usages: variable_usages
                .iter()
                .map(|v| Name::new(v).expect("valid variable name"))
                .collect(),
            requires: requires.clone(),
            operation_document: operation.clone(),
            operation_name: operation_name
                .as_ref()
                .map(|n| Name::new(n.as_ref()).expect("valid operation name")),
            operation_kind: (*operation_kind).into(),
            input_rewrites: Arc::new(into_arc_vec_option(input_rewrites)),
            output_rewrites: into_arc_vec_option(output_rewrites),
            context_rewrites: into_arc_vec_option(context_rewrites),
        }
    }
}

fn from_legacy_subscription_node(
    primary: &legacy::SubscriptionNode,
    rest: &Option<Box<legacy::PlanNode>>,
) -> next::SubscriptionNode {
    let legacy::SubscriptionNode {
        service_name,
        variable_usages,
        operation,
        operation_name,
        operation_kind,
        input_rewrites,
        output_rewrites,
    } = primary;
    let primary = next::FetchNode {
        subgraph_name: service_name.clone(),
        id: None,
        variable_usages: variable_usages
            .iter()
            .map(|v| Name::new(v).expect("valid variable name"))
            .collect(),
        requires: vec![],
        operation_document: operation.clone(),
        operation_name: operation_name
            .as_ref()
            .map(|n| Name::new(n.as_ref()).expect("valid operation name")),
        operation_kind: (*operation_kind).into(),
        input_rewrites: Arc::new(into_arc_vec_option(input_rewrites)),
        output_rewrites: into_arc_vec_option(output_rewrites),
        context_rewrites: vec![],
    };
    next::SubscriptionNode {
        primary: into_box(primary),
        rest: into_box_option(rest),
    }
}

impl From<&'_ legacy::FlattenNode> for next::FlattenNode {
    fn from(value: &'_ legacy::FlattenNode) -> Self {
        let legacy::FlattenNode { path, node } = value;
        Self {
            path: into_vec(&path.0),
            node: into_box(node.as_ref()),
        }
    }
}

impl From<&'_ legacy::Primary> for next::PrimaryDeferBlock {
    fn from(value: &'_ legacy::Primary) -> Self {
        let legacy::Primary { node, subselection } = value;
        Self {
            sub_selection: subselection.clone(),
            node: into_box_option(node),
        }
    }
}

impl From<&'_ legacy::DeferredNode> for next::DeferredDeferBlock {
    fn from(value: &'_ legacy::DeferredNode) -> Self {
        let legacy::DeferredNode {
            depends,
            label,
            query_path,
            subselection,
            node,
        } = value;
        Self {
            depends: into_vec(depends),
            label: label.clone(),
            query_path: query_path
                .0
                .iter()
                .filter_map(|e| match e {
                    // Note: Currently, no type conditioned fetching for deferred queries.
                    PathElement::Key(key, _conditions) => {
                        if key == ".." {
                            // Unexpected parent key
                            None
                        } else {
                            Some(next::QueryPathElement::Field {
                                response_key: Name::new(key).expect("valid key name"),
                            })
                        }
                    }
                    PathElement::Fragment(type_name) => {
                        Some(next::QueryPathElement::InlineFragment {
                            type_condition: Name::new(type_name).expect("valid type name"),
                        })
                    }
                    // Unexpected path keys
                    PathElement::Flatten(_) | PathElement::Index(_) => None,
                })
                .collect(),
            sub_selection: subselection.clone(),
            node: node.as_ref().map(|n| into_box(n.as_ref())),
        }
    }
}

impl From<&'_ legacy::Depends> for next::DeferredDependency {
    fn from(value: &'_ legacy::Depends) -> Self {
        let legacy::Depends { id } = value;
        Self { id: id.clone() }
    }
}

impl From<&'_ legacy::DataRewrite> for next::FetchDataRewrite {
    fn from(value: &'_ legacy::DataRewrite) -> Self {
        match value {
            legacy::DataRewrite::ValueSetter(setter) => Self::ValueSetter(setter.into()),
            legacy::DataRewrite::KeyRenamer(renamer) => Self::KeyRenamer(renamer.into()),
        }
    }
}

impl From<&'_ legacy::DataValueSetter> for next::FetchDataValueSetter {
    fn from(value: &'_ legacy::DataValueSetter) -> Self {
        let legacy::DataValueSetter { path, set_value_to } = value;
        Self {
            path: into_vec(&path.0),
            set_value_to: set_value_to.clone(),
        }
    }
}

impl From<&'_ legacy::DataKeyRenamer> for next::FetchDataKeyRenamer {
    fn from(value: &'_ legacy::DataKeyRenamer) -> Self {
        let legacy::DataKeyRenamer {
            path,
            rename_key_to,
        } = value;
        Self {
            path: into_vec(&path.0),
            rename_key_to: rename_key_to.clone(),
        }
    }
}

impl From<&'_ PathElement> for next::FetchDataPathElement {
    fn from(value: &'_ PathElement) -> Self {
        match value {
            PathElement::Key(name, conditions) => {
                if name == ".." {
                    Self::Parent
                } else {
                    Self::Key(
                        // Note: unchecked due to the empty root key string.
                        Name::new_unchecked(name),
                        from_legacy_type_conditions(conditions),
                    )
                }
            }
            PathElement::Flatten(conditions) => {
                Self::AnyIndex(from_legacy_type_conditions(conditions))
            }
            PathElement::Index(_) => Self::AnyIndex(None),
            PathElement::Fragment(type_name) => {
                Self::TypenameEquals(Name::new(type_name).expect("valid type name"))
            }
        }
    }
}

fn from_legacy_type_conditions(conditions: &Option<Vec<String>>) -> Option<Vec<Name>> {
    conditions.as_ref().map(|conds| {
        conds
            .iter()
            .map(|c| Name::new(c).expect("valid condition name"))
            .collect()
    })
}

impl From<&legacy::Path> for Vec<next::FetchDataPathElement> {
    fn from(value: &legacy::Path) -> Self {
        into_vec(&value.0)
    }
}

fn into_box<T, U>(value: T) -> Box<U>
where
    U: From<T>,
{
    Box::new(value.into())
}

fn into_box_option<'a, T, U>(value: &'a Option<Box<T>>) -> Option<Box<U>>
where
    U: From<&'a T>,
{
    value.as_ref().map(|v| Box::new(v.as_ref().into()))
}

fn into_vec<'a, T, U>(value: &'a [T]) -> Vec<U>
where
    U: From<&'a T>,
{
    value.iter().map(Into::into).collect()
}

fn into_arc_vec_option<'a, T, U>(value: &'a Option<Vec<T>>) -> Vec<Arc<U>>
where
    U: From<&'a T>,
{
    value
        .iter()
        .flat_map(|v| v.iter().map(|i| Arc::new(i.into())))
        .collect()
}
