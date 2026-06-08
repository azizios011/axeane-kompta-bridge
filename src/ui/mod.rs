pub mod editor;
pub mod formats;
pub mod settings;
pub mod llm;

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

#[derive(PartialEq)]
pub enum ActiveTab {
    CsvEditor,
    FormatTemplates,
    BrowserSettings,
    LlmSettings,
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
