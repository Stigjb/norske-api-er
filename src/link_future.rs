use std::future::Future;
use wasm_bindgen_futures::spawn_local;
use yew::{Component, ComponentLink};

/// Trait that allows you to use `ComponentLink` and `AgentLink` to register futures.
pub trait LinkFuture {
    type Message;
    /// This method processes a Future that returns a message and sends it back to the component's
    /// loop.
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    fn send_future<F>(&self, future: F)
    where
        F: Future<Output = Self::Message> + 'static;

    /// Registers a future that resolves to multiple messages.
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    fn send_future_batch<F>(&self, future: F)
    where
        F: Future<Output = Vec<Self::Message>> + 'static;
}

impl<COMP: Component> LinkFuture for ComponentLink<COMP> {
    type Message = COMP::Message;

    fn send_future<F>(&self, future: F)
    where
        F: Future<Output = Self::Message> + 'static,
    {
        let link: ComponentLink<COMP> = self.clone();
        let js_future = async move {
            let message: COMP::Message = future.await;
            link.send_message(message);
        };
        spawn_local(js_future);
    }

    fn send_future_batch<F>(&self, future: F)
    where
        F: Future<Output = Vec<Self::Message>> + 'static,
    {
        let link: ComponentLink<COMP> = self.clone();
        let js_future = async move {
            let messages: Vec<COMP::Message> = future.await;
            link.send_message_batch(messages);
        };
        spawn_local(js_future);
    }
}
