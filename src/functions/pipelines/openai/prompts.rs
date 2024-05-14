pub const DEFAULT_SYSTEM_TEMPLATE: &str = r#"
You have access to the following tools:

{tools}

You must always select one of the above tools and respond with only a JSON object matching the following schema:

{
  "tool": <name of the selected tool>,
  "tool_input": <parameters for the selected tool, matching the tool's JSON schema>
}
"#;

pub const DEFAULT_RESPONSE_FUNCTION: &str = r#"
{
    "name": "__conversational_response",
    "description": "Respond conversationally if no other tools should be called for a given query.",
    "parameters": {
        "type": "object",
        "properties": {
            "response": {
                "type": "string",
                "description": "Conversational response to the user."
            }
        },
        "required": ["response"]
    }
}
"#;
