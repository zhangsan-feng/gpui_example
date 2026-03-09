use std::collections::HashSet;
use std::path::PathBuf;

use crate::state::GlobalState;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::{h_flex, v_flex, StyleSized};
use gpui_component::label::Label;
use log::info;
use reqwest::multipart;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::component::rgb_to_u32;
use crate::service::http_request::{HttpClient, RestResponse};

pub struct SendMessageEntity {
    text_input: Entity<InputState>,
    pick_send_files: Vec<PathBuf>,
    pub group_id: String,
    pub panel_height: f32,
    err_msg: String,
}


impl SendMessageEntity {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        SendMessageEntity {
            text_input: cx.new(|cx| InputState::new(window, cx)),
            pick_send_files: Vec::new(),
            group_id: String::new(),
            panel_height: 160.0,
            err_msg: String::new(),
        }
    }

    pub fn pick_files(&self, cx: &mut Context<Self>) {
        let mut async_cx = cx.to_async();
        let entity_view = cx.entity();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let paths = entity_view.update(&mut async_cx, |this, cx| {
                cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: true,
                    prompt: None,
                })
            });

            if let Ok(result) = paths.await {
                if let Ok(Some(path_vec)) = result {
                    info!("{:?}", path_vec);

                    entity_view.update(&mut async_cx, move |this, cx| {
                        let existing_set: HashSet<_> = this.pick_send_files.iter().collect();
                        let mut new_files: Vec<_> = path_vec.into_iter().filter(|p| !existing_set.contains(p)).collect();
                        if new_files.len() > 5 || this.pick_send_files.len() > 5 {
                            new_files.truncate(5);
                            this.err_msg = "最多选择5个文件".to_string();
                            cx.notify()
                        }
                        this.pick_send_files.extend(new_files);

                    })

                    // if let Some(first_path) = path_vec.first() {
                    //     let path_str = first_path.to_string_lossy().to_string();
                    //     entity_view.update(&mut async_cx, |this, cx| {
                    //         // update_this.avatar_path = Some(path_str.replace("\\", "/").into());
                    //     });
                    // }
                }
            }
        })
        .detach();
    }
}

impl Render for SendMessageEntity {
    fn render (&mut self, window: &mut Window, cx: &mut Context<Self>, ) -> impl IntoElement {
        v_flex()
            .h(px(self.panel_height))
            .w_full()
            .py_2()
            .px_2()
            .child(
                h_flex()
                    .ml_1()
                    .child(
                        div()
                            .id("sned_message_component-folder-id")
                            .child(img("icon/icons8-folder-96.png").size(px(24.)))
                            .p_1()
                            .rounded(px(4.))
                            .hover(|mut style| {
                                style.background = Some(rgb(rgb_to_u32(235, 235, 235)).into());
                                style
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event, window, cx| this.pick_files(cx)),
                            ),
                    )
                    .child(
                        div()
                            .id("sned_message_component-folder-id")
                            .child(img("icon/icons8-scissors-50.png").size(px(24.)))
                            .p_1()
                            .rounded(px(4.))
                            .hover(|mut style| {
                                style.background = Some(rgb(rgb_to_u32(235, 235, 235)).into());
                                style
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event, window, cx| {}),
                            ),
                    )
                    .children(
                        self.pick_send_files.iter().enumerate().map(|(index, path)| {
                            let path_str = path.to_string_lossy().to_string();
                            div()
                                .mx_1()
                                .relative()
                                .size(px(25.0))
                                .child(
                                    img(path_str.clone())
                                        .size(px(24.0))
                                )
                                .child(
                                    Button::new(("remove_img", index.clone()))
                                        .label("×")
                                        .p_0()
                                        .size(px(12.0))
                                        .w(px(12.0))
                                        .h(px(12.0))
                                        .absolute()
                                        .top(px(-6.0))
                                        .right(px(-6.0))
                                        .bg(gpui::red())
                                        .text_color(gpui::white())
                                        .rounded_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .on_click(cx.listener(move |this, _, _, _| {
                                            if !this.err_msg.is_empty() {
                                                this.err_msg = String::new();
                                            }
                                            this.pick_send_files.remove(index);
                                        }))
                                )
                        })
                    )
                    .child(Label::new(self.err_msg.to_string()))
                    .child(div().flex_grow())
                    .child(
                        div()
                            .child(img("icon/icons8-history-96.png").size(px(24.)))
                            .id("sned_message_component-history-id")
                            .p_1()
                            .rounded(px(4.))
                            .hover(|mut style| {
                                style.background = Some(rgb(rgb_to_u32(235, 235, 235)).into());
                                style
                            }),
                    ),
            )

            .child(
                Input::new(&self.text_input)
                    .items_start()
                    .size_full()
                    // .border_0()
            )
            .child(
                h_flex()
                    .items_end()
                    .justify_end()
                    .child(
                        Button::new("send_message")
                            .label("发送")
                            .bg(rgb(rgb_to_u32(0, 141, 235)))
                            .w(px(120.))
                            .on_click(cx.listener(|this, event, window, cx| {
                                let global_state = cx.global::<GlobalState>().0.clone();
                                let user_id = global_state.read(cx).clone().user_state.user_id;
                                let send_group_id = this.group_id.clone();

                                let tokio_handler = global_state.read(cx).tokio_handle.clone();
                                let address = global_state.read(cx).http_server.clone();
                                let msg = this.text_input.read(cx).text().to_string();
                                let pick_files = this.pick_send_files.clone();

                                info!("{}", msg);
                                cx.spawn( |_, _: &mut AsyncApp| async move {
                                    let res = tokio_handler.spawn(async move {

                                        let mut form = multipart::Form::new()
                                            .text("send_user_id", user_id.to_string())
                                            .text("send_group_id", send_group_id.to_string())
                                            .text("message", msg.to_string());

                                        for path in pick_files {

                                            let file_name = path
                                                .file_name()
                                                .map(|n| n.to_string_lossy().into_owned())
                                                .unwrap_or_else(|| "unknown_file".to_string());

                                            let file = match File::open(&path).await {
                                                Ok(f) => f,
                                                Err(_) => continue,
                                            };

                                            let stream = FramedRead::new(file, BytesCodec::new());
                                            let body = reqwest::Body::wrap_stream(stream);

                                            let part = multipart::Part::stream(body)
                                                .file_name(file_name);
                                            form = form.part("files", part);
                                        }

                                        let response = HttpClient::new().post_form(format!("{}/user_send_message", address), form).await;
                                        response

                                    });
                                    match res.await {
                                        Ok(Ok(r)) => {

                                        }
                                        Ok(Err(e)) => println!("http error: {:?}", e),
                                        Err(e) => println!("tokio runtime error: {:?}", e),
                                    }
                                })
                                .detach();

                                this.text_input.update(cx, |state, cx| {
                                    state.set_value("", window, cx);
                                });
                                this.pick_send_files.clear();

                            })),
                ),
            )
            .into_any_element()
    }
}
