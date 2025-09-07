use crate::{
    models::case::CaseConfig,
    engine::{
        vm::CoreVM,
        lang::{
            ast::{Workflow, Expr, Value, Program, FunctionDef},
            parser::{WorkflowParser, Rule},
            builders::builder_workflow,
        },
    },
};
use pest::Parser;

pub struct CoreEngine {
    vm: CoreVM,
}

impl CoreEngine {
    pub fn new() -> Self {
        let mut vm = CoreVM::new();
        vm.context.env.enter_scope();
        Self { vm }
    }

    pub fn parse_workflow(&self, source: &str) -> Result<Vec<Workflow>, String> {
        let pairs = WorkflowParser::parse(Rule::program, source)
            .map_err(|e| format!("Parse error: {}", e))?;
        
        let workflows = builder_workflow::build_workflows(pairs);
        
        if workflows.is_empty() {
            Err("No workflows found in source".to_string())
        } else {
            Ok(workflows)
        }
    }

    pub fn add_case(&mut self, case: CaseConfig) {
        self.vm.add_case(case);
    }

    pub fn add_cases(&mut self, cases: Vec<CaseConfig>) {
        for case in cases {
            self.vm.add_case(case);
        }
    }

    pub fn execute_workflow(&mut self, workflow: &Workflow) -> Result<(), String> {
        self.vm.execute_workflow(workflow)
    }

    pub fn execute_workflow_from_source(&mut self, source: &str) -> Result<(), String> {
        let workflows = self.parse_workflow(source)?;
        
        if workflows.len() > 1 {
            return Err("Multiple workflows found. Use execute_workflows_from_source() or specify which workflow to execute.".to_string());
        }
        
        self.execute_workflow(&workflows[0])
    }

    pub fn execute_workflows(&mut self, workflows: &[Workflow]) -> Result<(), String> {
        for workflow in workflows {
            self.execute_workflow(workflow)?;
        }
        Ok(())
    }

    pub fn execute_workflows_from_source(&mut self, source: &str) -> Result<(), String> {
        let workflows = self.parse_workflow(source)?;
        self.execute_workflows(&workflows)
    }

    pub fn parse_program(&self, source: &str) -> Result<Program, String> {
        let pairs = WorkflowParser::parse(Rule::program, source)
            .map_err(|e| format!("Parse error: {}", e))?;
        
        let program = builder_workflow::build_program(pairs);
        Ok(program)
    }

    pub fn execute_program(&mut self, program: &Program) -> Result<(), String> {
        self.vm.execute_program(program)
    }

    pub fn execute_program_from_source(&mut self, source: &str) -> Result<(), String> {
        let program = self.parse_program(source)?;
        self.execute_program(&program)
    }

    pub fn register_function(&mut self, function: FunctionDef) {
        self.vm.register_function(function);
    }

    pub fn register_functions(&mut self, functions: Vec<FunctionDef>) {
        self.vm.register_functions(functions);
    }

    pub fn get_user_function_names(&self) -> Vec<String> {
        self.vm.get_user_function_names()
    }

    pub fn get_cases(&self) -> &[CaseConfig] {
        self.vm.get_cases()
    }

    pub fn get_cases_copy(&self) -> Vec<CaseConfig> {
        self.vm.get_cases().to_vec()
    }

    pub fn clear_cases(&mut self) {
        self.vm.clear_cases();
    }

    pub fn case_count(&self) -> usize {
        self.vm.get_cases().len()
    }

    pub fn has_cases(&self) -> bool {
        !self.vm.get_cases().is_empty()
    }

    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, String> {
        self.vm.evaluate_expr(expr)
    }

    pub fn evaluate_expression_from_string(&mut self, expr_str: &str) -> Result<Value, String> {
        let full_source = format!("workflow temp {{ score {{ when {} then score = 1 }} }}", expr_str);
        let pairs = WorkflowParser::parse(Rule::program, &full_source)
            .map_err(|e| format!("Expression parse error: {}", e))?;
        
        let workflows = builder_workflow::build_workflows(pairs);
        if workflows.is_empty() || workflows[0].phases.is_empty() {
            return Err("Failed to parse expression".to_string());
        }
        
        if let crate::engine::lang::ast::Phase::Score(rules) = &workflows[0].phases[0] {
            if !rules.is_empty() {
                return self.vm.evaluate_expr(&rules[0].condition);
            }
        }
        
        Err("Failed to extract expression".to_string())
    }

    pub fn get_variable(&self, name: &str) -> Option<Value> {
        self.vm.context.env.lookup(name).cloned()
    }

    pub fn set_variable(&mut self, name: impl Into<String>, value: Value) {
        self.vm.context.env.insert(name, value);
    }

    pub fn get_variable_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for scope in &self.vm.context.env.env {
            for key in scope.keys() {
                if !names.contains(key) {
                    names.push(key.clone());
                }
            }
        }
        names.sort();
        names
    }

    pub fn enter_scope(&mut self) {
        self.vm.context.env.enter_scope();
    }

    pub fn exit_scope(&mut self) {
        self.vm.context.env.exit_scope();
    }

    pub fn reset(&mut self) {
        self.vm.clear_cases();
        self.vm.context.env = crate::engine::vm::environment::Environment::new();
    }

    pub fn get_stats(&self) -> EngineStats {
        let cases = self.get_cases();
        let total_score: i64 = cases.iter().map(|c| c.score).sum();
        let avg_score = if cases.is_empty() { 0.0 } else { total_score as f64 / cases.len() as f64 };
        
        let max_score = cases.iter().map(|c| c.score).max().unwrap_or(0);
        let min_score = cases.iter().map(|c| c.score).min().unwrap_or(0);
        
        EngineStats {
            case_count: cases.len(),
            total_score,
            average_score: avg_score,
            max_score,
            min_score,
            variable_count: self.get_variable_names().len(),
        }
    }

    pub fn score_cases<F>(&mut self, scoring_fn: F) -> Result<(), String>
    where
        F: Fn(&CaseConfig) -> i64,
    {
        let cases = self.vm.context.stack.cases.clone();
        let mut processed_cases = Vec::new();
        
        for mut case in cases {
            case.score = scoring_fn(&case);
            processed_cases.push(case);
        }
        
        self.vm.context.stack.cases = processed_cases;
        Ok(())
    }

    pub fn filter_cases<F>(&mut self, predicate: F)
    where
        F: Fn(&CaseConfig) -> bool,
    {
        self.vm.context.stack.cases.retain(predicate);
    }

    pub fn sort_cases_by<F, K>(&mut self, key_fn: F)
    where
        F: Fn(&CaseConfig) -> K,
        K: Ord,
    {
        self.vm.context.stack.cases.sort_by_key(key_fn);
    }

    pub fn sort_cases_by_score_desc(&mut self) {
        self.vm.context.stack.cases.sort_by(|a, b| b.score.cmp(&a.score));
    }

    pub fn sort_cases_by_score_asc(&mut self) {
        self.vm.context.stack.cases.sort_by(|a, b| a.score.cmp(&b.score));
    }

    pub fn get_high_score_cases(&self, threshold: i64) -> Vec<&CaseConfig> {
        self.get_cases().iter().filter(|c| c.score > threshold).collect()
    }

    pub fn get_low_score_cases(&self, threshold: i64) -> Vec<&CaseConfig> {
        self.get_cases().iter().filter(|c| c.score < threshold).collect()
    }

    pub fn get_cases_by_category(&self, category: &str) -> Vec<&CaseConfig> {
        self.get_cases().iter().filter(|c| c.category == category).collect()
    }

    pub fn get_cases_by_status(&self, status: &str) -> Vec<&CaseConfig> {
        self.get_cases().iter().filter(|c| c.status == status).collect()
    }

    pub fn run(&mut self) -> Result<Vec<CaseConfig>, String> {
        Ok(self.get_cases_copy())
    }
}

impl Default for CoreEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub case_count: usize,
    pub total_score: i64,
    pub average_score: f64,
    pub max_score: i64,
    pub min_score: i64,
    pub variable_count: usize,
}
