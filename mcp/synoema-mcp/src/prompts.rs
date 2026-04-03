use serde_json::{json, Value};

const CODEGEN_PROMPT: &str = include_str!("../../prompts/codegen.md");

// ── Prompt list ───────────────────────────────────────────

pub fn list() -> Value {
    json!({
        "prompts": [
            {
                "name": "synoema_codegen",
                "description": "System prompt for Synoema code generation. Includes syntax rules, common mistakes, and examples."
            }
        ]
    })
}

// ── Prompt get ────────────────────────────────────────────

pub fn get(name: &str) -> Result<Value, String> {
    match name {
        "synoema_codegen" => Ok(json!({
            "description": "System prompt for Synoema code generation",
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": CODEGEN_PROMPT
                    }
                }
            ]
        })),
        other => Err(format!("unknown prompt: {other}")),
    }
}
