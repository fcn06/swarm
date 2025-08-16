use super::agent_runner::AgentRunner;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AgentRegistry {
    runners: HashMap<String, Arc<dyn AgentRunner>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
        }
    }

    pub fn register(&mut self, runner: Arc<dyn AgentRunner>) {
        self.runners.insert(runner.name(), runner);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn AgentRunner>> {
        self.runners.get(name).cloned()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
