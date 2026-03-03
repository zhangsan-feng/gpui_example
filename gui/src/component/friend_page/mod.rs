use gpui::*;

pub struct FriendPage{
    
}

impl FriendPage{
    pub fn new() -> Self{
        FriendPage{}
    }
}

impl Render for FriendPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
       div()
    }
}