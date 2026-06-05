use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Comparison operators for condition checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Exists,
    NotExists,
}

/// A single atomic condition: compare a game state key using an operator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionCheck {
    pub key: String,
    pub operator: ConditionOperator,
    /// Required for Equals/NotEquals; ignored for Exists/NotExists.
    #[serde(default)]
    pub value: Option<String>,
}

impl ConditionCheck {
    /// Evaluates this check against a set of game state flags.
    pub fn evaluate(&self, flags: &HashMap<String, String>) -> bool {
        match self.operator {
            ConditionOperator::Equals => match &self.value {
                Some(v) => flags.get(&self.key) == Some(v),
                None => false,
            },
            ConditionOperator::NotEquals => match &self.value {
                Some(v) => flags.get(&self.key) != Some(v),
                None => true,
            },
            ConditionOperator::Exists => flags.contains_key(&self.key),
            ConditionOperator::NotExists => !flags.contains_key(&self.key),
        }
    }
}

/// How multiple checks are combined.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionLogic {
    #[default]
    All,
    Any,
}

/// A compound condition: multiple checks combined with AND/OR logic.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BranchCondition {
    #[serde(default)]
    pub logic: ConditionLogic,
    #[serde(default)]
    pub checks: Vec<ConditionCheck>,
}

impl BranchCondition {
    /// Evaluates this condition against a set of game state flags.
    ///
    /// - `All` logic: returns true iff every check passes (empty = true).
    /// - `Any` logic: returns true iff at least one check passes (empty = true).
    pub fn evaluate(&self, flags: &HashMap<String, String>) -> bool {
        if self.checks.is_empty() {
            return true;
        }
        match self.logic {
            ConditionLogic::All => self.checks.iter().all(|c| c.evaluate(flags)),
            ConditionLogic::Any => self.checks.iter().any(|c| c.evaluate(flags)),
        }
    }
}

/// A condition-gated event trigger sequence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionalTrigger {
    pub condition: BranchCondition,
    pub actions: Vec<crate::map::EventAction>,
}
