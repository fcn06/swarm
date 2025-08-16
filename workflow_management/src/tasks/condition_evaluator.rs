use std::collections::HashMap;

pub fn evaluate_condition(condition: &str, dependencies: &HashMap<String, String>) -> bool {
    // This is a very simple evaluator for demonstration purposes.
    // It can be replaced with a more robust expression language engine if needed.

    // First, replace placeholders like `result` with actual values
    let mut replaced_condition = condition.to_string();
    if let Some(result) = dependencies.values().next() {
        replaced_condition = replaced_condition.replace("result", &format!("'{}'", result));
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
