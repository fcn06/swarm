// tools.rs

use anyhow::{Context, Result};
use rmcp::model::Tool as RmcpTool; // Alias for clarity
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Represents a tool definition that can be passed to the LLM.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    /// The type of the tool, typically "function".
    pub r#type: String,
    /// The function definition.
    pub function: FunctionDefinition,
}

/// Defines the structure of a function tool.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionDefinition {
    /// The name of the function.
    pub name: String,
    /// A description of what the function does.
    pub description: String,
    /// The parameters the function accepts, described as a JSON Schema object.
    pub parameters: FunctionParameters,
}

/// Represents the parameters of a function tool using JSON Schema.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionParameters {
    /// The type of the parameters object, typically "object".
    pub r#type: String,
    /// A map describing the properties of the parameters object.
    /// The keys are parameter names, and values are JSON Schema descriptions.
    /// Example: `serde_json::json!({{"location": {{"type": "string", "description": "City name"}}}})`
    pub properties: Value,
    /// An optional list of parameter names that are required.
    /// Example: `Some(vec!["location".to_string()])`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// Represents the result of a tool call to be sent back to the LLM.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolResult {
    /// The ID of the tool call this result corresponds to.
    pub tool_call_id: String,
    /// The output/result of the tool execution as a string.
    pub output: String,
}

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
/// Todo : Move this function outside of llm_api crate. It should be in a crate defining mcp agent interaction
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

            // The tool.input_schema is already the JSON schema for parameters
            let parameters_schema = tool.input_schema; // This is Map<String, Value>

            let param_type = parameters_schema
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("object")
                .to_string();

            let properties = parameters_schema
                .get("properties")
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new()));

            let required = parameters_schema
                .get("required")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).map(String::from).collect());


            Ok(Tool {
                r#type: "function".to_string(),
                function: FunctionDefinition {
                    name: tool_name, // Use owned name
                    description,
                    parameters: FunctionParameters {
                        r#type: param_type,
                        properties,
                        required,
                    },
                },
            })
        })
        .collect::<Result<Vec<Tool>>>()
        .with_context(|| "Failed to define tools from rmcp::model::Tool vector")
}
