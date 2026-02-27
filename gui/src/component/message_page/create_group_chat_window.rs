use crate::state::GlobalState;
use gpui::*;
use gpui_component::avatar::Avatar;
use gpui_component::button::Button;
use gpui_component::form::field;
use gpui_component::input::{Input, InputState};
use gpui_component::*;
use log::info;
use reqwest::multipart;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use crate::service::http_request::{ HttpClient};

pub struct CreateGroupChatWindow {
    group_name: Entity<InputState>,
    avatar_path: Option<SharedString>,
}

impl CreateGroupChatWindow {
    pub fn new(cx: &mut Context<Self>, window: &mut Window) -> Self {
        let mut myself = CreateGroupChatWindow {
            group_name: cx.new(|cx| InputState::new(window, cx)),
            avatar_path: Default::default(),
        };
        myself
    }
}

impl Render for CreateGroupChatWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.on_window_closed(|window| {
            window.update_global(|global_state: &mut GlobalState, app| {
                global_state.0.update(app, |this, cx| {
                    this.dial_window_is_open = false;
                });
            })
        })
        .detach();

        v_flex()
            .py_4()
            // .size_full()
            .items_center()
            .child(
                div()
                    .flex_none()
                    .size_40()
                    .rounded_full()
                    .border_2()
                    .border_color(rgb(0x9999AF))
                    .bg(rgb(0x333333))
                    .cursor_pointer()
                    .overflow_hidden()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            let mut async_cx = cx.to_async();
                            let entity_view = cx.entity();

                            cx.spawn(|_, _: &mut AsyncApp| async move {
                                    let paths = entity_view.update(&mut async_cx, |this, cx| {
                                        cx.prompt_for_paths(PathPromptOptions {
                                            files: true,
                                            directories: false,
                                            multiple: false,
                                            prompt: None,
                                        })
                                    });

                                    info!("{:#?} ", paths);

                                    if let Ok(result) = paths.await {
                                        if let Ok(Some(path_vec)) = result {
                                            if let Some(first_path) = path_vec.first() {
                                                let path_str =
                                                    first_path.to_string_lossy().to_string();
                                                entity_view.update(&mut async_cx, |this, cx| {
                                                    this.avatar_path =
                                                        Some(path_str.replace("\\", "/").into());
                                                    println!("{:?}", this.avatar_path);
                                                });
                                            }
                                        }
                                    }
                                })
                                .detach();
                        }),
                    )
                    .child(if let Some(path) = &self.avatar_path {
                        div().size_full().rounded_full().overflow_hidden().child(
                            Avatar::new()
                                .src(path.to_string())
                                .size_full()
                                .with_size(gpui_component::Size::Size(px(150.0))),
                        )
                    } else {
                        div().size_full().rounded_full().bg(rgb(0x333333))
                    }),
            )
            .child(
                field()
                    .label("群聊名称")
                    .items_center()
                    .child(Input::new(&self.group_name).w(px(200.)).h(px(50.))),
            )
            .child(
                Button::new("create_group_chat").label("创建")
                    .mt_2()
                    .justify_center()
                    .on_click(cx.listener(|this, _, window, cx|{
                        let global_state = cx.global::<GlobalState>().0.read(cx).clone();
                        let group_name = this.group_name.read(cx).text().to_string();
                        let avatar_path = this.avatar_path.clone();
                        let address = global_state.http_server;
                        let window_handler = window.window_handle();
                        let mut cx_async = cx.to_async().clone();

                        cx.spawn(move |_, _: &mut AsyncApp| async move {
                            let res = global_state.tokio_handle.spawn(async move{
                                let mut form = multipart::Form::new()
                                    .text("user_id", global_state.user_state.user_id.to_string())
                                    .text("group_name", group_name.to_string())
                                    .text("type", "group_chat");

                                    if let Some(file_path) = avatar_path{
                                        let file = match File::open(file_path.as_str()).await {
                                            Ok(f) => f,
                                            Err(e) => {
                                                eprintln!("Failed to open file: {}", e);
                                                return Err(anyhow::anyhow!("Failed to open file: {}", e));
                                            }
                                        };
                                        let stream = FramedRead::new(file, BytesCodec::new());
                                        let body = reqwest::Body::wrap_stream(stream);
                                        let file_name = file_path.split('/').last().unwrap_or("").to_string();
                                        let part = multipart::Part::stream(body).file_name(file_name);
                                        form = form.part("file", part);
                                    }

                                let response = HttpClient::new().post_form(format!("{}/create_group_chat", address), form).await;
                                response
                            });
                            match res.await {
                                Ok(Ok(r)) => {
                                    if r.data == "success"{
                                        window_handler.update(&mut cx_async, |_, window, _|{
                                            window.remove_window();
                                        }).expect("close window filed");
                                    }
                                }
                                Ok(Err(e)) => println!("http error: {:?}", e),
                                Err(e) => println!("tokio runtime error: {:?}", e),
                            }
                        }).detach()

                    }))
            )
    }
}
