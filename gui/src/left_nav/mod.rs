
use crate::component::home::{ HomeView};

use gpui::{
    Context, InteractiveElement, IntoElement, MouseButton,  ParentElement, Pixels,
    Render, Size, Styled, Window, div, px, rgb, size,
};
use gpui_component::button::Button;
use gpui_component::divider::Divider;
use gpui_component::menu::{ContextMenuExt, PopupMenu, PopupMenuItem};
use gpui_component::resizable::{h_resizable, resizable_panel};
use gpui_component::scroll::*;
use gpui_component::*;
use std::rc::Rc;

use gpui::Entity;

pub struct LeftNav {
    items: Vec<String>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    scroll_handle: VirtualListScrollHandle,
    select_id: String,
    pub parent: Option<Entity<HomeView>>,
}

pub const fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

impl LeftNav {
    pub fn new() -> LeftNav {
        LeftNav {
            scroll_handle: VirtualListScrollHandle::new(),
            item_sizes: Rc::new(Vec::new()),
            items: Vec::new(),
            select_id: String::new(),
            parent: None,
        }
    }

    pub fn set_parent(&mut self, parent: Entity<HomeView>) {
        self.parent = Some(parent);

    }
}

impl Render for LeftNav {
    fn render(&mut self, w: &mut Window, c: &mut Context<Self>) -> impl IntoElement {
        let width = w.bounds().clone().size.width;
        let view = c.entity().clone();

        self.items = (0..5000).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
        self.item_sizes = Rc::new(self.items.iter().map(|_| size(px(200.), px(30.))).collect());

        h_flex()
            .h_full()
            .w_full()
            .items_start()
            .child(
                h_resizable("my-layout")
                    .child(
                        resizable_panel()
                            .size_range(px(f32::from(width * 0.2))..px(f32::from(width * 0.4)))
                            .size(width * 0.2)
                            .child(
                                v_virtual_list(
                                    c.entity().clone(),
                                    "my-list",
                                    self.item_sizes.clone(),
                                    |view, visible_range, _, cx| {
                                        visible_range
                                            .map(|ix| {
                                                Button::new(ix)
                                                    .label(format!("{}", ix))
                                                    .w_full()
                                                    .mt_2()
                                                    .on_click(move |_, _, _| {
                                                        println!("{}", format!("Item {}", ix));
                                                    })
                                                    .on_mouse_down(
                                                        MouseButton::Right,
                                                        cx.listener(move |this, e, window, cx| {
                                                            this.select_id = ix.to_string();
                                                        }),
                                                    )
                                            })
                                            .collect()
                                    },
                                )
                                .text_center()
                                .track_scroll(&self.scroll_handle),
                            )
                            .child(div())
                            .child(
                                // Add scrollbars
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .right_0()
                                    .bottom_0()
                                    .child(
                                        Scrollbar::vertical(&self.scroll_handle)
                                            .scrollbar_show(ScrollbarShow::Always)
                                            .axis(ScrollbarAxis::Vertical),
                                    )
                                    .into_any_element(),
                            ),
                    )
                    .child(
                        v_flex()
                            .child(div().child("123").m_2())
                            .child(Divider::horizontal().w_full())
                            .child(div().child("123").m_2())
                            .bg(rgb(rgb_to_u32(245, 245, 245)))
                            .w_full()
                            .into_any_element(),
                    ),
            )
            .context_menu(
                move |menu: PopupMenu, w: &mut Window, _cx: &mut Context<PopupMenu>| {
                    menu.item(PopupMenuItem::new("新增1").on_click(w.listener_for(
                        &view,
                        |this, _ev, _w, _cx| {

                            // _cx.open_window(WindowOptions::default(), |window, cx| {
                            //     let view = cx.new(|_| session_window::SessionWindow {});
                            //
                            //     cx.new(|cx| Root::new(view, window, cx))
                            // })
                            // .expect("TODO: panic message");
                            // _w.remove_window();
                        },
                    )))
                    .item(PopupMenuItem::new("新增2").on_click(w.listener_for(
                        &view,
                        |this, _ev, _w, _cx| {
                            println!("新增项目 - 当前选中: {:?}", this.select_id);
                            if let Some(parent_handle) = &this.parent {
                                parent_handle.update(_cx, |parent_data, parent_cx| {
                                    parent_data.data_store = this.select_id.to_string();
                                    parent_cx.notify();
                                });
                            }
                        },
                    )))
                },
            )
    }
}
