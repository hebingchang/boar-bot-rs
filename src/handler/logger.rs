use async_trait::async_trait;
use ricq::handler::QEvent;
use crate::engine::Module;

pub struct LoggingHandler;

#[async_trait]
impl Module for LoggingHandler {
    fn get_name(&self) -> &'static str {
        "logger"
    }

    async fn handle(&self, e: QEvent) {
        match e {
            QEvent::GroupMessage(m) => {
                tracing::info!(
                    "MESSAGE (GROUP={}): {}",
                    m.inner.group_code,
                    m.inner.elements
                )
            }
            QEvent::FriendMessage(m) => {
                tracing::info!(
                    "MESSAGE (FRIEND={}): {}",
                    m.inner.from_uin,
                    m.inner.elements
                )
            }
            QEvent::GroupTempMessage(m) => {
                tracing::info!("MESSAGE (TEMP={}): {}", m.inner.from_uin, m.inner.elements)
            }
            QEvent::GroupRequest(m) => {
                tracing::info!(
                    "REQUEST (GROUP={}, UIN={}): {}",
                    m.inner.group_code,
                    m.inner.req_uin,
                    m.inner.message
                )
            }
            QEvent::NewFriendRequest(m) => {
                tracing::info!("REQUEST (UIN={}): {}", m.inner.req_uin, m.inner.message)
            }
            _ => {}
        }
    }
}