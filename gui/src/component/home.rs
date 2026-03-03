
use crate::state::GlobalState;

use gpui::*;
use gpui_component::divider::Divider;
use gpui_component::*;
use gpui_component::button::Button;

use crate::component::login::LoginView;
use crate::component::message_page::MessagePage;
use crate::component::rgb_to_u32;

pub struct HomeView {
    pub data_store: String,
    pub select_page: i32,
    pub message_page:Entity<MessagePage>,
    is_maximized: bool,
    restore_bounds: Option<Bounds<Pixels>>,
}

impl HomeView {
    pub fn new(cx: &mut Context<LoginView>) {

        let mut window_options = WindowOptions::default();
        let window_size = size(px(1200.), px(700.));
        window_options.window_bounds = Some(WindowBounds::centered(window_size, cx));
        window_options.window_min_size = Some(window_size);
        window_options.titlebar = Some(TitlebarOptions{
            title: Some(SharedString::from("")),
            appears_transparent: false,
            ..Default::default()
        });

        cx.open_window(window_options, |window, app| {

            app.global::<GlobalState>().0.clone().update(app, |this, cx|{
                this.init_ws(cx);
            });

            gpui_component::init(app);

            let message_page = app.new(|cx| MessagePage::new(cx, window));

            let home_view = HomeView{
                data_store: String::from(""),
                select_page:0,
                message_page:message_page,
                is_maximized: false,
                restore_bounds: None,
            };
 
            let view = app.new(|_| home_view);

            app.new(|c| Root::new(view, window, c))
        }).expect("");
    }

    pub fn top_sidebar(&self,  window: &mut Window, cx: &mut Context<Self>) ->  impl IntoElement {
        h_flex().w_full().h(px(40.0))
            .child("TopNav")
            .child(div().flex_grow())
            .child(
                h_flex()
                    .p_2()
                    .child(
                        Button::new("min").label("min")
                            .on_click(cx.listener(|this, event ,window, cx|{
                                cx.notify();
                                window.minimize_window()
                            }))
                    )
                    .child(
                        Button::new("max").label("max")
                            .on_click(cx.listener(|this, _event, window, cx| {
                                if !this.is_maximized {
                                    this.restore_bounds = Some(window.bounds());

                                    if let Some(display) = window.display(cx) {
                                        let usable_bounds = display.visible_bounds();
                                        window.zoom_window();
                                        window.resize(usable_bounds.size);
                                    }

                                    this.is_maximized = true;
                                } else {
                                    if let Some(bounds) = this.restore_bounds {
                                        window.zoom_window();
                                        window.resize(bounds.size);
                                    }

                                    this.is_maximized = false;
                                }

                                cx.notify();
                            }))
                    )
                    .child(
                        Button::new("close").label("close")
                            .on_click(cx.listener(|this, event ,window, cx|{
                                window.remove_window()
                            }))
                    )
            )

    }
}

impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {


        let icons = [
            "icon/message_icon.png",
            "icon/user_icon.png"
        ];

        v_flex()
            .size_full()
            // .child(self.top_sidebar(window, cx))
            // .child(Divider::horizontal().w_full())
            .child(
              h_flex()
                  .items_start()
                  .size_full()
                  .child(
                      v_flex()
                          .items_center()
                          .justify_center()
                          .w(px(60.))
                          .h_full()
                          .children(icons.iter().enumerate().map(|(index, icon_path)| {
                              div()
                                  .id(format!("btn-id-{}", index))
                                  .rounded_2xl()
                                  .flex()
                                  .w(px(40.))
                                  .h(px(40.))
                                  .items_center()
                                  .justify_center()
                                  .hover(|mut style| {
                                      if self.select_page != index as i32 {
                                          style.background = Some(rgb(rgb_to_u32(235, 235, 235)).into());
                                      }
                                      style
                                  })
                                  .bg(
                                      if self.select_page == index as i32 {
                                          rgb(rgb_to_u32(0, 153, 255))
                                      } else {
                                          rgb(rgb_to_u32(245, 245, 245))
                                      }
                                  )
                                  .on_click(cx.listener(move |this, event, window, view| {
                                      this.select_page = index as i32;
                                      view.notify();
                                  }))
                                  .m_2()
                                  .child(
                                      img(icon_path.to_string())
                                          .size(px(25.))
                                          .object_fit(ObjectFit::Cover)
                                  )
                          }))
                          .child(div().flex_grow())
                          .child(div().child("123"))

                  )
                    .child(Divider::vertical().h_full())
                    .child(
                        if self.select_page == 0 {
                            self.message_page.clone().into_any_element()
                        }else if self.select_page == 1 {
                            div().child("1111").into_any_element()
                        } else {
                            div().child("2222").into_any_element()
                        }

                    ),
            )

            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
    }
}
