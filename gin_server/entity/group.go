package entity

type GroupHistory struct {
	GroupId        string   `json:"group_id"`
	MessageId      string   `json:"message_id"`
	SendGroupId    string   `json:"send_group_id"`
	SendUserId     string   `json:"send_user_id"`
	SendUserName   string   `json:"send_username"`
	SendUserAvatar string   `json:"send_user_avatar"`
	Message        string   `json:"message"`
	Time           string   `json:"time"`
	Files          []string `json:"files"`
}

type GroupMembers struct {
	GroupId  string `json:"group_id"`
	Id       string `json:"id"`
	Name     string `json:"name"`
	Avatar   string `json:"avatar"`
	Usertype string `json:"user_type"`
	Status   string `json:"status"`
}

type MessageGroup struct {
	ID      string          `json:"id"`
	Name    string          `json:"name"`
	Avatar  string          `json:"avatar"`
	History []*GroupHistory `json:"history"`
	Members []*GroupMembers `json:"members"`
	Type    string          `json:"type"`
}
