package entity

import (
	"github.com/gorilla/websocket"
	"log"
	"sync"
	"time"
)

type UserDetailInfo struct {
	Friends       []*FriendUser   `json:"friends"`
	MessageGroups []*MessageGroup `json:"message_groups"`
}

type User struct {
	Id                       string               `json:"id"`
	Name                     string               `json:"name"`
	Avatar                   string               `json:"avatar"`
	Status                   string               `json:"state"`
	MessageGroups            []string             `json:"message_groups"`
	Friends                  []*FriendUser        `json:"friends"`
	FriendNotice             []*UserMessageNotice `json:"notice"`
	WebSocketConn            *websocket.Conn      `json:"-"`
	RealTimeMessage          chan []byte          `json:"-"`
	CloseWebSocketConnSignal chan struct{}        `json:"-"`
	Lock                     *sync.Mutex          `json:"-"`
}

func (u *User) WebSocketConnWrite() {
	ticker := time.NewTicker(time.Second * 60)
	defer func() {
		ticker.Stop()
		_ = u.WebSocketConn.Close()
		//delete(AllUsers, u.Id)
		log.Printf("User %s write loop exited", u.Id)
	}()

	for {
		select {
		case msg, ok := <-u.RealTimeMessage:
			if !ok {
				_ = u.WebSocketConn.SetWriteDeadline(time.Now().Add(time.Second * 3))
				_ = u.WebSocketConn.WriteMessage(websocket.CloseMessage, []byte(""))
				log.Println("Channel closed, exiting...")
				return
			}

			if err := u.WebSocketConn.WriteMessage(websocket.TextMessage, msg); err != nil {
				log.Println("Write error:", err)
				return
			}

		case <-ticker.C:
			if err := u.WebSocketConn.WriteMessage(websocket.PingMessage, nil); err != nil {
				log.Println("Ping failed:", err)
				return
			}

		case <-u.CloseWebSocketConnSignal:
			log.Println("Received exit signal")
			return
		}
	}
}

func (u *User) WebSocketConnRead() {

	defer func() {
		close(u.CloseWebSocketConnSignal)
		log.Printf("User %s disconnected and cleaned up", u.Id)
	}()

	for {
		_, msg, err := u.WebSocketConn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("Read error (unexpected): %v", err)
			} else {
				log.Printf("Read error (normal close): %v %s", err, string(msg))
			}
			break
		}
	}
}

type FriendUser struct {
	Id     string `json:"id"`
	Name   string `json:"name"`
	Avatar string `json:"avatar"`
	Status string `json:"state"`
}

type UserMessageNotice struct {
	Id         string      `json:"id"`
	SelfUser   *FriendUser `json:"self_user"`
	FriendUser *FriendUser `json:"friend_user"`
	Status     string      `json:"state"`
}
