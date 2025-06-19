use crate::action::Action;
use crate::data_structure::{
    StateMachineMergeMode, StateMachineState, StateMachineTransition, StateMachineTrigger, StateMap,
};
use color_eyre::eyre::eyre;

#[derive(Debug)]
pub struct StateMachine {
    pub(crate) states: StateMap,
}

impl Default for StateMachine {
    fn default() -> Self {
        let mut s = Self {
            states: StateMap::new(),
        };

        s.states.insert(
            "Init".to_string(),
            StateMachineState {
                merge: Default::default(),
                transitions: vec![StateMachineTransition {
                    target: "SwitchDetect".to_string(),
                    trigger: StateMachineTrigger::Immediate,
                    actions: vec![Action::SetupJob],
                }],
            },
        );

        s.states.insert(
            "SwitchDetect".to_string(),
            StateMachineState {
                merge: Default::default(),
                transitions: vec![StateMachineTransition {
                    target: "SwitchDetect".to_string(),
                    trigger: StateMachineTrigger::String {
                        string: "A non-empty Data Buffering File was found.".to_string(),
                    },
                    actions: vec![Action::SendLine {
                        line: "E".to_string(),
                    }],
                }],
            },
        );

        s.states.insert(
            "EndJob".to_string(),
            StateMachineState {
                merge: Default::default(),
                transitions: vec![StateMachineTransition {
                    target: "JobFinished".to_string(),
                    trigger: StateMachineTrigger::Immediate,
                    actions: vec![Action::FinishJob],
                }],
            },
        );

        s.states.insert(
            "JobFinished".to_string(),
            StateMachineState {
                merge: Default::default(),
                transitions: vec![StateMachineTransition {
                    target: "JobFinished".to_string(),
                    trigger: StateMachineTrigger::String {
                        string: "AAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                    },
                    actions: vec![],
                }],
            },
        );

        s
    }
}

impl StateMachine {
    pub fn merge_states(&mut self, states: StateMap) {
        for (key, value) in states {
            if let Some(v) = self.states.get_mut(&key) {
                match value.merge {
                    StateMachineMergeMode::Replace => {
                        *v = value;
                    }
                    StateMachineMergeMode::Append => {
                        v.transitions.extend(value.transitions);
                    }
                }
            } else {
                self.states.insert(key, value);
            }
        }
    }
    
    pub fn states(&self) -> Vec<String> {
        self.states.keys().cloned().collect::<Vec<String>>()
    }

    pub fn get_state(&self, key: &str) -> Option<StateMachineState> {
        self.states.get(key).cloned()
    }

    pub fn state(&self, key: &str) -> color_eyre::Result<StateMachineState> {
        self.states
            .get(key)
            .cloned()
            .ok_or_else(|| eyre!("unknown state: {}", key))
    }
}
