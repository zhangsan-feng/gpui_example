use std::rc::Rc;
use gpui::{div, px, rgb, size, Context, InteractiveElement, IntoElement, ParentElement, Styled, Window};
use gpui_component::{h_flex, v_flex, v_virtual_list, Sizable};
use gpui_component::avatar::Avatar;
use gpui_component::divider::Divider;
use gpui_component::label::Label;
use gpui_component::menu::{ContextMenuExt, PopupMenu, PopupMenuItem};
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use crate::component::message_page::MessagePage;
use crate::component::rgb_to_u32;
use crate::state::GlobalState;




impl MessagePage {
    pub fn group_chat_member(&self,  window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity_handle = cx.entity().clone();


        if self.message_group.len() != 0 && self.message_group[self.select_index].group_type == "private"{
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
                        .child(

                            v_virtual_list(
                                cx.entity().clone(),
                                "group_member_component_vm_list",
                                Rc::new(
                                    self
                                        .message_group
                                        .get(self.select_index)
                                        .map(|group| {
                                            group
                                                .members
                                                .iter()
                                                .map(|_| {
                                                    let base_height = 50.0;
                                                    let total_height = base_height;
                                                    size(px(200.), px(total_height))
                                                })
                                                .collect::<Vec<_>>()
                                        })
                                        .unwrap_or_default()
                                ),
                                |view, visible_range, _, cx| {
                                    visible_range
                                        .map(|index| {

                                            let group_user = view.message_group[view.select_index].members[index].clone();
                                            h_flex()
                                                .id(("group-member-component", index))
                                                .rounded(px(4.0))
                                                .p_2()
                                                .size_full()
                                                .hover(|mut style|{
                                                    style.background = Some(rgb(rgb_to_u32(235, 235, 235)).into());
                                                    style
                                                })
                                                .child(Avatar::new().src(group_user.avatar).with_size(px(32.)))
                                                .child(Label::new(group_user.name))
                                        })
                                        .collect()
                                },
                            ).track_scroll(&self.group_member_scroll_handle)

                        )
                        .child(
                            Scrollbar::vertical(&self.group_member_scroll_handle)
                                .scrollbar_show(ScrollbarShow::Always)
                                .axis(ScrollbarAxis::Vertical),
                        )
                )
                .context_menu(move |menu: PopupMenu, w: &mut Window, _cx: &mut Context<PopupMenu>| {
                    menu.item(PopupMenuItem::new("发消息").on_click(w.listener_for(
                        &entity_handle,
                        |this, _ev, _w, _cx| {

                        },
                    )))
                        .item(PopupMenuItem::new("查看资料").on_click(w.listener_for(
                            &entity_handle,
                            |this, _ev, _w, _cx| {

                            },
                        )))
                },
                )
                .into_any_element()
        }


    }
}