use super::task_runner::TaskRunner;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TaskRegistry {
    runners: HashMap<String, Arc<dyn TaskRunner>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
        }
    }

    pub fn register(&mut self, runner: Arc<dyn TaskRunner>) {
        self.runners.insert(runner.name(), runner);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn TaskRunner>> {
        self.runners.get(name).cloned()
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}
