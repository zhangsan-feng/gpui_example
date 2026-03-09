package user

import (
	"gin_server/datastore"
	"gin_server/entity"
	"gin_server/internal"
	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
	"log"
	"math/rand"
	"sync"
	"time"
)

type LoginApiParams struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

func LoginApi(r *gin.Context) {
	params := &LoginApiParams{}

	err := r.ShouldBind(params)
	if err != nil {
		log.Println(err)
		return
	}
	userUuid := uuid.New().String()
	if val, ok := datastore.UserMapper[params.Username]; ok {
		userUuid = val
	}

	token, err := internal.GenerateJWT(&internal.MyCustomClaims{
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
		datastore.AllUsers[userUuid] = &entity.User{
			Id:                       userUuid,
			Name:                     params.Username,
			Avatar:                   datastore.UserAvatar[index],
			WebSocketConn:            nil,
			Status:                   "在线",
			MessageGroups:            []string{},
			Friends:                  []*entity.FriendUser{},
			FriendNotice:             []*entity.UserMessageNotice{},
			RealTimeMessage:          nil,
			CloseWebSocketConnSignal: nil,
			Lock:                     &sync.Mutex{},
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
