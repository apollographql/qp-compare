// Copied from `apollo-router/src/query_planner/selection.rs`.

use apollo_compiler::Name;
use serde::Deserialize;
use serde::Serialize;

/// A selection that is part of a fetch.
/// Selections are used to propagate data to subgraph fetches.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", tag = "kind")]
pub(crate) enum Selection {
    /// A field selection.
    Field(Field),

    /// An inline fragment selection.
    InlineFragment(InlineFragment),
}

/// The field that is used
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Field {
    /// An optional alias for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) alias: Option<Name>,

    /// The name of the field.
    pub(crate) name: Name,

    /// The selections for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selections: Option<Vec<Selection>>,
}

impl Field {
    // Mirroring `apollo_compiler::Field::response_name`
    pub(crate) fn response_name(&self) -> &Name {
        self.alias.as_ref().unwrap_or(&self.name)
    }
}

/// An inline fragment.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InlineFragment {
    /// The required fragment type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) type_condition: Option<Name>,

    /// The selections from the fragment.
    pub(crate) selections: Vec<Selection>,
}
