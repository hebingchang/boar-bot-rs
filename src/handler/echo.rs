use async_trait::async_trait;
use ricq::handler::QEvent;
use crate::engine::Module;

pub struct EchoHandler;

#[async_trait]
impl Module for EchoHandler {
    fn get_name(&self) -> &'static str {
        "echo"
    }

    async fn handle(&self, e: QEvent) {
        match e {
            QEvent::FriendMessage(m) => {
                match m.client.send_friend_message(m.inner.from_uin, m.inner.elements).await {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }
}