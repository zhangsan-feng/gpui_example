package datastore

import (
	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"log"
	"os"
)

type User struct {
	Id            string          `json:"id"`
	Name          string          `json:"name"`
	Avatar        string          `json:"avatar"`
	Conn          *websocket.Conn `json:"-"`
	Status        string          `json:"state"`
	MessageGroups []string        `json:"message_groups"`
	FriendGroups  []string        `json:"friend_groups"`
}

type GroupHistory struct {
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

type WebSocketMessage struct {
	GroupId string      `json:"group_id"`
	Type    string      `json:"type"`
	Data    interface{} `json:"data"`
}

var AllGroup map[string]*MessageGroup
var AllUsers map[string]*User
var UserAvatar []string
var UserMapper map[string]string

/*
所有用户
用户有哪些群
	群里有哪些成员
	群里历史消息
用户有哪些好友
*/

const StaticAddress = "http://127.0.0.1:34332"

const (
	WsMsgTypeMessage        = "message"
	WsMsgUserJoinGroupChat  = "user_join_group_chat"
	WsMsgOtherJoinGroupChat = "other_join_group_chat"
	WsMsgExitGroupChat      = "exit_group_chat"
	WsMsgDropGroupChat      = "drop_group_chat"
	WsMsgCreateGroupChat    = "create_group_chat"
)

func InitData() {
	AllGroup = make(map[string]*MessageGroup)
	AllUsers = make(map[string]*User)
	UserMapper = make(map[string]string)
	GroupAvatar, err := os.ReadDir("./static/group_avatar/")
	if err != nil {
		log.Println("读取目录失败: ", err)
		return
	}

	for _, entry := range GroupAvatar {
		groupUuid := uuid.New().String()
		AllGroup[groupUuid] = &MessageGroup{
			ID:      groupUuid,
			Name:    groupUuid,
			Avatar:  StaticAddress + "/avatar/group_avatar/" + entry.Name(),
			History: []*GroupHistory{},
			Members: []*GroupMembers{},
			Type:    "group",
		}

	}

	userAvatar, err := os.ReadDir("./static/user_avatar/")
	if err != nil {
		log.Println("read dir filed: ", err)
		return
	}

	for _, entry := range userAvatar {
		UserAvatar = append(UserAvatar, StaticAddress+"/avatar/user_avatar/"+entry.Name())
		//userUuid := uuid.New().String()

		//userData := &User{
		//	Id:     userUuid,
		//	Name:   userUuid,
		//	Avatar: StaticAddress + "/avatar/user_avatar/" + entry.Name(),
		//	Conn:   nil,
		//}
		//AllUsers[userUuid] = userData
	}

	//
	//for k := range AllUsers {
	//	for j := range AllGroup[k] {
	//		AllGroup[k][j].Members = groupUserList
	//	}
	//}

	//log.Println(ActiveGroup)
	//log.Println(ActiveUsers)
	//for i := range AllGroup {
	//	log.Println(i)
	//}

}
