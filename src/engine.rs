use async_trait::async_trait;
use ricq::handler::{Handler, QEvent};

#[async_trait]
pub trait Module: Send + Sync {
    fn get_name(&self) -> &'static str;
    async fn handle(&self, event: QEvent);
}

#[derive(Default)]
pub struct Engine {
    modules: Vec<Box<dyn Module>>,
}

impl Engine {
    /// Register modules with chaining functions
    pub fn register_module<M>(mut self, module: M) -> Self
        where
            M: Module + 'static + Sync + Send,
    {
        self.modules.push(Box::new(module));
        self
    }
}

#[async_trait]
impl Handler for Engine {
    /// Broadcast event to all registered modules
    async fn handle(&self, e: QEvent) {
        for module in &self.modules {
            module.handle(e.clone()).await;
        }
    }
}