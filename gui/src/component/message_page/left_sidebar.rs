use std::collections::HashMap;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use gpui::*;
use std::rc::Rc;

use crate::component::message_page::MessagePage;
use crate::component::message_page::search_group_and_user_window::SearchGroupAndUserWindow;

use gpui_component::avatar::Avatar;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::label::Label;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::{Root, Sizable, StyledExt, h_flex, v_flex, v_virtual_list, VirtualListScrollHandle};
use gpui_component::menu::{ContextMenuExt, DropdownMenu, PopupMenu, PopupMenuItem};
use crate::component::message_page::create_group_chat_window::CreateGroupChatWindow;

use crate::component::rgb_to_u32;
use crate::state::GlobalState;

pub fn window_center_options( window: &mut Window, w:f32, h:f32) -> WindowOptions {
    let parent_bounds = window.bounds();
    let parent_x = parent_bounds.origin.x;
    let parent_y = parent_bounds.origin.y;

    let parent_width = parent_bounds.size.width;
    let parent_height = parent_bounds.size.height;

    let child_x = parent_x + (parent_width - px(w)) / 2.0;
    let child_y = parent_y + (parent_height - px(h)) / 2.0;
    let mut window_options = WindowOptions::default();
    let window_size = size(px(w), px(h));

    let bounds = Bounds {
        origin: Point { x: child_x, y:child_y },
        size:window_size,
    };
    window_options.window_bounds = Some(WindowBounds::Windowed(bounds));

    window_options.window_min_size = Some(window_size);
    window_options.is_resizable = false;
    window_options.titlebar = Some(TitlebarOptions {
        title: Some(SharedString::from("")),
        appears_transparent: false,
        ..Default::default()
    });
    window_options
}


impl MessagePage {
    fn left_sidebar_vm_list(&self, cx: &mut Context<Self>)->impl IntoElement{
        v_virtual_list(
            cx.entity().clone(),
            "left_sidebar_vm_list",
            Rc::new(self.message_group.iter().map(|_| size(px(200.), px(70.))).collect()),
            |this, visible_range, window, cx| {
                visible_range
                    .map(|vm_index| {
                        let global_state = cx.global::<GlobalState>().0.clone().read(cx);
                        let index = this.select_index.clone();
                        let group = this.message_group[vm_index].clone();
                        let mut group_avatar = group.avatar.clone();

                        let name = group.name.clone();

                        let max_chars = 15;
                        let mut group_name = format!("{}...", name.chars().take(max_chars).collect::<String>());

                        let group_id = group.id.clone();
                        let last_message = group.history.last().cloned().unwrap_or_default();

                        let mut last_message_ui = format!(
                            "{}:{}",
                            last_message.send_username,
                            last_message.message.replace("\n", "").chars().take(max_chars).collect::<String>()
                        );
                        if last_message_ui == ":" {
                            last_message_ui = "".to_string();
                        }

                        let mut message_len = group.history.len();
                        if message_len > 99 {
                            message_len = 99
                        }
                        let message_len = message_len.to_string();

                        let time_str = &last_message.time;

                        let message_time = if let Ok(naive_dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
                            let dt: DateTime<Local> = Local.from_local_datetime(&naive_dt).unwrap();
                            let now = Local::now();
                            if dt.date_naive() == now.date_naive() {
                                dt.format("%H:%M").to_string()
                            } else {
                                dt.format("%m-%d").to_string()
                            }
                        } else {
                            time_str.clone()
                        };

                        if group.group_type == "private_chat"{
                            group.members.iter().for_each(|x|{
                                if x.id != global_state.user_state.user_id{
                                    group_avatar = x.avatar.clone();
                                    group_name = format!("{}...", x.name.clone().chars().take(max_chars).collect::<String>())
                                }
                            })
                        }


                        h_flex()
                            .id(("message-group-vm-list", vm_index))
                            .size_full()
                            .hover(|mut style| {
                                if index != vm_index {
                                    style.background =
                                        Some(rgb(rgb_to_u32(226, 226, 226)).into());
                                }
                                style
                            })
                            .rounded(px(12.))
                            .on_mouse_down(MouseButton::Right, cx.listener(|this, _, _, cx|{

                            }))
                            .on_click({
                                cx.listener({
                                    move |this, _, _, cx| {
                                        this.select_index = vm_index.clone();
                                        let group_data = this.message_group[this.select_index].clone();

                                        if !this.click_column.contains_key(&group_data.id.to_string()) {
                                            this.click_column.insert(group_data.id.to_string(), vm_index);
                                        }
                                        let group_id = this.message_group[this.select_index].id.clone();

                                        this.sned_message_entity.update(cx, |this, cx| {
                                            this.group_id = group_id.clone();
                                        });

                                        this.history_message_entity.update(cx, |this, cx|{
                                            this.history_message = group_data.history.clone();
                                            this.scroll_handle.reset(group_data.history.len());
                                            cx.notify();
                                        });

                                        this.group_members_entity.update(cx, |this, cx|{
                                            this.group_users = group_data.members.clone();
                                            this.group_type = group_data.group_type.clone();
                                            cx.notify();
                                        });
                                    }
                                })
                            })

                            .bg(if index == vm_index {
                                rgb(rgb_to_u32(226, 226, 226))
                            } else {
                                rgb(rgb_to_u32(255, 255, 255))
                            })
                            .child(
                                h_flex()
                                    .size_full()
                                    .p_2()
                                    .child(
                                        Avatar::new()
                                            .src(group_avatar)
                                            .with_size(gpui_component::Size::Size(px(
                                                50.0,
                                            ))),
                                    )
                                    .child(
                                        v_flex()
                                            .mr_4()
                                            .size_full()
                                            .child(
                                                h_flex()
                                                    .child(
                                                        div()
                                                            .w(px(200.))
                                                            .child(group_name),
                                                    )
                                                    .child(div().flex_grow())
                                                    .child(message_time),
                                            )
                                            .child(
                                                h_flex()
                                                    .child(last_message_ui)
                                                    .child(div().flex_grow())
                                                    .child(
                                                        if this.click_column.contains_key(&group_id.to_string())
                                                            || message_len == "0" || this.select_index == vm_index {
                                                            div()
                                                        } else {
                                                            div()

                                                                .paddings(px(2.0))
                                                                .text_center()
                                                                .child(Label::new(
                                                                    message_len,
                                                                ))
                                                                .bg(rgb(rgb_to_u32(
                                                                    252, 90, 78,
                                                                )))
                                                                .rounded_full()
                                                        },
                                                    ),
                                            ),
                                    ),
                            )
                    })
                    .collect()
            }
        )
            .h_full()
            .track_scroll(&self.message_group_scroll_handle)
    }

    fn add_btn_menu(&self, cx: &mut Context<Self>) ->impl IntoElement{
        let entity_handle = cx.entity().clone();
        Button::new("left-sidebar-btn-menu")
            .label("+")
            .mx_2()
            .dropdown_menu_with_anchor(Corner::TopLeft, move |menu, window, cx| {
                menu.item(
                    PopupMenuItem::new("创建群聊")
                        .on_click(window.listener_for(&entity_handle.clone(), |this, event, window, cx|{
                            let global_state = cx.global::<GlobalState>().0.clone();
                            if global_state.read(cx).dial_window_is_open{
                                return
                            }

                            let _ = cx.open_window(window_center_options(window, 350., 300.), move |window, app| {
                                let view = app.new(|cx| CreateGroupChatWindow::new(cx, window));
                                app.new(|cx| Root::new(view, window, cx))
                            });

                            global_state.update(cx, |this, cx|{
                                this.dial_window_is_open = true;
                            })
                        }))

                )
                    .item(
                        PopupMenuItem::new("添加好友")
                            .on_click(window.listener_for(& entity_handle, |this, event, window, cx|{
                                let global_state = cx.global::<GlobalState>().0.clone();
                                if global_state.read(cx).dial_window_is_open{
                                    return
                                }

                                let _ = cx.open_window(window_center_options(window, 600.,600.), move |window, app| {
                                    let view = app.new(|cx| SearchGroupAndUserWindow::new(cx, window));
                                    app.new(|cx| Root::new(view, window, cx))
                                });

                                global_state.update(cx, |this, cx|{
                                    this.dial_window_is_open = true;
                                })
                            }))
                    )


            })
    }

    pub(crate) fn left_sidebar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity_handle = cx.entity().clone();
        v_flex()
            .size_full()
            .w(px(self.left_panel_default_width))
            .child(
                h_flex()
                    .p_2()
                    .child(Input::new(&self.search_input).w(px(235.)))
                    .child(
                        self.add_btn_menu(cx)
                    ),
            )
            .child(
                h_flex()
                    .size_full()
                    .child(
                        self.left_sidebar_vm_list(cx)
                    )
                    .child(
                        Scrollbar::vertical(&self.message_group_scroll_handle)
                            .scrollbar_show(ScrollbarShow::Always)
                            .axis(ScrollbarAxis::Vertical),
                    )
            )
            .context_menu(move |menu: PopupMenu, w: &mut Window, _cx: &mut Context<PopupMenu>| {
                    menu
                        .item(PopupMenuItem::new("置顶").on_click(w.listener_for(
                        &entity_handle,
                          |this, _, _, cx| {

                        },
                        )))
                        .item(PopupMenuItem::new("复制群号").on_click(w.listener_for(
                            &entity_handle,
                              |this, _, _, cx| {

                            },
                        )))
                        .item(PopupMenuItem::new("删除消息").on_click(w.listener_for(
                            &entity_handle,
                              |this, _, _, cx| {

                            },
                        )))
                        .submenu("群消息设置 >", w, _cx, |submenu, window, cx| {
                            submenu
                                .item(PopupMenuItem::new("接收但不提醒"))
                                .item(PopupMenuItem::new("屏蔽消息"))
                        })
                },
            )

    }
}
