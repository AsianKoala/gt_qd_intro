use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMsgRoot {
    #[serde(rename = "type")]
    pub type_field: String,
    pub topic: String,
    pub subject: String,
    pub data: MarketData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrU64 {
    Str(String),
    U64(u64),
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketData {
    pub asks: Vec<Vec<StringOrU64>>,
    pub bids: Vec<Vec<StringOrU64>>,
    pub ts: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WelcomeMsg {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerRoot {
    pub code: String,
    pub data: KCServers,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KCServers {
    pub token: String,
    pub instance_servers: Vec<InstanceServer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceServer {
    pub endpoint: String,
    pub encrypt: bool,
    pub protocol: String,
    pub ping_interval: u64,
    pub ping_timeout: u32,
}
