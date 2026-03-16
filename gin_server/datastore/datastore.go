package datastore

import (
	"gin_server/entity"
	"log"
	"os"

	"github.com/google/uuid"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

var db *gorm.DB
var err error

var AllGroup map[string]*entity.MessageGroup
var AllUsers map[string]*entity.User
var UserAvatar []string
var UserMapper map[string]string

type WebSocketMessage struct {
	Type string      `json:"type"`
	Data interface{} `json:"data"`
}

const StaticAddress = "http://127.0.0.1:34332"

func InitData() {
	db, err = gorm.Open(sqlite.Open("./sqlite.db?cache=shared&_fk=1"), &gorm.Config{
		Logger: logger.New(
			log.New(os.Stdout, "\r\n", log.LstdFlags),
			logger.Config{
				LogLevel: logger.Info,
			},
		),
	})
	if err != nil {
		panic("连接SQLite数据库失败: " + err.Error())
	}

	sqls := []string{
		CreateUsersTableSQL,
		CreateGroupsTableSQL,
		CreateFriendsTableSQL,
		CreateGroupMembersTableSQL,
		CreateGroupHistoryTableSQL,
	}
	for _, sql := range sqls {
		if execErr := db.Exec(sql).Error; execErr != nil {
			panic("SQLite exec failed: " + execErr.Error())
		}
	}

	AllGroup = make(map[string]*entity.MessageGroup)
	AllUsers = make(map[string]*entity.User)
	UserMapper = make(map[string]string)
	GroupAvatar, err := os.ReadDir("./static/group_avatar/")
	if err != nil {
		log.Println("读取目录失败: ", err)
		return
	}

	for _, entry := range GroupAvatar {
		groupUuid := uuid.New().String()
		AllGroup[groupUuid] = &entity.MessageGroup{
			ID:      groupUuid,
			Name:    groupUuid,
			Avatar:  StaticAddress + "/avatar/group_avatar/" + entry.Name(),
			History: []*entity.GroupHistory{},
			Members: []*entity.GroupMembers{},
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
		userUuid := uuid.New().String()

		userData := &entity.User{
			Id:              userUuid,
			Name:            userUuid,
			Avatar:          StaticAddress + "/avatar/user_avatar/" + entry.Name(),
			WebSocketConn:   nil,
			Status:          "",
			MessageGroups:   []string{},
			Friends:         []*entity.Friends{},
			RealTimeMessage: make(chan []byte, 1024),
		}
		AllUsers[userUuid] = userData
	}

	for j := range AllGroup {
		for k := range AllUsers {
			AllGroup[j].Members = append(AllGroup[j].Members, &entity.GroupMembers{
				GroupId:  AllGroup[j].ID,
				Id:       AllUsers[k].Id,
				Name:     AllUsers[k].Name,
				Avatar:   AllUsers[k].Avatar,
				Usertype: "群员",
				Status:   AllUsers[k].Status,
			})
		}
	}

	//log.Println(ActiveGroup)
	//log.Println(ActiveUsers)
	//for i := range AllGroup {
	//	log.Println(i)
	//}

}
