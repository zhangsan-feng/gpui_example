package api

import (
	"gin_server/datastore"
	"github.com/gin-gonic/gin"
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gorilla/websocket"
	"log"
	"net/http"
)

func RegisterWsConn(r *gin.Context) {
	w := websocket.Upgrader{
		CheckOrigin: func(r *http.Request) bool {
			return true
		},
	}

	conn, err := w.Upgrade(r.Writer, r.Request, nil)
	if err != nil {
		log.Printf("升级 WebSocket 失败: %v", err)
		return
	}
	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Println(err)
	}
	log.Println("userid:", gconv.Map(string(msg))["id"])
	log.Println("user_token:", gconv.Map(string(msg))["token"])
	id := gconv.String(gconv.Map(string(msg))["id"])

	if datastore.AllUsers[id] != nil {
		if datastore.AllUsers[id].WebSocketConn != nil {
			datastore.AllUsers[id].CloseWebSocketConnSignal <- struct{}{}
		}
		
		datastore.AllUsers[id].WebSocketConn = conn
		datastore.AllUsers[id].RealTimeMessage = make(chan []byte, 1024)
		datastore.AllUsers[id].CloseWebSocketConnSignal = make(chan struct{})
		go datastore.AllUsers[id].WebSocketConnWrite()
		go datastore.AllUsers[id].WebSocketConnRead()
	} else {
		if connError := conn.Close(); connError != nil {
			log.Println(connError)
		}
	}

}
