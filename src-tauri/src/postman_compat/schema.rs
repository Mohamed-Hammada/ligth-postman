use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    #[serde(default)]
    pub item: Vec<PostmanItem>,
    #[serde(default)]
    pub variable: Vec<PostmanVariable>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
    #[serde(default)]
    pub event: Vec<PostmanEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanItem {
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// If present, this item is a folder containing child items.
    #[serde(default)]
    pub item: Option<Vec<PostmanItem>>,
    /// If present, this item is a request.
    #[serde(default)]
    pub request: Option<PostmanRequest>,
    #[serde(default)]
    pub response: Option<Vec<PostmanResponse>>,
    #[serde(default)]
    pub event: Option<Vec<PostmanEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanRequest {
    pub url: PostmanUrlOrString,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub header: Option<Vec<PostmanHeader>>,
    #[serde(default)]
    pub body: Option<PostmanBody>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanUrlOrString {
    Structured(PostmanUrl),
    Plain(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanUrl {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub host: Option<PostmanSegments>,
    #[serde(default)]
    pub path: Option<PostmanSegments>,
    #[serde(default)]
    pub query: Option<Vec<PostmanQueryParam>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanSegments {
    List(Vec<String>),
    Single(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanQueryParam {
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanHeader {
    pub key: Option<String>,
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanBody {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub formdata: Option<Vec<PostmanFormDataItem>>,
    #[serde(default)]
    pub urlencoded: Option<Vec<PostmanUrlEncodedItem>>,
    #[serde(default)]
    pub graphql: Option<PostmanGraphQLBody>,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanFormDataItem {
    pub key: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub src: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanUrlEncodedItem {
    pub key: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanGraphQLBody {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub variables: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanAuth {
    pub r#type: String,
    #[serde(default)]
    pub bearer: Option<Vec<PostmanAuthParam>>,
    #[serde(default)]
    pub basic: Option<Vec<PostmanAuthParam>>,
    #[serde(default)]
    pub apikey: Option<Vec<PostmanAuthParam>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanAuthParam {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanEvent {
    pub listen: String,
    pub script: Option<PostmanScript>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanScript {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub exec: Option<PostmanScriptExec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanScriptExec {
    Lines(Vec<String>),
    Single(String),
}

impl PostmanScriptExec {
    pub fn to_string_content(&self) -> String {
        match self {
            PostmanScriptExec::Lines(lines) => lines.join("\n"),
            PostmanScriptExec::Single(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanResponse {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub code: Option<u16>,
    #[serde(default)]
    pub header: Option<Vec<PostmanHeader>>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanVariable {
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanEnvironment {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub values: Vec<PostmanEnvValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanEnvValue {
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub r#type: Option<String>,
}

fn default_true() -> bool {
    true
}
