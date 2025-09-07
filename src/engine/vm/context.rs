use crate::engine::vm::{ stack::VmStack, environment::Environment };

#[derive(Default)]
pub struct VmContext {
    pub stack: VmStack,
    pub env: Environment,
}

impl VmContext {
    pub fn new(stack: VmStack, env: Environment) -> Self {
        Self { stack, env }
    }

    pub fn default() -> Self 
    where
        VmStack: Default,
        Environment: Default,
    {
        Self {
            stack: VmStack::default(),
            env: Environment::default(),
        }
    }

    pub fn stack(&self) -> &VmStack {
        &self.stack
    }

    pub fn stack_mut(&mut self) -> &mut VmStack {
        &mut self.stack
    }

    pub fn env(&self) -> &Environment {
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.env
    }

    pub fn replace_stack(&mut self, new_stack: VmStack) -> VmStack {
        std::mem::replace(&mut self.stack, new_stack)
    }

    pub fn replace_env(&mut self, new_env: Environment) -> Environment {
        std::mem::replace(&mut self.env, new_env)
    }
}
