use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct User{
    pub id: String,
    pub name: String,
    pub avatar: String,
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GroupHistory {
    pub group_id: String,
    pub message_id: String,
    pub send_group_id: String,
    pub send_user_id: String,
    pub send_username: String,
    pub send_user_avatar: String,
    pub message: String,
    pub time: String,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GroupMembers {
    pub group_id: String,
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub user_type: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MessageGroup {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub history: Vec<GroupHistory>,
    pub members: Vec<GroupMembers>,
    #[serde(rename = "type")]
    pub group_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LeftSidebarMessageGroup {
    pub id: String,
    pub name: String,
    pub avatar: String,
} 

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WsMsgEvent{
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data:serde_json::Value,
}