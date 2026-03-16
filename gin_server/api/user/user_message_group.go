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
	data := entity.UserDetail{
		Friends:  []*entity.Friends{},
		Groups:   []*entity.Groups{},
		Sessions: []*entity.MessageGroup{},
	}
	if user != nil {
		for _, val := range user.MessageGroups {
			data.Sessions = append(data.Sessions, datastore.AllGroup[val])
		}
		for _, val := range user.Friends {
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
