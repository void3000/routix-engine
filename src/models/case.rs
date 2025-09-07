#[derive(Debug, Clone)]
pub struct CaseConfig {
    pub id: i32,
    pub category: String,
    pub status: String,
    pub priority: i32,
    pub customer: Option<String>,
    pub score: i64,
}
