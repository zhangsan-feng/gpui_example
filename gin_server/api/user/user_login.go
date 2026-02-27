package user

import (
	"gin_server/api/datastore"
	"gin_server/global"
	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
	"log"
	"math/rand"
	"time"
)

type LoginParams struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

func LoginApi(r *gin.Context) {
	params := &LoginParams{}

	err := r.ShouldBind(params)
	if err != nil {
		log.Println(err)
		return
	}
	userUuid := uuid.New().String()
	if val, ok := datastore.UserMapper[params.Username]; ok {
		userUuid = val
	}

	token, err := global.GenerateJWT(&global.MyCustomClaims{
		Username:         params.Username,
		UserAvatar:       "",
		UserUuid:         userUuid,
		RegisteredClaims: jwt.RegisteredClaims{},
	})
	if err != nil {
		log.Println(err)
		return
	}

	rand.NewSource(time.Now().UnixNano())

	index := rand.Intn(len(datastore.UserAvatar))

	if datastore.AllUsers[userUuid] == nil {
		datastore.AllUsers[userUuid] = &datastore.User{
			Id:     userUuid,
			Name:   params.Username,
			Avatar: datastore.UserAvatar[index],
			Status: "在线",
			Conn:   nil,
		}
	}

	if _, ok := datastore.UserMapper[params.Username]; !ok {
		datastore.UserMapper[params.Username] = userUuid
	}

	res := gin.H{
		"code": "200",
		"data": gin.H{
			"user_token":  token,
			"user_id":     userUuid,
			"user_avatar": datastore.AllUsers[userUuid].Avatar,
		},
		"msg": "success",
	}
	log.Println(res)
	r.JSON(200, res)

}
