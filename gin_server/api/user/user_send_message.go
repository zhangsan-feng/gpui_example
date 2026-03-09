package user

import (
	"gin_server/datastore"
	"gin_server/entity"
	"gin_server/enum"
	"github.com/gin-gonic/gin"
	"github.com/gogf/gf/v2/os/gfile"
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"log"
	"net/http"
	"time"
)

type UserSendMessageApiParams struct {
	SendUserId  string `form:"send_user_id" binding:"required"`
	SendGroupId string `form:"send_group_id" binding:"required"`
	Message     string `form:"message"`
}

func UserSendMessageApi(r *gin.Context) {
	params := &UserSendMessageApiParams{}
	if bindError := r.ShouldBind(&params); bindError != nil {
		log.Println(bindError)
		return
	}

	//log.Println(req.SendUserId, req.SendGroupId, req.Message)

	form, fromFilesError := r.MultipartForm()
	if fromFilesError != nil {
		log.Println(fromFilesError)
	}

	files := form.File["files"]
	//log.Println(files)

	messageFiles := []string{}

	if len(files) > 0 {
		for _, fileHeader := range files {
			tmpFilePath := "./static/files/" + fileHeader.Filename
			messageFiles = append(messageFiles, "http://127.0.0.1:34332/avatar/files/"+fileHeader.Filename)
			if gfile.Exists(tmpFilePath) {
				continue
			}
			if fileHeader.Size > 32*1024*1024 {
				continue
			}
			if err := r.SaveUploadedFile(fileHeader, tmpFilePath); err != nil {
				r.JSON(http.StatusInternalServerError, gin.H{"error": "failed to save file"})
				return
			}
		}
	}

	//AllUsers[req.SendUserId].WebSocketConn.WriteMessage(websocket.TextMessage, []byte(req.Message))
	//if err := global.EventBus.Publish(event_bus.EventWebSocketMessage, gconv.String("")); err != nil {
	//	log.Println(err)
	//}

	//log.Println(messageFiles)
	if len(messageFiles) == 0 && len(params.Message) == 0 {
		return
	}

	group := datastore.AllGroup[params.SendGroupId]
	if group == nil {
		return
	}
	exist := false
	for _, groupMember := range group.Members {
		if groupMember.Id == params.SendUserId {
			exist = true
		}
	}
	if !exist {
		groupMember := &entity.GroupMembers{
			GroupId:  group.ID,
			Id:       params.SendUserId,
			Name:     datastore.AllUsers[params.SendUserId].Name,
			Avatar:   datastore.AllUsers[params.SendUserId].Avatar,
			Usertype: "普通群员",
			Status:   datastore.AllUsers[params.SendUserId].Status,
		}

		group.Members = append(group.Members, groupMember)
		for _, v := range group.Members {
			if datastore.AllUsers[v.Id] != nil {
				log.Println(v.Name)
				if datastore.AllUsers[v.Id].WebSocketConn != nil {
					send := &datastore.WebSocketMessage{
						Type: enum.WsMsgOtherJoinGroupChat,
						Data: groupMember,
					}
					if err := datastore.AllUsers[v.Id].WebSocketConn.WriteMessage(websocket.TextMessage, []byte(gconv.String(send))); err != nil {
						log.Println(err)
					}
				}
			}
		}

	}

	sendMsg := &entity.GroupHistory{
		GroupId:        group.ID,
		MessageId:      uuid.New().String(),
		SendGroupId:    params.SendGroupId,
		SendUserId:     params.SendUserId,
		SendUserName:   datastore.AllUsers[params.SendUserId].Name,
		SendUserAvatar: datastore.AllUsers[params.SendUserId].Avatar,
		Message:        params.Message,
		Time:           time.Now().Format("2006-01-02 15:04:05"),
		Files:          messageFiles,
	}

	if len(group.History) == 1000 {
		group.History = group.History[500:]
	}
	group.History = append(group.History, sendMsg)

	for _, v := range group.Members {
		if datastore.AllUsers[v.Id] != nil {
			if datastore.AllUsers[v.Id].WebSocketConn != nil {
				//log.Println(v.Name)

				//log.Println(gconv.String(sendMsg))
				send := &datastore.WebSocketMessage{
					Type: enum.WsMsgTypeMessage,
					Data: sendMsg,
				}
				if err := datastore.AllUsers[v.Id].WebSocketConn.WriteMessage(websocket.TextMessage, []byte(gconv.String(send))); err != nil {
					log.Println(err)
				}
			}
		}
	}
	r.JSON(200, gin.H{
		"code": "200",
		"data": "ok",
		"msg":  "",
	})
	//log.Println(data)
}
