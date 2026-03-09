use serde::{Deserialize, Serialize};

pub mod login;
pub mod home;
mod message_page;
mod friend_page;

pub const fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct User{
    pub id: String,
    pub name: String,
    pub avatar: String,
    // pub status: String,
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

pub enum WsMsgType{

}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WsMsgEvent{
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data:serde_json::Value,
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserDetailInfo{
    pub friends: Vec<User>,
    pub message_groups: Vec<MessageGroup>,
}