package group

import (
	"gin_server/api/datastore"
	"github.com/gin-gonic/gin"
	"github.com/gogf/gf/v2/os/gfile"
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"log"
)

type CreateGroupChatParams struct {
	GroupName string   `json:"group_name" form:"group_name" binding:"required"`
	UserId    []string `json:"user_id" form:"user_id" binding:"required"`
	Type      string   `json:"type" form:"type" binding:"required"`
}

func CreateGroupChat(r *gin.Context) {
	params := &CreateGroupChatParams{}
	if bindError := r.ShouldBind(params); bindError != nil {
		log.Println(bindError)
		return
	}

	tmpFilePath := ""
	tmpFileName := ""
	if params.Type == "group_chat" {
		file, fileError := r.FormFile("file")
		if fileError != nil {
			log.Println(fileError)
		}
		tmpFileName = file.Filename

		tmpFilePath = "./static/group_avatar/" + file.Filename
		if !gfile.Exists(tmpFilePath) {
			if saveFileError := r.SaveUploadedFile(file, tmpFilePath); saveFileError != nil {
				log.Println(saveFileError)
			}
		}
	}

	groupUuid := uuid.New().String()
	groupMember := []*datastore.GroupMembers{}

	for _, userId := range params.UserId {
		groupMember = append(groupMember, &datastore.GroupMembers{
			Id:       datastore.AllUsers[userId].Id,
			Name:     datastore.AllUsers[userId].Name,
			Avatar:   datastore.AllUsers[userId].Avatar,
			Usertype: "群主",
			Status:   "",
		})
	}
	groupInfo := &datastore.MessageGroup{
		ID:      groupUuid,
		Name:    params.GroupName,
		Avatar:  datastore.StaticAddress + "/avatar/group_avatar/" + tmpFileName,
		History: []*datastore.GroupHistory{},
		Members: groupMember,
		Type:    params.Type,
	}

	datastore.AllGroup[groupUuid] = groupInfo
	send := &datastore.WebSocketMessage{
		GroupId: groupUuid,
		Type:    datastore.WsMsgCreateGroupChat,
		Data:    groupInfo,
	}
	//log.Println(groupInfo)
	flag := false
	for _, userId := range params.UserId {
		user := datastore.AllUsers[userId]
		//log.Println(params.UserId)
		if user != nil {
			for _, val := range user.MessageGroups {
				if val == groupUuid {
					flag = true
				}
			}
			if !flag {
				user.MessageGroups = append([]string{groupUuid}, user.MessageGroups...)
			}
			//log.Println(user.Id)
			//log.Println(user.MessageGroups)

			datastore.AllGroup[groupUuid] = groupInfo
			if sendError := user.Conn.WriteMessage(websocket.TextMessage, []byte(gconv.String(send))); sendError != nil {
				log.Println(sendError)
			}
		}
	}
	r.JSON(200, gin.H{
		"code": "200",
		"data": "success",
		"msg":  "",
	})
}
