use std::rc::Rc;
use gpui::*;
use gpui_component::{h_flex, v_flex, v_virtual_list, Sizable, VirtualListScrollHandle};
use gpui_component::avatar::Avatar;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::label::Label;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::tab::{Tab, TabBar};
use log::info;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use crate::component::{rgb_to_u32, MessageGroup, User};
use crate::entity;
use crate::service::http_request::HttpClient;
use crate::state::GlobalState;




pub struct SearchGroupAndUserWindow {
    search_input:Entity<InputState>,
    scroll_handler:VirtualListScrollHandle,
    search_result:entity::search_entity::SearchResult,
    select_index:usize,

}


impl SearchGroupAndUserWindow {
    pub fn new(cx: &mut Context<Self>, window: &mut Window) -> Self {
        Self {
            search_input:cx.new(|cx| InputState::new(window, cx)),
            scroll_handler: VirtualListScrollHandle::new(),
            search_result: entity::search_entity::SearchResult { groups: vec![], users: vec![] },
            select_index: 0,
        }
    }

    fn search_button(&self,  cx: &mut Context<Self>)-> impl IntoElement{
        Button::new("search-window-btn")
            .label("搜索")
            .items_center()
            .justify_center()

            .on_click({
                cx.listener({
                    |this, _, window, cx|{
                        let search_input = this.search_input.read(cx).text().clone();
                        let global_state = cx.global::<GlobalState>().0.read(cx).clone();
                        let address = global_state.http_server.clone();
                        let form = multipart::Form::new().text("keyword", search_input)
                            .text("user_id", global_state.user_state.user_id);
                        let mut cx_async = cx.to_async();
                        let entity = cx.entity().clone();

                        cx.spawn(move |_, _: &mut AsyncApp| async move {
                            let res = global_state.tokio_handle.spawn(async move {
                                let response = HttpClient::new().post_form(format!("{}/search_group_and_user", address), form).await;
                                response
                            });

                            match res.await {
                                Ok(Ok(r)) => {
                    
                                    let r:entity::search_entity::SearchResult = serde_json::from_value(r.data).unwrap_or_default();
                                    entity.update(&mut cx_async, |this, cx|{
                                        this.search_result = r;
                                        cx.notify()
                                    })

                                }
                                Ok(Err(e)) => println!("http error: {:?}", e),
                                Err(e) => println!("tokio runtime error: {:?}", e),
                            }
                        }).detach()
                    }
                })
            })
    }


    fn add_group(&self,  cx: &mut Context<Self>, index:usize, group_id:String)-> impl IntoElement{

        Button::new(("search-window-group-add", index))
            .label("加入")
            .on_click({
                let group_id = group_id.clone();
                cx.listener({
                    move |this, _, window, cx|{
                        let group_id = group_id.clone();
                        let global_state = cx.global::<GlobalState>().0.read(cx).clone();
                        let address = global_state.http_server.clone();
                        let form = multipart::Form::new()
                            .text("user_id", global_state.user_state.user_id)
                            .text("group_id", group_id);
                        let window_handler = window.window_handle();
                        let mut cx_async = cx.to_async();
                        cx.spawn(move |_, _: &mut AsyncApp| async move {
                            let res = global_state.tokio_handle.spawn(async move {
                                let response = HttpClient::new().post_form(format!("{}/join_group_chat", address), form).await;
                                response
                            });

                            match res.await {
                                Ok(Ok(r)) => {
                                    println!("{}", r.data);
                                    window_handler.update(&mut cx_async, |_,window,app|{
                                        window.remove_window();
                                    }).expect("close window filed");
                                }
                                Ok(Err(e)) => println!("http error: {:?}", e),
                                Err(e) => println!("tokio runtime error: {:?}", e),
                            }
                        }).detach()
                    }
                })
            })
    }

    fn add_friend(&self,  cx: &mut Context<Self>, index:usize, friend_id:String)-> impl IntoElement{

        Button::new(("search-window-friend-add", index))
            .label("添加")
            .on_click({
                let friend_id = friend_id.clone();
                cx.listener({
                    move |this, _, window, cx|{
                        let friend_id = friend_id.clone();
                        let global_state = cx.global::<GlobalState>().0.read(cx).clone();
                        let address = global_state.http_server.clone();
                        let form = multipart::Form::new()
                            .text("self_user_id", global_state.user_state.user_id)
                            .text("friend_user_id", friend_id);
                        let window_handler = window.window_handle();
                        let mut cx_async = cx.to_async();
                        cx.spawn(move |_, _: &mut AsyncApp| async move {
                            let res = global_state.tokio_handle.spawn(async move {
                                let response = HttpClient::new().post_form(format!("{}/friends/add", address), form).await;
                                response
                            });

                            match res.await {
                                Ok(Ok(r)) => {
                                    println!("{}", r.data);
                                    window_handler.update(&mut cx_async, |_,window,app|{
                                        window.remove_window();
                                    }).expect("close window filed");
                                }
                                Ok(Err(e)) => println!("http error: {:?}", e),
                                Err(e) => println!("tokio runtime error: {:?}", e),
                            }
                        }).detach()
                    }
                })
            })
    }

    fn search_result_group_vm_list(&self, cx: &mut Context<Self>) -> impl IntoElement{
        v_virtual_list(
            cx.entity().clone(),
            "search-window-vm-list",
            Rc::new(self.search_result.groups.iter().map(|_| size(px(200.), px(50.))).collect()),
            |this, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let group = this.search_result.groups[index].clone();
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()

                            .child(
                                h_flex()
                                    .gap(px(10.))
                                    .items_center()
                                    .child(
                                        Avatar::new()
                                            .src(group.avatar.clone())
                                            .size(px(40.))
                                    )
                                    .child(
                                        Label::new(group.name.clone())
                                    )
                            )
                            .child(
                                this.add_group(cx, index, group.id.clone())
                                    .into_any_element()

                            )
                    })
                    .collect()
            },
        ).track_scroll(&self.scroll_handler)
    }

    fn search_result_user_vm_list(&self, cx: &mut Context<Self>) -> impl IntoElement{
        v_virtual_list(
            cx.entity().clone(),
            "search-window-vm-list",
            Rc::new(self.search_result.users.iter().map(|_| size(px(200.), px(50.))).collect()),
            |this, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let user = this.search_result.users[index].clone();
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()

                            .child(
                                h_flex()
                                    .gap(px(10.))
                                    .items_center()
                                    .child(
                                        Avatar::new()
                                            .src(user.avatar.clone())
                                            .size(px(40.))
                                    )
                                    .child(
                                        Label::new(user.name.clone())
                                    )

                            )
                            .child(
                                this.add_friend(cx, index, user.id.clone())
                                    .into_any_element()
                            )
                    })
                    .collect()
            },
        ).track_scroll(&self.scroll_handler)
    }

}

impl Render for SearchGroupAndUserWindow {



    fn render(&mut self,  window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.on_window_closed(|window|{
            window.update_global(|global_state:&mut GlobalState, app|{
                global_state.0.update(app, |this, cx|{
                    this.dial_window_is_open = false;
                });
            })
        }).detach();

        v_flex()
            .items_center()
            .size_full()
            .p_4()
            .child(
                h_flex()
                    .gap(px(10.))
                    .child(
                        Input::new(&self.search_input)
                            .w(px(300.))
                            .h(px(50.))
                    )
                    .child(
                        self.search_button(cx)
                    )
            )
            .child(
                v_flex()
                    .size_full()
                    .items_center()
                    .mt_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                               div()
                                   .p_2()
                                   .rounded(px(10.))
                                   .id("search-result-group-btn")
                                   .child("群聊")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, event, window, cx|{
                                        this.select_index  = 0;
                                        cx.notify()
                                    }))
                                   .bg(if self.select_index == 0 {
                                       rgb(rgb_to_u32(226, 226, 226))
                                   } else {
                                       rgb(rgb_to_u32(255, 255, 255))
                                   })
                                   .hover(|mut style| {
                                       if self.select_index != 0 {
                                           style.background =
                                               Some(rgb(rgb_to_u32(226, 226, 226)).into());
                                       }
                                       style
                                   })
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded(px(10.))
                                    .id("search-result-user-btn")
                                    .child("用户")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, event, window, cx|{
                                        this.select_index  = 1;
                                        cx.notify()
                                    }))
                                    .bg(if self.select_index == 1 {
                                        rgb(rgb_to_u32(226, 226, 226))
                                    } else {
                                        rgb(rgb_to_u32(255, 255, 255))
                                    })
                                    .hover(|mut style| {
                                        if self.select_index != 1 {
                                            style.background =
                                                Some(rgb(rgb_to_u32(226, 226, 226)).into());
                                        }
                                        style
                                    })
                            )
                    )
                    .child(
                        if self.select_index == 0 {
                            self.search_result_group_vm_list(cx).into_any_element()
                        } else if self.select_index == 1{
                            self.search_result_user_vm_list(cx).into_any_element()
                        } else{
                            div().into_any_element()
                        }
                    )
                    .child(
                        Scrollbar::vertical(&self.scroll_handler)
                            .scrollbar_show(ScrollbarShow::Always)
                            .axis(ScrollbarAxis::Vertical),
                    )

                // self.search_result_group_vm_list(cx)
            )

    }
}