use anyhow::{Context, Result};
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
    rmcp_tools
        .into_iter()
        .map(|tool| {
            let tool_name = tool.name.to_string(); // Get name early for potential error context
            let description = tool
                .description
                .ok_or_else(|| {
                    anyhow::anyhow!("Tool description is missing for tool '{}'", tool_name)
                })?
                .to_string(); // Convert Arc<str> to String

            // Clone the input schema map directly
            let properties_map: Map<String, Value> = tool.input_schema.as_ref().clone();

            let properties = properties_map.get("properties");
            //println!("Properties : {:#?}", properties);

            Ok(Tool {
                r#type: "function".to_string(),
                function: FunctionDefinition {
                    name: tool_name, // Use owned name
                    description,
                    parameters: FunctionParameters {
                        r#type: "object".to_string(),
                        properties: properties
                            .cloned()
                            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
                        required: None, // Keep as None for now
                    },
                },
            })
        })
        .collect::<Result<Vec<Tool>>>()
        .with_context(|| "Failed to define tools from rmcp::model::Tool vector")
}
