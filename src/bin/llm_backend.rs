use crate::app_state::{EditableRow, LlmConfig};
use serde_json::Value;
use std::path::Path;

pub fn rip_pdf_text<P: AsRef<Path>>(path: P) -> Result<String, String> {
    pdf_extract::extract_text_from_mem(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Static Node Reading Failure: {}", e))
}

pub async fn parse_pdf_via_ai(raw_text: &str, config: &LlmConfig) -> Result<Vec<EditableRow>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let system_instructions = r#"
    You are an expert accounting parser. Analyze the provided raw accounting ledger text dump.
    Extract every individual invoice/credit note row into a strictly formatted JSON array matching the template.
    
    CRITICAL RULES:
    1. Identify the Client Header (e.g., lines starting with 'C00' followed by a name). Every invoice beneath it belongs to that client until a new client header appears.
    2. Convert European commas used as decimals into dots (e.g., '981,293' becomes '981.293').
    3. Output ONLY valid JSON array items. Do not wrap in markdown ```json blocks.
    "#;

    let payload = serde_json::json!({
        "model": config.model_name,
        "messages": [
            {"role": "system", "content": system_instructions},
            {"role": "user", "content": raw_text}
        ],
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
        "response_format": { "type": "json_object" }
    });

    let response = client.post(&config.endpoint_url)
        .bearer_auth(&config.api_key)
        .json(&payload)
        .send()
        .await?;

    let res_json: Value = response.json().await?;
    let raw_content = res_json["choices"][0]["message"]["content"].as_str().unwrap_or("[]");
    let parsed_rows: Vec<EditableRow> = serde_json::from_str(raw_content)?;
    Ok(parsed_rows)
}

