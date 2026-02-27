package api

import (
	"gin_server/api/datastore"
	"github.com/gorilla/websocket"
	"log"
	"time"
)

func CheckWebsocketConn() {
	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	defer func() {
		if err := recover(); err != nil {
			log.Println(err)
		}
	}()

	for {
		select {
		case <-ticker.C:
			for k := range datastore.AllUsers {
				//log.Println(datastore.AllUsers[k].Name)
				if datastore.AllUsers[k].Conn == nil {
					datastore.AllUsers[k].Status = "离线"
					continue
				}
				//for j := range AllGroup {
				//	for l := range AllGroup[j].Members {
				//		if AllUsers[k].Id == AllGroup[j].Members[l].Id {
				//			AllGroup[j].Members = append(AllGroup[j].Members[:l], AllGroup[j].Members[l+1:]...)
				//		}
				//	}
				//}
				if err := datastore.AllUsers[k].Conn.WriteMessage(websocket.PingMessage, []byte("ping")); err != nil {
					datastore.AllUsers[k].Status = "离线"
					//log.Printf("发送 ping 失败: %v", err)
				}

			}
		}
	}
}
