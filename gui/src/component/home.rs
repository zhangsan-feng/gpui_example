
use crate::state::{EventBus, GlobalState};

use gpui::*;
use gpui_component::divider::Divider;
use gpui_component::*;
use gpui_component::button::Button;
use log::{info};
use crate::component::friend_page::FriendPage;
use crate::component::login::LoginView;
use crate::component::message_page::MessagePage;
use crate::component::{rgb_to_u32, GroupMembers, MessageGroup, UserDetailInfo, WsMsgEvent};
use crate::service::http_request;

pub struct HomeView {
    select_page: i32,
    message_page:Entity<MessagePage>,
    friend_page:Entity<FriendPage>,
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
            
            let mut home_view = HomeView{
                select_page:0,
                message_page: app.new(|cx| MessagePage::new(cx, window)),
                friend_page:app.new(|cx|FriendPage::new(cx, window)),
                is_maximized: false,
                restore_bounds: None,
            };


            let view = app.new(|cx| {
                home_view.init_component_data(cx);
                home_view.init_subscribe_ws(cx);
                home_view
             });

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

    fn init_subscribe_ws(&mut self, cx: &mut Context<Self>){
        let state_handle = cx.global::<GlobalState>().0.clone();
        cx.subscribe(&state_handle, |this: &mut Self, _model, event: &EventBus, cx| {
            match event {
                EventBus::WebSocketText(txt)=>{
                    println!("组件收到消息: {}", txt);

                    match serde_json::from_str::<WsMsgEvent>(&txt) {
                        Ok(event) => {
                            this.message_page.update(cx, |this,cx| {
                                this.update_component_data(event.clone(), cx);
                            });
                            this.friend_page.update(cx, |this, cx|{
                                this.update_component_data(event.clone(), cx);
                            });

                        }
                        Err(e) => {
                            eprintln!("Failed to parse WsMsgEvent: {} | Raw: {}", e, txt);
                        }
                    }
                }
                EventBus::ChildrenChangeSelectIndex=>{
                    this.select_page = 0
                }
                _ => {}
            }

        }).detach();


    }

    fn init_component_data(&self,cx: &mut Context<Self>){

        let global_state = cx.global::<GlobalState>().0.clone().read(cx);
        let user_id = global_state.user_state.user_id.clone();
        let tokio_handler = global_state.tokio_handle.clone();
        let address = global_state.http_server.clone();

        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();


        cx.spawn( |_, _: &mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move {
                http_request::HttpClient::new().get(format!("{}/user_message_group?user_id={}", address, user_id)).await
            });
            match res.await {
                Ok(Ok(r)) => {
                    info!("{:?}",r.data);
                    match serde_json::from_value::<UserDetailInfo>(r.data)  {
                        Ok(data)=>{
                            entity.update(&mut cx_async, |this, cx| {
                                this.message_page.update(cx, |this, cx|{
                                    this.init_component_data(data.message_groups, cx);

                                });
                                this.friend_page.update(cx, |this, cx|{
                                    info!("{:?}", data.friends);
                                   this.init_component_data(data.friends, cx);
                                });
                            });
                        },
                        Err(e)=>{info!("{}",e)}
                    }
                }
                Ok(Err(e)) => info!("http error: {:?}", e),
                Err(e) => info!("tokio runtime error: {:?}", e),
            }
        }).detach();
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
                           self.friend_page.clone().into_any_element()
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
