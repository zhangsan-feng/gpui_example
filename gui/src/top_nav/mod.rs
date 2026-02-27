
use gpui::*;
use gpui_component::*;
use gpui_component::button::Button;
use gpui_component::input::Copy;
use gpui_component::menu::{ContextMenuExt, DropdownMenu, PopupMenuItem};



#[derive(Clone)]
pub struct TopNav{}


impl TopNav{
    pub fn new() -> TopNav {
        TopNav{}
    }
}

impl Render for  TopNav {

    fn render(&mut self, w: &mut Window, c: &mut Context<Self>) -> impl IntoElement {

        h_flex().m_2().w_full()
            .child(
                Button::new("menu-btn1")
                    .label("会话1")
                    .dropdown_menu(move |menu, _, _| {
                        menu.item(
                            PopupMenuItem::new("新建会话")
                                .on_click(|_,a, b|{
                                    let width = a.bounds().clone().size.width;
                                    a.open_dialog(b, move |dialog, _, _| {
                                        dialog.
                                            confirm()
                                            .title("Welcome")
                                            .title("")
                                            // .on_ok(|_, window, cx| {
                                            //     window.push_notification("Item deleted", cx);
                                            //     true // Return true to close dialog
                                            // })
                                            // .on_cancel(|_, window, cx| {
                                            //     window.push_notification("Cancelled", cx);
                                            //     true
                                            // })
                                            .child(
                                                div()
                                                    .child(
                                                        v_flex()
                                                            .child("This is a dialog content.")
                                                    )
                                            ).width(width - gpui::Pixels::from(200.0))
                                    })

                                })
                        )
                            .separator()
                            .menu("Exit", Box::new(Copy))

                    })
            ).w(gpui::Pixels::from(50.))
            .child(
                Button::new("custom-item-menu")
                    .label("Options")
                    .dropdown_menu(|menu, window, cx| {
                        menu.item(
                            PopupMenuItem::new("Custom Action")
                                
                                .on_click(|a,b,c|{

                                    // Custom click handler logic
                                    println!("Custom Action Clicked!");
                                })
                        )
                            .separator()

                    }).into_any_element()

            )
            .child(
                Button::new("123456")
                    .label("Item")
                    .on_click(move |_, _, _| {
                        println!("{}",format!("Item {}", "123456"));
                    })
                    .context_menu(move |menu, _, _| {
                        menu
                            .item(
                                PopupMenuItem::new("new ").on_click(move |_, _, _| {
                                    println!("{}",format!("Item {}", "123456"));
                                })
                            )
                            .item(
                                PopupMenuItem::new("new ").on_click(move |_, _, _| {
                                    println!("{}",format!("Item {}", "123456"));
                                })
                            )
                    })
            )

            .w(px(50.))

    }
}

