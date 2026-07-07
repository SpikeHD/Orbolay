use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeMessage {
  pub cmd: String,
  #[serde(flatten)]
  pub data: Value,
}
