use std::collections::HashMap;
use std::path::PathBuf;
use gpui::*;
use gpui_component::input::{InputState};
use gpui_component::*;
use gpui_component::divider::Divider;
use log::info;
use serde::{Serialize};
use crate::component::message_page::entity::{GroupHistory, GroupMembers, MessageGroup, WsMsgEvent};
use crate::component::message_page::send_message_entity::SendMessageEntity;
use crate::component::rgb_to_u32;
use crate::service::http_request::RestResponse;
use crate::state::{ChatMessage, GlobalState};


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
    // history_message_scroll_handle: VirtualListScrollHandle,
    history_message_scroll_handle: gpui::ListState,
    group_member_scroll_handle: VirtualListScrollHandle,
    search_input: Entity<InputState>,

    click_column:HashMap<String, usize>,
    left_panel_default_width:f32,
    left_panel_min_width:f32,
    left_panel_max_width:f32,

    history_message_panel_min_height:f32,
    history_message_panel_max_height:f32,
    sned_message_component:Entity<SendMessageEntity>,
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
        let mut  s = MessagePage {
            select_index: 0,
            message_group: Vec::new(),
            message_group_scroll_handle: VirtualListScrollHandle::new(),
            history_message_scroll_handle: ListState::new(0, ListAlignment::Bottom, px(100.)),
            group_member_scroll_handle: VirtualListScrollHandle::new(),
            search_input:cx.new(|cx| InputState::new(window, cx)),
            click_column:HashMap::new(),
            left_panel_default_width: 330.0,
            left_panel_min_width: 200.0,
            left_panel_max_width: 330.0,
            history_message_panel_min_height: 140.0,
            history_message_panel_max_height: 500.0,
            sned_message_component:cx.new(|cx|{SendMessageEntity::new(window, cx)}),
        };
        if s.message_group.len() > 0 {
            s.click_column.insert(s.message_group[s.select_index].id.clone(), 0);
        }


        let state_handle = cx.global::<GlobalState>().0.clone();
        cx.subscribe(&state_handle, |this: &mut Self, _model, event: &ChatMessage, cx| {
            match event {
                ChatMessage::WebSocketText(txt)=>{
                    println!("组件收到消息: {}", txt);

                    match serde_json::from_str::<WsMsgEvent>(&txt) {
                        Ok(event) => {
                            match event.msg_type.as_str() {
                                "message" => {
                                    match serde_json::from_value::<GroupHistory>(event.data) {
                                        Ok(history) => {
                                            println!("Got GroupHistory: {:?}", history);
                                            this.message_group[this.select_index].history.push(history);
                                            this.history_message_scroll_handle.reset(this.message_group[this.select_index].history.len());
                                            if this.select_index != 0 {
                                                let item = this.message_group.remove(this.select_index);
                                                this.message_group.insert(0, item);
                                                this.select_index = 0;
                                                // this.select_index = item.id;
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
                                            let exists = this.message_group.iter().any(|msg| msg.id == group_id);

                                            if !exists {
                                                this.message_group.insert(0, data);
                                                this.select_index = 0;
                                                this.sned_message_component.update(cx, |this, cx| {
                                                    this.group_id = group_id;
                                                });
                                            }

                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse data as GroupHistory: {}", e);
                                        }
                                    }
                                }
                                "other_join_group_chat" => {
                                    match serde_json::from_value::<Vec<GroupMembers>>(event.data) {
                                        Ok(members) => {
                                            this.message_group[this.select_index].members = members;
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
                                            let exists = this.message_group.iter().any(|msg| msg.id == group_id);

                                            if !exists {
                                                this.message_group.insert(0, data);
                                                this.select_index = 0;
                                                this.sned_message_component.update(cx, |this, cx| {
                                                    this.group_id = group_id;
                                                });
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

                let result: Result<Vec<MessageGroup>, anyhow::Error> = {
                    let response = reqwest::get(format!("{}/user_message_group?user_id={}", address, user_id)).await
                        .map_err(|e| anyhow::anyhow!("请求失败: {}", e))?;
                    if !response.status().is_success() {
                        let status = response.status();
                        let bytes = response.bytes().await.unwrap_or_default();
                        let body_str = String::from_utf8_lossy(&bytes);
                        return Err(anyhow::anyhow!("HTTP 请求失败，状态码: {}, 响应体: {}",status,body_str));
                    }

                    let bytes = response.bytes().await.map_err(|e| anyhow::anyhow!("读取响应失败: {}", e))?;
                    let body_str = String::from_utf8_lossy(&bytes);
                    let resp: RestResponse<Vec<MessageGroup>> = serde_json::from_slice(&bytes)
                        .map_err(|e| anyhow::anyhow!("JSON 解析失败: {}\n原始内容: {}", e, body_str))?;
                    Ok(resp.data)
                };
                result
            });

            match res.await {
                Ok(Ok(r)) => {
                    entity.update(&mut cx_async, |this, cx| {
                        this.message_group = r.clone();
                        this.sned_message_component.update(cx, |this, cx|{
                            match r.first() {
                                Some(group_index)=>{
                                    this.group_id = group_index.id.clone()
                                }
                                _=>{}
                            }
                        })
                    });
                }
                Ok(Err(e)) => println!("http error: {:?}", e),
                Err(e) => println!("tokio runtime error: {:?}", e),
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
        self.sned_message_component.update(cx, |this, cx|{
            this.history_message_panel_height = f32::from(
                new_height.clamp(
                    px(self.history_message_panel_min_height),
                    px(self.history_message_panel_max_height)
                )
            )
        });


    }

    pub fn pick_files(&self, cx: &mut Context<Self>) {
        let mut async_cx = cx.to_async();
        let entity_view = cx.entity();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let paths = entity_view.update(&mut async_cx, |this, cx| {
                cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: true,
                    prompt: None,
                })
            });

            if let Ok(result) = paths.await {
                if let Ok(Some(path_vec)) = result {
                    info!("{:?}", path_vec);

                    // if let Some(first_path) = path_vec.first() {
                    //     let path_str = first_path.to_string_lossy().to_string();
                    //     entity_view.update(&mut async_cx, |this, cx| {
                    //         // update_this.avatar_path = Some(path_str.replace("\\", "/").into());
                    //     });
                    // }
                }
            }

        }).detach();
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
                                            self.history_message_component(window, cx)
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
                                            self.sned_message_component.clone()
                                        )
                                )
                                .child(Divider::vertical().h_full())
                                .child(
                                    self.group_chat_member(window, cx)
                                )
                        }

                    )
                    .into_any_element()
            )

    }
}
