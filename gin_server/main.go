package main

import (
	"bufio"
	"gin_server/api"
	"gin_server/api/datastore"
	"gin_server/event_bus"
	"gin_server/global"
	"github.com/ThreeDotsLabs/watermill/message"
	"github.com/gin-gonic/gin"
	"io"
	"log"
	"os"
	"sync"
	"time"
)

type FileHook struct {
	ch          chan []byte
	logFile     *os.File
	logPath     string
	logFileData string
}

func NewFileHook() *FileHook {
	w := &FileHook{ch: make(chan []byte, 4096)}
	go w.processLogs()
	return w
}

var bufferPool = sync.Pool{
	New: func() interface{} {
		return make([]byte, 0, 4096)
	},
}

func (w *FileHook) Write(p []byte) (n int, err error) {
	buf := bufferPool.Get().([]byte)
	if cap(buf) < len(p) {
		buf = make([]byte, len(p))
	} else {
		buf = buf[:len(p)]
	}
	copy(buf, p)
	w.ch <- buf
	return len(p), nil
}

func (w *FileHook) processLogs() {

	defer func() {
		if w.logFile != nil {
			_ = w.logFile.Close()
		}
	}()

	for buf := range w.ch {
		today := time.Now().Format("2006_01_02")
		if w.logFileData != today {
			if w.logFile != nil {
				_ = w.logFile.Close()
			}
			_ = os.MkdirAll("./logs", 0755)
			filename := "./logs/" + today + ".log"
			f, err := os.OpenFile(filename, os.O_RDWR|os.O_CREATE|os.O_APPEND, 0666)
			if err != nil {
				log.Println("failed to open log file:", err)
				bufferPool.Put(buf[:0])
				continue
			}
			bufio.NewWriter(f)
			w.logFile = f
			w.logFileData = today
		}

		_, _ = w.logFile.Write(buf)
		bufferPool.Put(buf[:0])
	}
}

func init() {

	go func() {
		defer func() {
			if err := recover(); err != nil {
				log.Println(err)
			}
		}()

		global.EventBus.RegisterEventHandler(event_bus.EventWebSocketMessage, false, func(msg *message.Message) {
			log.Println(string(msg.Payload))
		})
	}()
}

func main() {
	defer func() {
		if err := recover(); err != nil {
			log.Println(err)
		}
	}()
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	if _, osPathErr := os.Stat("./logs"); os.IsNotExist(osPathErr) {
		if osMkdirErr := os.MkdirAll("./logs", 0777); osMkdirErr != nil {
			log.Fatalln("os mkdir error")
		}
	}

	logWriterHandler := io.MultiWriter(os.Stdout)
	//logWriterHandler := io.MultiWriter(os.Stdout, NewFileHook())

	log.SetOutput(logWriterHandler)

	gin.SetMode(gin.ReleaseMode)
	go api.CheckWebsocketConn()

	engine := gin.Default()
	api.NewHttpRouter(engine)
	address := "0.0.0.0:34332"
	log.Println("http server start ", address)

	datastore.InitData()

	if err := engine.Run(address); err != nil {
		log.Println("http server start failed", address)
	}
}

/*
nohup ../go/bin/go run main.go >> /dev/null 2>&1 &
*/
