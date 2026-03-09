package group

import (
	"gin_server/datastore"
	"gin_server/entity"
	"gin_server/enum"
	"github.com/gin-gonic/gin"
	"github.com/gogf/gf/v2/util/gconv"
	"log"
	"net/http"
)

type JoinGroupChatParams struct {
	UserId   string `json:"user_id" form:"user_id" binding:"required"`
	GroupId  string `json:"group_id" form:"group_id"`
	FriendId string `json:"friend_id" form:"friend_id"`
}

func JoinGroupChatApi(r *gin.Context) {
	params := &JoinGroupChatParams{}
	if bindError := r.ShouldBind(params); bindError != nil {
		log.Println(bindError)
		r.JSON(http.StatusBadRequest, gin.H{"error": bindError.Error()})
		return
	}
	user := datastore.AllUsers[params.UserId]
	group := datastore.AllGroup[params.GroupId]

	flag1 := false
	flag2 := false

	if group != nil && user != nil {

		groupMember := &entity.GroupMembers{
			GroupId:  group.ID,
			Id:       user.Id,
			Name:     user.Name,
			Avatar:   user.Avatar,
			Usertype: "群员",
			Status:   "",
		}

		for _, val := range group.Members {
			if val.Id == params.UserId {
				flag1 = true
			}
		}
		if !flag1 {
			group.Members = append(group.Members, groupMember)
		}

		for _, val := range user.MessageGroups {
			if val == params.GroupId {
				flag2 = true
			}
		}
		if !flag2 {
			user.MessageGroups = append(user.MessageGroups, params.GroupId)
		}

		datastore.AllUsers[params.UserId].RealTimeMessage <- []byte(gconv.String(
			datastore.WebSocketMessage{
				Type: enum.WsMsgUserJoinGroupChat,
				Data: group,
			}))

		for _, val := range group.Members {
			if activeUser := datastore.AllUsers[val.Id]; activeUser != nil {

				if activeUser.Id == params.UserId {
					continue
				}

				activeUser.RealTimeMessage <- []byte(gconv.String(
					datastore.WebSocketMessage{
						Type: enum.WsMsgOtherJoinGroupChat,
						Data: groupMember,
					}))

			}
		}
	}

	r.JSON(200, gin.H{
		"code": "200",
		"data": "success",
		"msg":  "",
	})
}
