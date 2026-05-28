use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: CommandErrorCode,
    pub message: String,
}

impl CommandError {
    pub fn new(code: CommandErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandErrorCode {
    AutomationFailed,
    CodexNotFound,
    InvalidState,
    WindowsApiFailed,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum ProcessPriorityRequest {
    Normal,
    High,
}

impl ProcessPriorityRequest {
    pub fn cli_value(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::High => "High",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "Normal" => Some(Self::Normal),
            "High" => Some(Self::High),
            _ => None,
        }
    }
}
