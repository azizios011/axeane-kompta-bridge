use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditableRow {
    pub client: String,
    pub reference: String,
    pub date: String,
    pub ttc: String,
    pub ht: String,
    pub tva: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    Debit,
    Credit,
}

#[derive(Debug, Clone)]
pub struct TemplateRow {
    pub compte: String,
    pub entry_type: EntryType,
    pub formula: String,
}

#[derive(Debug, Clone)]
pub struct FormatTemplate {
    pub id: Option<i32>,
    pub client_key: String,
    pub rows: Vec<TemplateRow>,
}

#[derive(Clone)]
pub struct LlmConfig {
    pub provider_name: String,
    pub endpoint_url: String,
    pub api_key: String,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

pub struct SharedAppState {
    pub csv_rows: Vec<EditableRow>,
    pub templates: Vec<FormatTemplate>,
    pub port: String,
    pub browser_path: String,
    pub incognito: bool,
    pub status: String,
    pub trigger_injection: bool,
    pub auto_detect_active: bool,
    pub is_loading: bool,
    pub llm: LlmConfig,
}

pub fn initial_state() -> SharedAppState {
    SharedAppState {
        csv_rows: Vec::new(),
        templates: vec![FormatTemplate {
            id: Some(1),
            client_key: "PASSAGER".to_string(),
            rows: vec![
                TemplateRow {
                    compte: "41100000".to_string(),
                    entry_type: EntryType::Debit,
                    formula: "row.ttc".to_string(),
                },
                TemplateRow {
                    compte: "70700000".to_string(),
                    entry_type: EntryType::Credit,
                    formula: "row.ht".to_string(),
                },
                TemplateRow {
                    compte: "43670000".to_string(),
                    entry_type: EntryType::Credit,
                    formula: "row.tva".to_string(),
                },
            ],
        }],
        port: "9222".to_string(),
        browser_path: "chrome".to_string(),
        incognito: true,
        status: "System online. Configure parameters within AI Core tab before parsing document.pdf.".to_string(),
        trigger_injection: false,
        auto_detect_active: false,
        is_loading: false,
        llm: LlmConfig {
            provider_name: "DeepSeek".to_string(),
            endpoint_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
            api_key: "".to_string(),
            model_name: "deepseek-chat".to_string(),
            temperature: 0.1,
            max_tokens: 3000,
        },
    }
}
