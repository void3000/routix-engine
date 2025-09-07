use crate::models::case::CaseConfig;

#[derive(Debug)]
pub struct WorkflowResult {
    pub routed: Vec<CaseConfig>,
    pub logs: Vec<String>,
}
