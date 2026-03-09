use std::rc::Rc;
use gpui::*;
use gpui_component::divider::Divider;
use gpui_component::{h_flex, v_flex, v_virtual_list, VirtualListScrollHandle};
use gpui_component::avatar::Avatar;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::label::Label;
use gpui_component::menu::{ContextMenuExt, PopupMenu, PopupMenuItem};
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use log::{error, info};
use reqwest::multipart;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use crate::component::{rgb_to_u32, GroupHistory, MessageGroup, User, WsMsgEvent};
use crate::service::http_request::HttpClient;
use crate::state::{EventBus, GlobalState};

pub struct FriendPage{
    user_notice:String,
    group_notice: String,
    friends: Vec<User>,
    groups:Vec<MessageGroup>,
    scroll_handler:VirtualListScrollHandle,
    search_input: Entity<InputState>,
    select_user_id:String,
}

impl FriendPage{
    pub fn new(cx: &mut Context<Self>, window: &mut Window) -> Self {
        FriendPage{
            user_notice: String::new(),
            group_notice:String::new(),
            friends: vec![],
            groups: vec![],
            scroll_handler: VirtualListScrollHandle::new(),
            search_input: cx.new(|cx| InputState::new(window, cx)),
            select_user_id: Default::default(),
        }
    }
    pub fn init_component_data(&mut self, data:Vec<User>, cx: &mut Context<Self>){
        info!("{:?}", data);
        self.friends = data.clone();
    }

    pub fn update_component_data(&mut self, event: WsMsgEvent, cx: &mut Context<Self>){
        match event.msg_type.as_str() {
            "add_friend"=>{
                match serde_json::from_value::<User>(event.data) {
                    Ok(data) => {
                        info!("{:?}", data);
                        self.friends.push(data);
                    }
                    Err(e) => {
                        info!("Failed to parse data as GroupHistory: {}", e);
                    }
                }
            }
            _=>{}
        }
    }
}

impl Render for FriendPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity_handle = cx.entity();
        h_flex()
            .text_size(px(14.))
            .size_full()
            .child(
                v_flex()
                    .gap_2()
                    .items_start()
                    .justify_start()
                    .h_full()
                    .p_2()
                    .w(px(320.))
                    .child(
                        h_flex()
                            .child(Input::new(&self.search_input).w(px(235.)))
                            .child(
                               Button::new("friend-page-search-btn").label("+").ml_2()
                            ),
                    )
                    .child(
                        h_flex()
                            .id("friend-page-friends-notice")
                            .p_2()
                            .rounded_l(px(5.0))
                            .w_full()
                            .justify_between()
                            .child(
                                div().child("好友通知")
                            )
                            .child(">")
                            .hover(|mut style| {
                                style.background = Some(rgb(rgb_to_u32(226, 226, 226)).into());
                                style
                            })
                    )
                    .child(
                        h_flex()
                            .id("friend-page-groups-notice")
                            .p_2()
                            .rounded_l(px(5.0))
                            .w_full()
                            .justify_between()
                            .child(
                                div().child("群通知")
                            )
                            .child(">")
                            .hover(|mut style| {
                                style.background = Some(rgb(rgb_to_u32(226, 226, 226)).into());
                                style
                            })
                    )

                    .child(
                        h_flex()
                            .w_full()
                            .p_1()
                            .justify_between()
                            .gap_2()
                            .bg(rgb(rgb_to_u32(235, 235, 235)))
                            .child(
                                div()
                                    .flex()
                                    .rounded_l(px(5.0))
                                    .p_1()
                                    .items_center()
                                    .justify_center()
                                    .id("friend-page-friends-btn-id")
                                    .bg(rgb(rgb_to_u32(255, 255, 255)))
                                    .child("好友")
                                    .flex_1()
                                    .justify_center()
                            )
                            .child(
                                div()
                                    .flex()
                                    .rounded_l(px(5.0))
                                    .p_1()
                                    .items_center()
                                    .justify_center()
                                    .id("friend-page-groups-btn-id")
                                    .child("群聊")
                                    .flex_1()
                                    .w(px(120.))
                                    .justify_center()
                            )
                    )

                    .child(
                        v_virtual_list(
                            cx.entity().clone(),
                            "search-window-vm-list",
                            Rc::new(self.friends.iter().map(|_| size(px(200.), px(50.))).collect()),
                            |this, visible_range, _, cx| {
                                visible_range
                                    .map(|index| {
                                        let user = this.friends[index].clone();
                                        h_flex()
                                            .rounded(px(5.0))
                                            .p_2()
                                            .gap_2()
                                            .w_full()
                                            .id(("friend-page-friends-hover-id", index))
                                            .on_mouse_down(MouseButton::Right, cx.listener(move |this, _, _, cx|{
                                                this.select_user_id = user.id.clone();
                                            }))
                                            .hover(|mut style| {
                                                style.background = Some(rgb(rgb_to_u32(226, 226, 226)).into());
                                                style
                                            })
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .child(
                                                        Avatar::new()
                                                            .src(user.avatar.clone())
                                                            .size(px(40.))
                                                    )
                                                    .child(Label::new(user.name.chars().take(15).collect::<String>()))
                                            )

                                    })
                                    .collect()
                            },
                        )
                            .track_scroll(&self.scroll_handler)
                    )
                    .child(
                        Scrollbar::vertical(&self.scroll_handler)
                            .scrollbar_show(ScrollbarShow::Always)
                            .axis(ScrollbarAxis::Vertical),
                    )
            )
            .child(Divider::vertical().h_full())
            .child(
                div().flex_grow()
            )
            .context_menu(move |menu: PopupMenu, w: &mut Window, _cx: &mut Context<PopupMenu>| {
                menu.item(PopupMenuItem::new("发消息").on_click(w.listener_for(
                    &entity_handle,
                    |this, _, _, cx| {
                        let global_state = cx.global::<GlobalState>().0.clone();
                        let global_state_read = global_state.read(cx).clone();
                        let address = global_state_read.http_server;
                        let user_id = this.select_user_id.clone();
                        cx.spawn(move |_, _: &mut AsyncApp| async move {
                            let res = global_state_read.tokio_handle.spawn(async move{
                                let form = multipart::Form::new()
                                    .text("user_id", global_state_read.user_state.user_id.to_string())
                                    .text("user_id", user_id)
                                    .text("type", "private_chat");
                                let response = HttpClient::new().post_form(format!("{}/create_group_chat", address), form).await;
                                response
                            });
                            match res.await {
                                Ok(Ok(r)) => {
                                }
                                Ok(Err(e)) => println!("http error: {:?}", e),
                                Err(e) => println!("tokio runtime error: {:?}", e),
                            }
                        }).detach();

                        global_state.update(cx,|this, cx|{
                            cx.emit(EventBus::ChildrenChangeSelectIndex)
                        });
                    },
                )))
                    .item(PopupMenuItem::new("查看资料").on_click(w.listener_for(
                        &entity_handle,
                          |this, _, _, cx| {

                        },
                    )))
            })
    }
}