use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Me {
    pub id: Uuid,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchSet {
    pub id: Uuid,
    pub name: String,
    pub brief: String,
    pub extraction_status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchSetInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub document_id: String,
    pub citation: String,
    pub title: String,
    pub status: String,
    pub whole_document: bool,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub id: Uuid,
    pub part_id: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub id: Uuid,
    pub text: String,
    pub document_id: Option<String>,
    pub part_id: Option<String>,
    pub kind: String,
    pub reason: Option<String>,
    pub file_id: Option<Uuid>,
    pub file_name: Option<String>,
    pub path: Option<String>,
    pub source_part_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub id: Uuid,
    pub watch_set_id: Uuid,
    #[serde(rename = "type")]
    pub change_type: String,
    pub document_id: String,
    pub citation: String,
    pub section_id: Uuid,
    pub part_id: Option<String>,
    pub label: Option<String>,
    pub effective_from: Option<String>,
    pub detected_at: String,
    pub amendments: Vec<Amendment>,
    pub diff: Option<String>,
    pub diff_markdown: Option<String>,
    #[serde(default)]
    pub law_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turboleal_url: Option<String>,
    pub references: Vec<Reference>,
    pub acknowledged_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amendment {
    pub document_id: String,
    pub citation: String,
}
