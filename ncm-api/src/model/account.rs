use crate::model::FromJson;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Account {
    /// 用户 id
    pub user_id: u64,
    /// 昵称
    pub nickname: String,
    /// 等级
    pub vip_type: i64,
}

impl FromJson for Account {
    type SelfType = Account;

    fn from_json(value: Value) -> Result<Self::SelfType> {
        let user_id = value["userId"].as_u64().unwrap();
        let nickname = value["nickname"].as_str().unwrap().to_string();
        let vip_type = value["vipType"].as_i64().unwrap();

        Ok(Account { user_id, nickname, vip_type })
    }
}
