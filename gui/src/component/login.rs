use crate::component::{rgb_to_u32};

use crate::state::*;

use gpui::*;
use gpui_component::avatar::Avatar;
use gpui_component::button::Button;
use gpui_component::form::{field, v_form};
use gpui_component::input::{Input, InputState};
use gpui_component::{Disableable, Sizable, h_flex, v_flex};


use log::info;
use reqwest;
use serde::{Deserialize, Serialize};
use crate::component::home::HomeView;

pub enum Page {
    LoginPage,
    RegisterPage,
    ResetPasswordPage,
}

pub struct LoginView {
    username: Entity<InputState>,
    password: Entity<InputState>,
    avatar_path: Option<SharedString>,
    login_state: bool,
    button_loading: bool,
    err_msg: String,
    current_page: Page,
}

#[derive(Serialize, Deserialize, Clone)]
struct LoginParams {
    #[serde(rename = "username")]
    username: String,
    #[serde(rename = "password")]
    password: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct LoginResponse<T> {
    #[serde(rename = "code")]
    code: String,
    #[serde(rename = "data")]
    data: T,
    #[serde(rename = "msg")]
    msg: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct LoginResponseMsg {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_avatar")]
    pub user_avatar: String,
    #[serde(rename = "user_token")]
    pub user_token: String,
}

impl LoginView {
    pub fn new(win: &mut Window, cx: &mut Context<Self>) -> Self {
        LoginView {
            username: cx.new(|child_cx| InputState::new(win, child_cx)),
            password: cx.new(|child_cx| InputState::new(win, child_cx)),
            avatar_path: None,
            login_state: false,
            err_msg: String::new(),
            current_page: Page::LoginPage,
            button_loading: false,
        }
    }

    pub fn login_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("login_button")
            .bg(rgb(rgb_to_u32(195, 197, 203)))
            .mt_8()
            .label("登录")
            .disabled(self.button_loading)
            .on_click(cx.listener(|this, event, window, cx| {
                let win_handler = window.window_handle();
                let global_state = cx.global::<GlobalState>().0.clone();

                let address = global_state.read(cx).http_server.clone();
                let tokio_handler = global_state.read(cx).tokio_handle.clone();
                let username = this.username.read(cx).text().to_string();
                let password = this.password.read(cx).text().to_string();

                if username.len() == 0 || password.len() == 0 {
                    this.err_msg = "请填写用户名密码".to_string();
                    return
                }

                this.button_loading = true;

                let mut btn_cx_async_cx = cx.to_async().clone();
                let btn_cx_entity = cx.entity().clone();

                cx.spawn(move |_, _: &mut AsyncApp| async move {

                    let res = tokio_handler.spawn(async move {
                        let client = reqwest::Client::new();
                        let body = LoginParams {
                            username: username,
                            password: password,
                        };

                        let result: Result<LoginResponse<LoginResponseMsg>, anyhow::Error> = {
                                let response = client.post(format!("{}/user/login", address))
                                    .json(&body).send().await.map_err(|e| anyhow::anyhow!("请求失败: {}", e))?;

                                let bytes = response.bytes().await.map_err(|e| anyhow::anyhow!("读取响应失败: {}", e))?;

                                let body_str = String::from_utf8_lossy(&bytes);
                                // info!("DEBUG - 原始响应体: {}", body_str);

                                let login_resp: LoginResponse<LoginResponseMsg> =
                                    serde_json::from_slice(&bytes).map_err(|e| { anyhow::anyhow!("JSON 解析失败: {}\n原始内容: {}",e,body_str) })?;
                                Ok(login_resp)
                            };
                        result
                    });

                    match res.await {
                        Ok(Ok(r)) => {
                            if r.code == "200" {
                                info!("登录成功！Token: {}", r.data.user_token);
                                global_state.update(&mut btn_cx_async_cx, |state, cx| {
                                    state.user_state = r.data;
                                });

                                btn_cx_entity.update(&mut btn_cx_async_cx, |this, cx| {
                                    this.login_state = true;
                                    HomeView::new(cx);

                                });

                                win_handler.update(&mut btn_cx_async_cx, |view, window, cx| {
                                        window.remove_window();
                                }).expect("close login window failed");
                            } else {
                                btn_cx_entity.update(&mut btn_cx_async_cx, |this, cx| {
                                    this.err_msg = r.msg;
                                });
                            }
                        }
                        Ok(Err(e)) => {
                            println!("业务错误: {:?}", e);
                            btn_cx_entity.update(&mut btn_cx_async_cx, |this, cx| {
                                this.err_msg = e.to_string();
                            })

                        },
                        Err(e) =>{
                            println!("任务执行失败: {:?}", e);
                            btn_cx_entity.update(&mut btn_cx_async_cx, |this, cx| {
                                this.err_msg = e.to_string();
                            })
                        },
                    }

                })
                .detach();
                this.button_loading = false;
            }))
            .into_element()
    }

    pub fn login_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
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
                        cx.listener(|this, _, _, down_cx| {
                            let mut async_cx = down_cx.to_async();
                            let entity_view = down_cx.entity();

                            down_cx
                                .spawn(|_, _: &mut AsyncApp| async move {
                                    let paths = entity_view.update(&mut async_cx, |this, cx| {
                                            cx.prompt_for_paths(PathPromptOptions {
                                                files: true,
                                                directories: false,
                                                multiple: true,
                                                prompt: None,
                                            })
                                        });

                                    info!("{:#?} ", paths);

                                        if let Ok(result) = paths.await {
                                            if let Ok(Some(path_vec)) = result {
                                                if let Some(first_path) = path_vec.first() {
                                                    let path_str = first_path.to_string_lossy().to_string();
                                                    entity_view.update(&mut async_cx, |this, cx| {
                                                        this.avatar_path = Some(path_str.replace("\\", "/").into());
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
                v_form()
                    .child(field().label("用户名").child(Input::new(&self.username)))
                    .child(field().label("密 码").child(Input::new(&self.password)))
            )
            .child(
                h_flex().child(self.login_button(cx))

            )
    }

    pub fn register_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(
            Button::new("Register")
                .on_click(|a, b, c|{})
                .hover(|mut style|{
                    style.background = Some(rgb(rgb_to_u32(30, 31, 34)).into());
                    style
            })
        )
    }

    pub fn register_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Register")
    }
}

impl Render for LoginView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let element = match self.current_page {
            Page::LoginPage => self.login_page(cx).into_any_element(),
            Page::RegisterPage => self.register_page(cx).into_any_element(),
            Page::ResetPasswordPage => self.login_page(cx).into_any_element(),
        };
        v_flex()
            .p_12()
            .bg(rgb(rgb_to_u32(67, 69, 74)))
            .w_full()
            .text_color(Hsla::white())
            .h_full()
            .items_center()
            .flex_none()
            .child(element)
            .child(self.err_msg.clone())
    }
}
