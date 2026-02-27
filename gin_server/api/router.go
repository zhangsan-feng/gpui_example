package api

import (
	"bytes"
	"gin_server/api/group"
	"gin_server/api/user"
	"github.com/gin-gonic/gin"
	"io"
	"log"
	"net/http"
	"strings"
)

func NewHttpRouter(r *gin.Engine) {
	r.Static("/avatar", "./static")

	r.Use(func(c *gin.Context) {
		contentType := c.GetHeader("Content-Type")
		if strings.HasPrefix(contentType, "multipart/form-data") {
			err := c.Request.ParseMultipartForm(32 << 20)
			if err != nil {
				log.Printf("Failed to parse multipart form: %v\n", err)
				c.AbortWithStatusJSON(http.StatusBadRequest, gin.H{"error": "invalid multipart form"})
				return
			}

			for fieldName, fileHeaders := range c.Request.MultipartForm.File {
				for _, fh := range fileHeaders {
					log.Printf("[File Upload] Field: %s, Filename: %s, Size: %d bytes\n",
						fieldName, fh.Filename, fh.Size)
				}
			}

			for key, values := range c.Request.MultipartForm.Value {
				for _, v := range values {
					log.Printf("[Form Field] %s = %s\n", key, v)
				}
			}

		} else {

			body, err := io.ReadAll(c.Request.Body)
			if err != nil {
				log.Println("Failed to read request body: %v\n", err)
				c.AbortWithStatusJSON(http.StatusBadRequest, gin.H{"error": "failed to read body"})
				return
			}

			log.Println(string(body))
			c.Request.Body = io.NopCloser(bytes.NewBuffer(body))
		}

		c.Next()
	})

	r.GET("/user_message_group", user.MessageGroupApi)

	r.POST("/user_send_message", user.UserSendMessageApi)
	r.GET("/register_ws", RegisterWsConn)
	r.POST("/user/login", user.LoginApi)
	r.POST("/create_group_chat", group.CreateGroupChat)
	r.POST("/search_group_and_user", user.SearchGroupAndUserApi)
	r.POST("/join_group_chat", group.JoinGroupChatApi)
}
