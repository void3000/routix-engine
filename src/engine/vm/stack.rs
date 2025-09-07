use crate::models::{ agent::AgentConfig, case::CaseConfig };

#[derive(Debug, Default)]
pub struct VmStack {
    pub agent: Option<AgentConfig>,
    pub cases: Vec<CaseConfig>,
}

impl VmStack {
    pub fn new(agent: Option<AgentConfig>, cases: Vec<CaseConfig>) -> Self {
        VmStack { agent, cases }
    }

    pub fn set_agent(&mut self, agent: AgentConfig) {
        self.agent = Some(agent);
    }

    pub fn push_case(&mut self, case: CaseConfig) {
        self.cases.push(case);
    }

    pub fn pop_case(&mut self) -> Option<CaseConfig> {
        self.cases.pop()
    }

    pub fn peek_case(&self) -> Option<&CaseConfig> {
        self.cases.last()
    }

    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cases.len()
    }
}
