use crate::data_structure::StateMachineTrigger;
use regex::Regex;
use swexpect::hay::ReadUntil;

impl StateMachineTrigger {
    pub fn to_needle(&self) -> color_eyre::Result<Option<ReadUntil>> {
        match self {
            StateMachineTrigger::String { string: s } => Ok(Some(ReadUntil::String(s.clone()))),
            StateMachineTrigger::Regex { regex: s } => Ok(Some(ReadUntil::Regex(Regex::new(s)?))),
            StateMachineTrigger::Immediate => Ok(None),
        }
    }

    pub fn matches_result(&self, m: &str) -> color_eyre::Result<bool> {
        match self {
            StateMachineTrigger::String { string: s } => Ok(m == s),
            StateMachineTrigger::Regex { regex: s } => {
                let r = Regex::new(s)?;
                Ok(r.is_match(m))
            }
            StateMachineTrigger::Immediate => Ok(true),
        }
    }
}
