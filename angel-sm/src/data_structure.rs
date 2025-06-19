use crate::action::Action;
use crate::util::vec_or_single;
use serde::Deserialize;
use std::collections::BTreeMap;

pub type State = String;
pub type StateMap = BTreeMap<State, StateMachineState>;

#[derive(Deserialize, Clone, Debug, PartialOrd, PartialEq)]
pub struct StateMachineFile {
    pub id: String,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(rename = "state")]
    pub states: StateMap,
}

#[derive(Deserialize, Clone, Debug, PartialOrd, PartialEq)]
pub struct StateMachineState {
    #[serde(default)]
    pub merge: StateMachineMergeMode,
    #[serde(rename = "transition", deserialize_with = "vec_or_single")]
    pub transitions: Vec<StateMachineTransition>,
}

#[derive(Deserialize, Clone, Debug, Default, PartialOrd, PartialEq)]
pub enum StateMachineMergeMode {
    #[default]
    #[serde(rename = "replace")]
    Replace,
    #[serde(rename = "append")]
    Append,
}

#[derive(Deserialize, Clone, Debug, PartialOrd, PartialEq)]
pub struct StateMachineTransition {
    pub target: State,
    pub trigger: StateMachineTrigger,
    #[serde(rename = "action", default, deserialize_with = "vec_or_single")]
    pub actions: Vec<Action>,
}

#[derive(Deserialize, Clone, Debug, Eq, PartialOrd, PartialEq, Ord, Hash)]
#[serde(tag = "type")]
pub enum StateMachineTrigger {
    #[serde(rename = "string")]
    String { string: String },
    #[serde(rename = "regex")]
    Regex { regex: String },
    #[serde(rename = "immediate")]
    Immediate,
}
