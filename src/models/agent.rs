
#[derive(Debug)]
pub struct AgentConfig {
    pub id: String,
    pub skills: Skills,
    pub max_concurrent: u32,
}

#[derive(Debug)]
pub struct Skills {
    pub languages: Vec<String>,
    pub services: Vec<String>,
    pub platforms: Vec<String>,
}
