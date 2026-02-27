package user

import (
	"gin_server/api/datastore"
	"github.com/gin-gonic/gin"
)

func MessageGroupApi(r *gin.Context) {
	user_id := r.Query("user_id")
	//log.Println(user_id)
	user := datastore.AllUsers[user_id]
	//log.Println(user)
	data := []*datastore.MessageGroup{}
	if user != nil {
		for _, val := range user.MessageGroups {
			data = append(data, datastore.AllGroup[val])
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
