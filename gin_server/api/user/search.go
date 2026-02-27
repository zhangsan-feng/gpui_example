package user

import (
	"gin_server/api/datastore"
	"github.com/gin-gonic/gin"
	"log"
	"strings"
)

type SearchParams struct {
	Keyword string `json:"keyword" form:"keyword" binding:"required"`
}

type SearchResponse struct {
	Groups []*datastore.MessageGroup `json:"groups"`
	Users  []*datastore.User         `json:"users"`
}

func SearchGroupAndUserApi(r *gin.Context) {
	params := &SearchParams{}
	if bindError := r.ShouldBind(params); bindError != nil {
		log.Println(bindError)
		r.JSON(400, gin.H{
			"code": "400",
			"msg":  "参数错误",
			"data": "",
		})
		return
	}

	result := &SearchResponse{
		Groups: []*datastore.MessageGroup{},
		Users:  []*datastore.User{},
	}

	keyword := strings.ToLower(params.Keyword)

	for _, group := range datastore.AllGroup {
		groupName := strings.ToLower(group.Name)
		if strings.Contains(groupName, keyword) {
			result.Groups = append(result.Groups, group)
		}
	}

	for _, user := range datastore.AllUsers {
		userName := strings.ToLower(user.Name)
		if strings.Contains(userName, keyword) {
			result.Users = append(result.Users, user)
		}
	}

	r.JSON(200, gin.H{
		"code": "200",
		"msg":  "success",
		"data": result,
	})
}
