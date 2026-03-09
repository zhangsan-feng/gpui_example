package internal

import (
	"fmt"
	"gin_server/event_bus"
	"github.com/golang-jwt/jwt/v5"
	"github.com/robfig/cron/v3"
	"log"
)

const (
	TokenKey = "token"
)

var EventBus = event_bus.New()

type MyCustomClaims struct {
	Username   string `json:"username"`
	UserAvatar string `json:"user_avatar"`
	UserUuid   string `json:"user_uuid"`
	jwt.RegisteredClaims
}

func GenerateJWT(claims *MyCustomClaims) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, claims)
	tokenString, err := token.SignedString([]byte(TokenKey))
	if err != nil {
		log.Println(err)
		return "", err
	}
	return tokenString, nil
}

func ParseJwt(tokenStr string) (*MyCustomClaims, error) {
	claims := &MyCustomClaims{}
	token, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (interface{}, error) {
		if t.Method != jwt.SigningMethodHS512 {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		return TokenKey, nil
	})

	if err != nil {
		return nil, fmt.Errorf("parse failed: %w", err)
	}

	if !token.Valid {
		return nil, fmt.Errorf("token is invalid")
	}

	return claims, nil
}

type CrontabTask struct {
	CronId cron.EntryID
	Task   *cron.Cron
	Fn     func()
}

func NewCron() *CrontabTask {
	return &CrontabTask{}
}

func (c *CrontabTask) AddTask() {
}

func DelTask() {

}
