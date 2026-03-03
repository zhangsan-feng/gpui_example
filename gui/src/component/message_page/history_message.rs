
use std::path::PathBuf;
use std::process::Command;
use crate::component::message_page::MessagePage;
use crate::state::GlobalState;
use gpui::*;
use gpui_component::avatar::Avatar;
use gpui_component::label::Label;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::{StyleSized, h_flex, v_flex, InteractiveElementExt, IconName};

use log::info;
use tokio::io::AsyncWriteExt;
use crate::component::message_page::entity::GroupHistory;
use crate::component::rgb_to_u32;



pub async fn download_and_open(file_url: &str, temp_file_path: &std::path::Path) -> anyhow::Result<()> {
    let bytes = reqwest::get(file_url).await?.bytes().await?;

    let mut file = tokio::fs::File::create(temp_file_path).await?;
    file.write_all(&bytes).await?;

    let s = temp_file_path.to_str().ok_or_else(|| anyhow::anyhow!("path not valid utf-8"))?;
    #[cfg(target_os = "macos")]
    { Command::new("open").arg(s).spawn()?; }
    #[cfg(target_os = "linux")]
    { Command::new("xdg-open").arg(s).spawn()?; }
    #[cfg(target_os = "windows")]
    { Command::new("cmd").args(["/C", "start", s]).spawn()?; }

    Ok(())
}

pub struct HistoryMessageEntity{
    pub scroll_handle: gpui::ListState,
    pub history_message:Vec<GroupHistory>
}

impl HistoryMessageEntity {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) ->Self  {
        HistoryMessageEntity{
            scroll_handle: ListState::new(0, ListAlignment::Bottom, px(100.)),
            history_message: vec![],
        }
    }
}

impl Render for HistoryMessageEntity {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let global_state = cx.global::<GlobalState>().0.read(cx).clone();
        let history_message_scroll_handle = self.scroll_handle.clone();
        let entity_view = cx.entity().clone();

        if self.history_message.len() == 0 {
            return div().into_any_element();
        }
        let message = self.history_message.clone();
        
        h_flex()
            .size_full()
            .child(
                gpui::list(history_message_scroll_handle, move |index, window, app| {
                    let message = message[index].clone();

                    let formatted_message: String = message
                        .message
                        .chars()
                        .collect::<Vec<char>>()
                        .chunks(40)
                        .map(|chunk| chunk.iter().collect::<String>())
                        .collect::<Vec<String>>()
                        .join("\n");

                    let element = h_flex().w(px(400.)).w_full().h_auto().py_2().px_3();

                    let message_image = message.files.iter().enumerate().map(|(index, file)| {
                        let id = format!("file-{}-{}", file, index);
                        let entity_view = entity_view.clone();
                        let file_path = file.to_string().clone();
                        let global_state = global_state.clone();

                        div()
                            .px_4()
                            .id(id)
                            // .hover(|mut style|{
                            //     style.background = Some(rgb(rgb_to_u32(228, 228, 228)).into());
                            //     style
                            // })
                            .child(
                                img(file.to_string())
                                    .size_with(gpui_component::Size::Size(px(80.0))).into_any_element()
                            )
                            .on_double_click(move |event, window, app| {
                                let file_path = file_path.clone();
                                let global_state = global_state.clone();
                                entity_view.update(app, |this, cx|{
                                    cx.spawn( |_, _: &mut AsyncApp| async move {

                                        global_state.tokio_handle.spawn(async move {

                                            let file_name = file_path.split('/').last().unwrap_or("");
                                            let mut temp_file_path: PathBuf = std::env::temp_dir();
                                            temp_file_path.push(file_name);

                                            match download_and_open(&*file_path, &*temp_file_path).await{
                                                Ok(msg)=>{},
                                                Err(err)=>{info!("error:{}", err)}
                                            }
                                        });


                                    }).detach();
                                })
                            })
                    });

                    let message_content = v_flex()
                        .child(
                            if global_state.user_state.user_id.clone() == message.send_user_id {
                                Label::new(message.send_username).text_right()
                            }else{
                                Label::new(message.send_username)
                            }
                        )
                        .child(
                            if message.files.is_empty() {
                                Label::new(formatted_message)
                                    .bg(rgb(rgb_to_u32(228, 231, 235)))
                                    .p_2()
                                    .rounded(px(4.))
                                    .into_any_element()
                            } else {
                                h_flex()
                                    .children(message_image)
                                    .into_any_element()
                            },
                        );

                    let avatar = div().child(
                        Avatar::new().src(message.send_user_avatar)
                    );



                    if global_state.user_state.user_id.clone() == message.send_user_id {
                        element
                            .justify_end()
                            .items_start()
                            .child(message_content)
                            .child(avatar)
                            .into_any_element()
                    } else {
                        element
                            .items_start()
                            .child(avatar)
                            .child(message_content)
                            .into_any_element()
                    }
                })
                    .size_full()
                    .p_2()
                    .mb(px(20.))
                    .into_any_element(),
            )
            .child(
                Scrollbar::vertical(&self.scroll_handle)
                    .scrollbar_show(ScrollbarShow::Always)
                    .axis(ScrollbarAxis::Vertical),
            )
            .into_any_element()
    }
}
