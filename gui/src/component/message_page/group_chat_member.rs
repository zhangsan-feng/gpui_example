use std::rc::Rc;
use gpui::{div, px, rgb, size, AsyncApp, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Render, Styled, Window};
use gpui_component::{h_flex, v_flex, v_virtual_list, Sizable, VirtualListScrollHandle};
use gpui_component::avatar::Avatar;
use gpui_component::divider::Divider;
use gpui_component::label::Label;
use gpui_component::menu::{ContextMenuExt, PopupMenu, PopupMenuItem};
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use reqwest::multipart;
use crate::component::{GroupMembers, User};
use crate::component::message_page::MessagePage;
use crate::component::rgb_to_u32;
use crate::service::http_request::HttpClient;
use crate::state::{EventBus, GlobalState};


pub struct GroupMemberEntity{
    pub group_type:String,
    pub group_users:Vec<GroupMembers>,
    scroll_handle:VirtualListScrollHandle,
    select_user_id:String,
}

impl GroupMemberEntity {
    pub fn new( window: &mut Window, cx: &mut Context<Self>)->Self{
        GroupMemberEntity{
            group_type: "".to_string(),
            group_users: vec![],
            scroll_handle: VirtualListScrollHandle::new(),
            select_user_id: Default::default(),
        }
    }
}

impl Render for GroupMemberEntity {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity_handle = cx.entity().clone();
        
        if self.group_users.len() != 0 && self.group_type == "private_chat"{
            div().into_any_element()
        }else{
            v_flex()
                .id("group_member")
                .mb_2()
                .h_full()
                .w(px(240.))
                .child(
                    v_flex()
                        .size_full()
                        .child(
                            div()
                                .h(px(150.))
                                .child(Label::new("群公告").p_2())
                        )
                        .child(Divider::horizontal().w_full())
                        .child(Label::new("群成员").p_2())
                        .child(Divider::horizontal().w_full())

                        .child(
                            v_flex()
                                .gap_2()
                                .size_full()
                                .child(
                                    v_virtual_list(
                                        cx.entity().clone(),
                                        "group_member_component_vm_list",
                                        Rc::new(self.group_users.iter().map(|_| size(px(200.), px(40.))).collect()),
                                        |view, visible_range, _, cx| {
                                            visible_range
                                                .map(|index| {

                                                    let group_user = view.group_users[index].clone();
                                                    h_flex()
                                                        .on_mouse_down(MouseButton::Right, cx.listener(move |this, _, _, cx|{
                                                            this.select_user_id = group_user.id.clone();
                                                        }))
                                                        .size_full()
                                                        .id(("group-member-component", index))
                                                        .rounded(px(4.0))
                                                        .size_full()
                                                        .hover(|mut style|{
                                                            style.background = Some(rgb(rgb_to_u32(235, 235, 235)).into());
                                                            style
                                                        })
                                                        .rounded(px(5.0))
                                                        .child(Avatar::new().src(group_user.avatar).with_size(px(30.)))
                                                        .child(Label::new(group_user.name.chars().take(10).collect::<String>()))
                                                        .child(div().flex_grow())
                                                        .child(Label::new(group_user.user_type))
                                                })
                                                .collect()
                                        },
                                    )
                                        .track_scroll(&self.scroll_handle)
                                        .w(px(165.))

                                )
                                .child(
                                    Scrollbar::vertical(&self.scroll_handle)
                                        .scrollbar_show(ScrollbarShow::Always)
                                        .axis(ScrollbarAxis::Vertical),
                                )

                        )
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
                .into_any_element()
        }
    }
}
