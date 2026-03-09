package friend

import (
	"gin_server/datastore"
	"gin_server/entity"
	"gin_server/enum"
	"github.com/gin-gonic/gin"
	"github.com/gogf/gf/v2/util/gconv"
	"log"
)

type AddFriendApiParams struct {
	SelfUserId   string `json:"self_user_id" form:"self_user_id" binding:"required"`
	FriendUserId string `json:"friend_user_id" form:"friend_user_id" binding:"required"`
}

func AddFriendApi(r *gin.Context) {
	params := &AddFriendApiParams{}
	err := r.ShouldBind(params)
	if err != nil {
		log.Println(err)
		return
	}
	selfUser := datastore.AllUsers[params.SelfUserId]
	friendUser := datastore.AllUsers[params.FriendUserId]

	log.Println(selfUser, friendUser, params.FriendUserId, params.SelfUserId)
	exist := false
	for _, v := range selfUser.Friends {
		if v.Id == params.FriendUserId {
			exist = true
			break
		}
	}

	if exist {
		r.JSON(200, gin.H{
			"code": "200",
			"msg":  "success",
			"data": "",
		})
		return
	}

	self := &entity.FriendUser{
		Id:     selfUser.Id,
		Name:   selfUser.Name,
		Avatar: selfUser.Avatar,
		Status: selfUser.Status,
	}
	friend := &entity.FriendUser{
		Id:     friendUser.Id,
		Name:   friendUser.Name,
		Avatar: friendUser.Avatar,
		Status: friendUser.Status,
	}

	selfUser.Friends = append(selfUser.Friends, friend)
	friendUser.Friends = append(selfUser.Friends, self)

	wsSendMsg := datastore.WebSocketMessage{
		Type: enum.WsMsgAddFriend,
		Data: nil,
	}

	wsSendMsg.Data = friend
	selfUser.RealTimeMessage <- []byte(gconv.String(wsSendMsg))

	wsSendMsg.Data = self
	friendUser.RealTimeMessage <- []byte(gconv.String(wsSendMsg))

	r.JSON(200, gin.H{
		"code": "200",
		"msg":  "success",
		"data": "",
	})
}
