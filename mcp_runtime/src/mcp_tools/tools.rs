use anyhow::Result;
use rmcp::model::Tool as RmcpTool; // Alias for clarity

use llm_api::tools::{FunctionDefinition, FunctionParameters, Tool};
use serde_json::{Map, Value};

/// Converts a vector of `rmcp::model::Tool` into a vector of locally defined `Tool` structs,
/// suitable for use with LLM APIs expecting this format.
///
/// Returns an empty vector if the input `tools` vector is empty.
/// Returns an error if any tool is missing a description.
///
/// # Arguments
///
/// * `rmcp_tools` - A vector of `rmcp::model::Tool` structs to convert.
///
/// # Returns
///
/// * Result<Vec<Tool>>` - A result containing the vector of converted `Tool` structs
///   or an error if the conversion fails for any tool.
///
/// # Note
/// Currently, the `required` field in `FunctionParameters` is always set to `None`.
/// Future improvements could involve parsing the `input_schema` to determine required parameters.
pub fn define_all_tools(rmcp_tools: Vec<RmcpTool>) -> Result<Vec<Tool>> {
    let mut tools = Vec::new();

    for tool in rmcp_tools {
        let tool_name = tool.name.to_string();
        if tool_name.is_empty() {
            continue;
        }

        let description = tool
            .description
            .as_ref()
            .map(|d| d.to_string())
            .unwrap_or_else(|| format!("Execute {}", tool_name));

        // Extract properties and required fields safely
        let properties_map: Map<String, Value> = tool.input_schema.as_ref().clone();

        let properties = properties_map
            .get("properties")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));

        let required = properties_map
            .get("required")
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok());

        tools.push(Tool {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: tool_name,
                description,
                parameters: FunctionParameters {
                    r#type: "object".to_string(),
                    properties,
                    required,
                },
            },
        });
    }

    Ok(tools)
}
