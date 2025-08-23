use std::collections::HashMap;
use serde_json::Value;

pub fn evaluate_condition(condition: &str, dependencies: &HashMap<String, Value>) -> bool {
    // This is a very simple evaluator for demonstration purposes.
    // It can be replaced with a more robust expression language engine if needed.

    // First, replace placeholders like `result` with actual values
    let mut replaced_condition = condition.to_string();
    if let Some(result_value) = dependencies.values().next() {
        // Convert the serde_json::Value to a string for simple evaluation
        let result_str = result_value.to_string();
        replaced_condition = replaced_condition.replace("result", &format!("'{}'", result_str.trim_matches('"'))); // Trim quotes if it was a JSON string
    }

    if replaced_condition.contains("==") {
        let parts: Vec<&str> = replaced_condition.split("==").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return parts[0].trim_matches('\'') == parts[1].trim_matches('\'');
        }
    } else if replaced_condition.contains("!=") {
        let parts: Vec<&str> = replaced_condition.split("!=").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return parts[0].trim_matches('\'') != parts[1].trim_matches('\'');
        }
    }

    // Default to true if the condition is malformed or not an equality check
    true
}
