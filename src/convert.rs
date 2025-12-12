// Convert the router QueryPlan into the native QueryPlan from apollo-federation
use std::sync::Arc;

use apollo_compiler::Name;
use apollo_federation::query_plan as next;

use crate::router::path;
use crate::router::plan;

pub(crate) fn convert_root_query_plan_node(legacy: &plan::QueryPlan) -> next::QueryPlan {
    let plan::QueryPlan { node } = legacy;
    next::QueryPlan {
        node: node.as_ref().map(|n| n.as_ref().into()),
        statistics: Default::default(),
    }
}

impl From<&'_ plan::PlanNode> for next::TopLevelPlanNode {
    fn from(value: &'_ plan::PlanNode) -> Self {
        match value {
            plan::PlanNode::Subscription { primary, rest } => {
                Self::Subscription(next::SubscriptionNode {
                    primary: Box::new(primary.into()),
                    rest: rest.as_ref().map(|r| {
                        let node: next::PlanNode = r.as_ref().into();
                        Box::new(node)
                    }),
                })
            }
            plan::PlanNode::Fetch(node) => Self::Fetch(Box::new(node.into())),
            plan::PlanNode::Sequence { nodes } => Self::Sequence(next::SequenceNode {
                nodes: nodes.iter().map(Into::into).collect(),
            }),
            plan::PlanNode::Parallel { nodes } => Self::Parallel(next::ParallelNode {
                nodes: nodes.iter().map(Into::into).collect(),
            }),
            plan::PlanNode::Flatten(node) => Self::Flatten(node.into()),
            plan::PlanNode::Defer { primary, deferred } => Self::Defer(next::DeferNode {
                primary: primary.into(),
                deferred: deferred.iter().map(Into::into).collect(),
            }),
            plan::PlanNode::Condition {
                condition,
                if_clause,
                else_clause,
            } => Self::Condition(Box::new(next::ConditionNode {
                condition_variable: Name::new(condition).expect("valid condition variable name"),
                if_clause: if_clause.as_ref().map(|c| {
                    let node: next::PlanNode = c.as_ref().into();
                    Box::new(node)
                }),
                else_clause: else_clause.as_ref().map(|c| {
                    let node: next::PlanNode = c.as_ref().into();
                    Box::new(node)
                }),
            })),
        }
    }
}

impl From<&'_ plan::PlanNode> for next::PlanNode {
    fn from(value: &'_ plan::PlanNode) -> Self {
        match value {
            plan::PlanNode::Fetch(node) => Self::Fetch(Box::new(node.into())),
            plan::PlanNode::Sequence { nodes } => Self::Sequence(next::SequenceNode {
                nodes: nodes.iter().map(Into::into).collect(),
            }),
            plan::PlanNode::Parallel { nodes } => Self::Parallel(next::ParallelNode {
                nodes: nodes.iter().map(Into::into).collect(),
            }),
            plan::PlanNode::Flatten(node) => Self::Flatten(node.into()),
            plan::PlanNode::Defer { primary, deferred } => Self::Defer(next::DeferNode {
                primary: primary.into(),
                deferred: deferred.iter().map(Into::into).collect(),
            }),
            plan::PlanNode::Condition {
                condition,
                if_clause,
                else_clause,
            } => Self::Condition(Box::new(next::ConditionNode {
                condition_variable: Name::new(condition).expect("valid condition variable name"),
                if_clause: if_clause.as_ref().map(|c| {
                    let node: next::PlanNode = c.as_ref().into();
                    Box::new(node)
                }),
                else_clause: else_clause.as_ref().map(|c| {
                    let node: next::PlanNode = c.as_ref().into();
                    Box::new(node)
                }),
            })),
            plan::PlanNode::Subscription { .. } => {
                panic!("Subscription nodes should only appear at the top level")
            }
        }
    }
}

impl From<&'_ plan::FetchNode> for next::FetchNode {
    fn from(value: &'_ plan::FetchNode) -> Self {
        let plan::FetchNode {
            service_name,
            requires,
            variable_usages,
            operation,
            operation_name,
            operation_kind,
            id,
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
            input_rewrites: Arc::new(
                input_rewrites
                    .as_ref()
                    .map(|v| v.iter().map(|r| Arc::new(r.into())).collect::<Vec<_>>())
                    .unwrap_or_default(),
            ),
            output_rewrites: output_rewrites
                .as_ref()
                .map(|v| v.iter().map(|r| Arc::new(r.into())).collect::<Vec<_>>())
                .unwrap_or_default(),
            context_rewrites: context_rewrites
                .as_ref()
                .map(|v| v.iter().map(|r| Arc::new(r.into())).collect::<Vec<_>>())
                .unwrap_or_default(),
        }
    }
}

impl From<&'_ plan::FlattenNode> for next::FlattenNode {
    fn from(value: &'_ plan::FlattenNode) -> Self {
        let plan::FlattenNode { path, node } = value;
        Self {
            path: path.0.iter().map(Into::into).collect(),
            node: Box::new(node.as_ref().into()),
        }
    }
}

impl From<&'_ plan::SubscriptionNode> for next::FetchNode {
    fn from(value: &'_ plan::SubscriptionNode) -> Self {
        let plan::SubscriptionNode {
            service_name,
            variable_usages,
            operation,
            operation_name,
            operation_kind,
            input_rewrites,
            output_rewrites,
        } = value;
        Self {
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
            input_rewrites: Arc::new(
                input_rewrites
                    .as_ref()
                    .map(|v| v.iter().map(|r| Arc::new(r.into())).collect::<Vec<_>>())
                    .unwrap_or_default(),
            ),
            output_rewrites: output_rewrites
                .as_ref()
                .map(|v| v.iter().map(|r| Arc::new(r.into())).collect::<Vec<_>>())
                .unwrap_or_default(),
            context_rewrites: vec![],
        }
    }
}

impl From<&'_ plan::Primary> for next::PrimaryDeferBlock {
    fn from(value: &'_ plan::Primary) -> Self {
        let plan::Primary { node, subselection } = value;
        Self {
            sub_selection: subselection.clone(),
            node: node.as_ref().map(|n| {
                let plan_node: next::PlanNode = n.as_ref().into();
                Box::new(plan_node)
            }),
        }
    }
}

impl From<&'_ plan::DeferredNode> for next::DeferredDeferBlock {
    fn from(value: &'_ plan::DeferredNode) -> Self {
        let plan::DeferredNode {
            depends,
            label,
            query_path,
            subselection,
            node,
        } = value;
        Self {
            depends: depends.iter().map(Into::into).collect(),
            label: label.clone(),
            query_path: query_path
                .0
                .iter()
                .filter_map(|e| match e {
                    path::PathElement::Key(key, _conditions) => {
                        if key == ".." {
                            None
                        } else {
                            Some(next::QueryPathElement::Field {
                                response_key: Name::new(key).expect("valid key name"),
                            })
                        }
                    }
                    path::PathElement::Fragment(type_name) => {
                        Some(next::QueryPathElement::InlineFragment {
                            type_condition: Name::new(type_name).expect("valid type name"),
                        })
                    }
                    path::PathElement::Flatten(_) | path::PathElement::Index(_) => None,
                })
                .collect(),
            sub_selection: subselection.clone(),
            node: node.as_ref().map(|n| {
                let plan_node: next::PlanNode = n.as_ref().into();
                Box::new(plan_node)
            }),
        }
    }
}

impl From<&'_ plan::Depends> for next::DeferredDependency {
    fn from(value: &'_ plan::Depends) -> Self {
        let plan::Depends { id } = value;
        Self { id: id.clone() }
    }
}

impl From<&'_ plan::DataRewrite> for next::FetchDataRewrite {
    fn from(value: &'_ plan::DataRewrite) -> Self {
        match value {
            plan::DataRewrite::ValueSetter(setter) => Self::ValueSetter(setter.into()),
            plan::DataRewrite::KeyRenamer(renamer) => Self::KeyRenamer(renamer.into()),
        }
    }
}

impl From<&'_ plan::DataValueSetter> for next::FetchDataValueSetter {
    fn from(value: &'_ plan::DataValueSetter) -> Self {
        let plan::DataValueSetter { path, set_value_to } = value;
        Self {
            path: path.0.iter().map(Into::into).collect(),
            set_value_to: set_value_to.clone(),
        }
    }
}

impl From<&'_ plan::DataKeyRenamer> for next::FetchDataKeyRenamer {
    fn from(value: &'_ plan::DataKeyRenamer) -> Self {
        let plan::DataKeyRenamer {
            path,
            rename_key_to,
        } = value;
        Self {
            path: path.0.iter().map(Into::into).collect(),
            rename_key_to: rename_key_to.clone(),
        }
    }
}

impl From<&'_ path::PathElement> for next::FetchDataPathElement {
    fn from(value: &'_ path::PathElement) -> Self {
        match value {
            path::PathElement::Key(name, conditions) => {
                if name == ".." {
                    Self::Parent
                } else {
                    Self::Key(
                        // TODO: unchecked due to the empty root key string.
                        Name::new_unchecked(name),
                        conditions.as_ref().map(|c| {
                            c.iter()
                                .map(|s| Name::new(s).expect("valid condition name"))
                                .collect()
                        }),
                    )
                }
            }
            path::PathElement::Flatten(conditions) => {
                Self::AnyIndex(conditions.as_ref().map(|c| {
                    c.iter()
                        .map(|s| Name::new(s).expect("valid condition name"))
                        .collect()
                }))
            }
            path::PathElement::Index(_) => Self::AnyIndex(None),
            path::PathElement::Fragment(type_name) => {
                Self::TypenameEquals(Name::new(type_name).expect("valid type name"))
            }
        }
    }
}

impl From<&path::Path> for Vec<next::FetchDataPathElement> {
    fn from(value: &path::Path) -> Self {
        value.0.iter().map(Into::into).collect()
    }
}
