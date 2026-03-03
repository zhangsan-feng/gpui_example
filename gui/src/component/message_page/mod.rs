use std::collections::HashMap;
use std::path::PathBuf;
use gpui::*;
use gpui_component::input::{InputState};
use gpui_component::*;
use gpui_component::divider::Divider;
use log::{error, info};
use serde::{Serialize};
use crate::component::message_page::entity::{GroupHistory, GroupMembers, MessageGroup, WsMsgEvent};
use crate::component::message_page::group_chat_member::GroupMemberEntity;
use crate::component::message_page::history_message::HistoryMessageEntity;
use crate::component::message_page::send_message_entity::SendMessageEntity;
use crate::component::rgb_to_u32;
use crate::service::http_request;
use crate::service::http_request::RestResponse;
use crate::state::{EventBus, GlobalState};


mod left_sidebar;
mod history_message;
mod group_chat_member;
mod send_message_entity;
mod search_group_and_user_window;
mod create_group_chat_window;
mod entity;


pub struct MessagePage {
    select_index: usize,
    message_group: Vec<MessageGroup>,
    message_group_scroll_handle: VirtualListScrollHandle,

    search_input: Entity<InputState>,

    click_column:HashMap<String, usize>,

    left_panel_default_width:f32,
    left_panel_min_width:f32,
    left_panel_max_width:f32,

    history_message_panel_min_height:f32,
    history_message_panel_max_height:f32,

    sned_message_entity:Entity<SendMessageEntity>,
    history_message_entity: Entity<HistoryMessageEntity>,
    group_members_entity: Entity<GroupMemberEntity>,
}

#[derive(Clone)]
struct LeftPanelResizeHandle;

impl Render for LeftPanelResizeHandle{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone)]
struct HistoryMessagePanelResizeHandle;
impl Render for HistoryMessagePanelResizeHandle{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

impl MessagePage {
    pub fn new(cx: &mut Context<Self>, window: &mut Window) -> Self {
        let mut s = MessagePage {
            select_index: 0,
            message_group: Vec::new(),
            message_group_scroll_handle: VirtualListScrollHandle::new(),
            search_input:cx.new(|cx| InputState::new(window, cx)),
            click_column:HashMap::new(),
            left_panel_default_width: 330.0,
            left_panel_min_width: 200.0,
            left_panel_max_width: 330.0,
            history_message_panel_min_height: 140.0,
            history_message_panel_max_height: 500.0,
            sned_message_entity:cx.new(|cx|{SendMessageEntity::new(window, cx)}),
            history_message_entity:cx.new(|cx|HistoryMessageEntity::new(window, cx)),
            group_members_entity:cx.new(|cx|GroupMemberEntity::new(window, cx)),
        };


        let state_handle = cx.global::<GlobalState>().0.clone();

        cx.subscribe(&state_handle, |this: &mut Self, _model, event: &EventBus, cx| {
            match event {
                EventBus::WebSocketText(txt)=>{
                    println!("组件收到消息: {}", txt);

                    match serde_json::from_str::<WsMsgEvent>(&txt) {
                        Ok(event) => {
                            match event.msg_type.as_str() {
                                "message" => {
                                    match serde_json::from_value::<GroupHistory>(event.data) {
                                        Ok(data) => {
                                            let data = data.clone();
                                            let group_id = data.group_id.clone();
                                            if let Some(index) = this.message_group.iter().position(|x| x.id == group_id) {
                                                this.message_group[index].history.push(data);
                                                let group = this.message_group[index].clone();
                                                if index != 0 {
                                                    let group = this.message_group.remove(index);
                                                    this.message_group.insert(0, group);
                                                    this.select_index = 0;
                                                }

                                                this.history_message_entity.update(cx, |this, cx|{
                                                    this.history_message = group.history.clone();
                                                    this.scroll_handle.reset(group.history.len())
                                                });

                                                // this.history_message_scroll_handle.reset(
                                                //     this.message_group[this.select_index].history.len(),
                                                // );
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse data as GroupHistory: {}", e);
                                        }
                                    }
                                }
                                "create_group_chat"=>{
                                    match serde_json::from_value::<MessageGroup>(event.data) {
                                        Ok(data) => {
                                            let group_id = data.id.clone();
                                            let exist= this.message_group.iter().any(|x| x.id == group_id);
                                            if !exist {
                                                this.message_group.insert(0, data);

                                                this.sned_message_entity.update(cx,|this,cx|{
                                                    this.group_id = group_id.clone();
                                                });
                                                this.select_index = 0;
                                                this.history_message_entity.update(cx, |this, cx|{
                                                    this.history_message = Vec::new();
                                                    this.scroll_handle.reset(0)
                                                });
                                            }

                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse data as GroupHistory: {}", e);
                                        }
                                    }
                                }
                                "other_join_group_chat" => {
                                    match serde_json::from_value::<GroupMembers>(event.data) {
                                        Ok(data) => {
                                            let data = data.clone();
                                            let group_id = data.group_id.clone();
                                            if let Some(index) = this.message_group.iter().position(|x| x.id == group_id) {
                                                this.message_group[index].members.push(data.clone());
                                                this.group_members_entity.update(cx, |this, cx|{
                                                    this.group_users.push(data);
                                                });

                                                // if index != 0 {
                                                //     this.history_message_entity.update(cx, |this, cx|{
                                                //         this.history_message = Vec::new();
                                                //         this.scroll_handle.reset(0)
                                                //     });
                                                //
                                                //     let group = this.message_group.remove(index);
                                                //     this.message_group.insert(0, group);
                                                //     this.select_index = 0;
                                                //
                                                // }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse data as GroupHistory: {}", e);
                                        }
                                    }
                                }
                                "user_join_group_chat"=>{
                                    match serde_json::from_value::<MessageGroup>(event.data) {
                                        Ok(data) => {
                                            let group_id = data.id.clone();
                                            let exist= this.message_group.iter().any(|x| x.id == group_id);
                                            if !exist {
                                                this.message_group.insert(0, data);
                                                this.sned_message_entity.update(cx,|this,cx|{
                                                    this.group_id = group_id.clone();
                                                });
                                                this.select_index = 0;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse data as GroupHistory: {}", e);
                                        }
                                    }
                                }

                                unknown => {
                                    println!("Unknown message type: {}", unknown);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse WsMsgEvent: {} | Raw: {}", e, txt);
                        }
                    }
                }
                _ => {}
            }

        }).detach();


        s.init_message_data(cx);
        s
    }

    pub fn update_component_data(&self){

    }
    pub fn update_data_position(){

    }

    pub fn init_message_data(&self, cx: &mut Context<Self>) {
        let global_state = cx.global::<GlobalState>().0.clone().read(cx);
        let user_id = global_state.user_state.user_id.clone();
        let tokio_handler = global_state.tokio_handle.clone();
        let address = global_state.http_server.clone();

        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();
        // info!("{}", user_id);

        cx.spawn( |_, _: &mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move {
                http_request::HttpClient::new().get(format!("{}/user_message_group?user_id={}", address, user_id)).await
            });
            match res.await {
                Ok(Ok(r)) => {
                    match serde_json::from_value::<Vec<MessageGroup>>(r.data)  {
                        Ok(data)=>{
                            entity.update(&mut cx_async, |this, cx| {
                                this.message_group = data.clone();
                                match data.first() {
                                    Some(first_data)=>{
                                        this.history_message_entity.update(cx, |this, cx|{
                                            this.history_message = first_data.history.clone();
                                            this.scroll_handle.reset(first_data.history.len());
                                        });
                                        this.sned_message_entity.update(cx, |this, cx|{
                                            this.group_id = first_data.id.clone()
                                        });
                                        this.group_members_entity.update(cx, |this, cx|{
                                            this.group_users = first_data.members.clone();
                                            this.group_type = first_data.group_type.clone();
                                        });
                                        this.click_column.insert(first_data.id.clone(), 0);

                                    },
                                    _=>{}
                                }
                            });
                        },
                        Err(e)=>{error!("{}",e)}
                    }
                }
                Ok(Err(e)) => error!("http error: {:?}", e),
                Err(e) => error!("tokio runtime error: {:?}", e),
            }




        }).detach();

    }


    fn left_panel_handle_resize(&mut self, event: &DragMoveEvent<LeftPanelResizeHandle>, cx: &mut Context<Self>) {
        let mouse_x = event.event.position.x;
        let left_offset = px(60.0);
        let new_width = mouse_x - left_offset;
        self.left_panel_default_width = f32::from(
            new_width.clamp(
                px(self.left_panel_min_width),
                px(self.left_panel_max_width)
            )
        );

    }

    fn history_message_handle_resize(&mut self, event: &DragMoveEvent<HistoryMessagePanelResizeHandle>,window: &mut Window, cx: &mut Context<Self>) {
        let mouse_y = event.event.position.y;
        let window_height = window.bounds().size.height;
        let new_height = window_height - mouse_y;

        self.sned_message_entity.update(cx, |this, cx|{
            this.panel_height = f32::from(
                new_height.clamp(
                    px(self.history_message_panel_min_height),
                    px(self.history_message_panel_max_height)
                )
            )
        });
    }
}



impl Render for MessagePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {

        h_flex()
            .size_full()
            .child(
                self.left_sidebar(window, cx),
            )
            .child(
                div()
                    .id("left_resize_handle")
                    .w(px(3.0))
                    .bg(rgb(rgb_to_u32(235, 235, 235)))
                    .h_full()
                    .cursor_col_resize()
                    .active(|s| s.bg(rgb(0x007acc)))
                    .on_drag(LeftPanelResizeHandle, |this, size, window, app| {
                        app.new(|_| this.clone())
                    })
                    .on_drag_move(cx.listener(|this, event, window, cx|{
                        this.left_panel_handle_resize(event, cx);
                    }))
            )
            .child(
                v_flex()
                    .flex_grow()
                    .size_full()
                    .justify_start()
                    .bg(rgb(rgb_to_u32(255, 255, 255)))
                    .child(
                        h_flex()
                            .h(px(60.))
                            .w_full()
                            .p_4()
                            .child(
                                if self.message_group.len() != 0{
                                    self.message_group[self.select_index].name.clone()
                                }else {
                                    "".into()
                                }
                            )
                            .child(div().flex_grow())
                            .child("1111")

                    )
                    .child(Divider::horizontal().w_full())
                    .child(
                        if self.message_group.len() == 0 {
                            div()
                        }else{
                            h_flex()
                                .size_full()
                                .child(
                                    v_flex()
                                        .size_full()
                                        .child(
                                            // self.history_message_component(window, cx)
                                            div().flex_grow()
                                                .child(
                                                    self.history_message_entity.clone()
                                                )
                                        )
                                        .child(
                                            div()
                                                .id("history_message_resize_handle")
                                                .h(px(3.0))
                                                .bg(rgb(rgb_to_u32(235, 235, 235)))
                                                .w_full()
                                                .cursor_row_resize()
                                                .active(|s| s.bg(rgb(0x007acc)))
                                                .on_drag(HistoryMessagePanelResizeHandle, |this, size, window, app| {
                                                    app.new(|_| this.clone())
                                                })
                                                .on_drag_move(cx.listener(|this, event, window, cx|{
                                                    this.history_message_handle_resize(event,window, cx);
                                                }))
                                        )
                                        .child(
                                            self.sned_message_entity.clone()
                                        )
                                )
                                .child(Divider::vertical().h_full())
                                .child(
                                    self.group_members_entity.clone()
                                )
                        }

                    )
                    .into_any_element()
            )

    }
}
