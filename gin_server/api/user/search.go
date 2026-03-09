package user

import (
	"gin_server/datastore"
	"gin_server/entity"
	"github.com/gin-gonic/gin"
	"log"
	"strings"
)

type SearchParams struct {
	Keyword string `json:"keyword" form:"keyword" binding:"required"`
	UserId  string `json:"user_id" form:"user_id" binding:"required"`
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

	result := &entity.SearchResult{
		Groups: []*entity.SearchGroupResult{},
		Users:  []*entity.SearchUserResult{},
	}

	keyword := strings.ToLower(params.Keyword)

	for _, group := range datastore.AllGroup {
		hasGroup := false
		for _, val := range datastore.AllUsers[params.UserId].MessageGroups {
			if val == group.ID {
				hasGroup = true
				break
			}
		}

		if !hasGroup {
			if keyword == "" || strings.Contains(strings.ToLower(group.Name), strings.ToLower(keyword)) {
				result.Groups = append(result.Groups, &entity.SearchGroupResult{
					ID:     group.ID,
					Name:   group.Name,
					Avatar: group.Avatar,
				})
			}
		}
	}

	for _, user := range datastore.AllUsers {
		hasFriend := false
		for _, val := range datastore.AllUsers[params.UserId].Friends {
			if val.Id == user.Id {
				hasFriend = true
				break
			}
		}
		if user.Id == params.UserId {
			hasFriend = true
		}
		if !hasFriend {
			if strings.Contains(strings.ToLower(user.Name), keyword) {
				result.Users = append(result.Users, &entity.SearchUserResult{
					ID:     user.Id,
					Name:   user.Name,
					Avatar: user.Avatar,
				})
			}
		}
	}

	//log.Println(result.Users)
	//log.Println(result.Groups)

	r.JSON(200, gin.H{
		"code": "200",
		"msg":  "success",
		"data": result,
	})
}
