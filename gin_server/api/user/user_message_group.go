package user

import (
	"gin_server/datastore"
	"gin_server/entity"
	"github.com/gin-gonic/gin"
)

func MessageGroupApi(r *gin.Context) {
	userId := r.Query("user_id")
	//log.Println(user_id)
	user := datastore.AllUsers[userId]
	//log.Println(user)
	data := entity.UserDetailInfo{
		Friends:       []*entity.FriendUser{},
		MessageGroups: []*entity.MessageGroup{},
	}
	if user != nil {
		for _, val := range user.MessageGroups {
			data.MessageGroups = append(data.MessageGroups, datastore.AllGroup[val])
		}
		for _, val := range user.Friends {
			val.Status = ""
			data.Friends = append(data.Friends, val)
		}
	}

	//for _, group := range datastore.AllGroup {
	//	data = append(data, group)
	//}

	r.JSON(200, gin.H{
		"code": "200",
		"msg":  "success",
		"data": data,
	})
}
