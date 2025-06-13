use crate::action::Action;
use cthulhu_common::stages::ProcessStage;
use regex::Regex;
use swexpect::hay::ReadUntil;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum StateCondition {
    WaitForString(String),
    WaitForRegex(String),
    Immediate,
}

impl StateCondition {
    pub fn to_needle(&self) -> color_eyre::Result<Option<ReadUntil>> {
        match self {
            StateCondition::WaitForString(s) => Ok(Some(ReadUntil::String(s.clone()))),
            StateCondition::WaitForRegex(s) => Ok(Some(ReadUntil::Regex(Regex::new(s)?))),
            StateCondition::Immediate => Ok(None),
        }
    }

    pub fn matches_result(&self, m: &str) -> color_eyre::Result<bool> {
        match self {
            StateCondition::WaitForString(s) => Ok(m == s),
            StateCondition::WaitForRegex(s) => {
                let r = Regex::new(s)?;
                Ok(r.is_match(m))
            }
            StateCondition::Immediate => Ok(true),
        }
    }
}

pub struct StateTransition {
    pub target_state: ProcessStage,
    pub actions: Vec<Action>,
    pub condition: StateCondition,
}
