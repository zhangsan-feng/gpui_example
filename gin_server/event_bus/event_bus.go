package event_bus

import (
	"context"
	"github.com/ThreeDotsLabs/watermill"
	"github.com/ThreeDotsLabs/watermill/message"
	"github.com/ThreeDotsLabs/watermill/pubsub/gochannel"
	"github.com/google/uuid"
	"log"
	"runtime/debug"
	"sync"
)
import _ "github.com/gorilla/websocket"

const (
	EventBusTypeDefault   = "default"
	EventWebSocketMessage = "websocket"
	EventCron             = "cron"
)

type CallFn struct {
	async bool
	fn    func(msg *message.Message)
}

type EventType struct {
	UUID      uuid.UUID
	EventType string
	handlers  []CallFn
}

type EventBus struct {
	Event    []*EventType
	pub      *gochannel.GoChannel
	mu       sync.RWMutex
	handlers []CallFn
}

func New() *EventBus {
	pubSub := gochannel.NewGoChannel(
		gochannel.Config{
			OutputChannelBuffer: 100,
		},
		watermill.NewStdLogger(true, true),
	)

	id, err := uuid.NewUUID()
	if err != nil {
		panic(err)
	}

	bus := &EventBus{
		pub: pubSub,
		Event: []*EventType{
			{
				UUID:      id,
				EventType: EventBusTypeDefault,
				handlers:  []CallFn{},
			},
		},
	}
	return bus
}

func (bus *EventBus) startSubscription(eventType string) {

	go func() {
		messages, err := bus.pub.Subscribe(context.Background(), eventType)
		if err != nil {
			log.Printf("Subscribe error for %s: %v\n", eventType, err)
			return
		}
		defer func() {
			if err := recover(); err != nil {
				log.Println("event bus panic:", eventType, err)
				debug.PrintStack()
			}
		}()

		for msg := range messages {
			bus.mu.RLock()
			for _, handler := range bus.handlers {
				if handler.async {
					go handler.fn(msg)
				} else {
					handler.fn(msg)
				}
			}

			for _, e := range bus.Event {
				if e.EventType == eventType {
					log.Printf("Processing %d handlers for event %s", len(e.handlers), eventType)
					for _, handler := range e.handlers {
						if handler.async {
							go handler.fn(msg)
						} else {
							handler.fn(msg)
						}
					}
				}
			}
			bus.mu.RUnlock()
			msg.Ack()
		}
	}()
}

func (bus *EventBus) Publish(event string, msg string) error {
	return bus.pub.Publish(event, message.NewMessage(watermill.NewUUID(), []byte(msg)))
}

func (bus *EventBus) RegisterEventHandler(event string, async bool, fn func(msg *message.Message)) {
	bus.mu.Lock()
	defer bus.mu.Unlock()

	for i := range bus.Event {
		if bus.Event[i].EventType == event {
			bus.Event[i].handlers = append(bus.Event[i].handlers, CallFn{async: async, fn: fn})
			return
		}
	}

	id, err := uuid.NewUUID()
	if err != nil {
		log.Printf("UUID error: %v\n", err)
		return
	}

	newEventType := &EventType{
		UUID:      id,
		EventType: event,
		handlers:  []CallFn{{fn: fn, async: false}},
	}
	bus.Event = append(bus.Event, newEventType)
	log.Println("Subscribed to event: " + event)
	bus.startSubscription(event)
}

func (bus *EventBus) RegisterHandler(async bool, fn func(msg *message.Message)) {
	bus.handlers = append(bus.handlers, CallFn{
		async: async,
		fn:    fn,
	})
}
