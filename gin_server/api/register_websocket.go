package api

import (
	"gin_server/api/datastore"
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
	log.Println(gconv.Map(string(msg))["id"])
	log.Println(gconv.Map(string(msg))["token"])
	id := gconv.String(gconv.Map(string(msg))["id"])

	if datastore.AllUsers[id] != nil {
		datastore.AllUsers[id].Conn = conn
	} else {
		if connError := conn.Close(); connError != nil {
			log.Println(connError)
		}
	}

}
