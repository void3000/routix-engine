use crate::{
    engine::{
        vm::{
            context::VmContext,
            evaluators::{
                expr_evaluator::ExprEvaluator,
                workflow_evaluator::WorkflowEvaluator,
                action_evaluator::ActionEvaluator,
                builtin_functions::BuiltinFunctions,
            },
        },
        lang::ast::{Workflow, Expr, Value, FunctionDef, Program},
    },
    models::case::CaseConfig,
};


pub struct CoreVM {
    pub context: VmContext,
}

impl CoreVM {
    pub fn new() -> Self {
        let mut vm = Self { 
            context: VmContext::default(),
        };
        // Initialize with a global scope for built-in functions
        vm.context.env.enter_scope();
        // Register built-in functions in the environment
        let builtin_functions = BuiltinFunctions::register_all();
        for (name, func) in builtin_functions {
            vm.context.env.insert(name, Value::BuiltinFunction(func));
        }
        vm
    }

    pub fn run(&mut self) -> Result<Vec<CaseConfig>, String> {
        Ok(self.context.stack.cases.clone())
    }

    /// Execute a workflow on the current cases in the stack
    pub fn execute_workflow(&mut self, workflow: &Workflow) -> Result<(), String> {
        // Clone the cases to avoid borrowing issues
        let cases = self.context.stack.cases.clone();
        
        // Use the workflow evaluator
        let processed_cases = WorkflowEvaluator::execute_workflow(
            &mut self.context,
            workflow,
            cases,
        )?;
        
        // Update the stack with processed cases
        self.context.stack.cases = processed_cases;
        Ok(())
    }

    /// Set up the case data in the environment for evaluation
    pub fn setup_case_context(&mut self, case: &CaseConfig) -> Result<(), String> {
        WorkflowEvaluator::setup_case_context(&mut self.context, case)
    }

    /// Execute a score phase
    pub fn execute_score_phase(&mut self, rules: &[crate::engine::lang::ast::Rule], case: &mut CaseConfig) -> Result<(), String> {
        WorkflowEvaluator::execute_score_phase(&mut self.context, rules, case)
    }

    /// Execute a match phase
    pub fn execute_match_phase(&mut self, rules: &[crate::engine::lang::ast::MatchRule], case: &mut CaseConfig) -> Result<(), String> {
        WorkflowEvaluator::execute_match_phase(&mut self.context, rules, case)
    }

    /// Execute an action
    pub fn execute_action(&mut self, action: &crate::engine::lang::ast::Action, case: &mut CaseConfig) -> Result<(), String> {
        ActionEvaluator::execute_action(&mut self.context, action, case)
    }

    /// Evaluate an expression
    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        ExprEvaluator::evaluate_expr(&mut self.context, expr)
    }

    /// Register a user-defined function
    pub fn register_function(&mut self, function: FunctionDef) {
        let name = function.name.clone();
        self.context.env.insert(name, Value::UserFunction(function));
    }

    /// Register multiple user-defined functions
    pub fn register_functions(&mut self, functions: Vec<FunctionDef>) {
        for function in functions {
            self.register_function(function);
        }
    }

    /// Execute a program (functions + workflows)
    pub fn execute_program(&mut self, program: &Program) -> Result<(), String> {
        // Register user-defined functions first
        self.register_functions(program.functions.clone());
        
        // Execute all workflows
        for workflow in &program.workflows {
            self.execute_workflow(workflow)?;
        }
        
        Ok(())
    }

    /// Get all function names (both built-in and user-defined)
    pub fn get_function_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for scope in &self.context.env.env {
            for (key, value) in scope {
                match value {
                    Value::BuiltinFunction(_) | Value::UserFunction(_) => {
                        if !names.contains(key) {
                            names.push(key.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        names.sort();
        names
    }

    /// Get user-defined function names only
    pub fn get_user_function_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for scope in &self.context.env.env {
            for (key, value) in scope {
                if matches!(value, Value::UserFunction(_)) {
                    if !names.contains(key) {
                        names.push(key.clone());
                    }
                }
            }
        }
        names.sort();
        names
    }



    /// Add a case to the stack for processing
    pub fn add_case(&mut self, case: CaseConfig) {
        self.context.stack.push_case(case);
    }

    /// Get all processed cases
    pub fn get_cases(&self) -> &[CaseConfig] {
        &self.context.stack.cases
    }

    /// Clear all cases from the stack
    pub fn clear_cases(&mut self) {
        self.context.stack.cases.clear();
    }
}

pub trait CoreEval {
    fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String>;
    fn execute_workflow(&mut self, workflow: &Workflow) -> Result<(), String>;
}

impl CoreEval for CoreVM {
    fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        self.evaluate_expr(expr)
    }

    fn execute_workflow(&mut self, workflow: &Workflow) -> Result<(), String> {
        self.execute_workflow(workflow)
    }
}
