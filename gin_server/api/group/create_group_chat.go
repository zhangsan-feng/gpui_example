package group

import (
	"gin_server/datastore"
	"gin_server/entity"
	"gin_server/enum"
	"github.com/gin-gonic/gin"
	"github.com/gogf/gf/v2/os/gfile"
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/google/uuid"
	"log"
)

type CreateGroupChatParams struct {
	GroupName string   `json:"group_name" form:"group_name"`
	GroupId   string   `json:"group_id" form:"group_id"`
	UserId    []string `json:"user_id" form:"user_id" binding:"required"`
	Type      string   `json:"type" form:"type" binding:"required"`
}

/*
		存在的直接返回
		给用户发私聊 创建
	    直接创建
*/
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
	groupMember := []*entity.GroupMembers{}

	for _, userId := range params.UserId {
		groupMember = append(groupMember, &entity.GroupMembers{
			Id:       datastore.AllUsers[userId].Id,
			Name:     datastore.AllUsers[userId].Name,
			Avatar:   datastore.AllUsers[userId].Avatar,
			Usertype: "群主",
			Status:   "",
		})
	}
	groupInfo := &entity.MessageGroup{
		ID:      groupUuid,
		Name:    params.GroupName,
		Avatar:  datastore.StaticAddress + "/avatar/group_avatar/" + tmpFileName,
		History: []*entity.GroupHistory{},
		Members: groupMember,
		Type:    params.Type,
	}

	datastore.AllGroup[groupUuid] = groupInfo
	send := &datastore.WebSocketMessage{
		Type: enum.WsMsgCreateGroupChat,
		Data: groupInfo,
	}
	//log.Println(groupInfo)
	for _, userId := range params.UserId {
		flag := false
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

			user.RealTimeMessage <- []byte(gconv.String(send))
		}
	}
	r.JSON(200, gin.H{
		"code": "200",
		"data": "success",
		"msg":  "",
	})
}
