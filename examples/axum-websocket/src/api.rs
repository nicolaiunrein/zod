use std::collections::VecDeque;

use futures::channel::mpsc;
use futures::Stream;
use futures::StreamExt;
use zod::types::Usize;
use zod::{Namespace, RequestType, ResponseType};

#[derive(Namespace, Default)]
pub struct Chat {
    subscribers: Vec<mpsc::UnboundedSender<Message>>,
}

#[derive(RequestType, ResponseType, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[zod(namespace = "Chat")]
pub struct User {
    name: String,
}

#[derive(RequestType, ResponseType, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[zod(namespace = "Chat")]
pub struct Message {
    user: User,
    content: String,
}

#[zod::rpc]
impl Chat {
    pub async fn send(&mut self, msg: Message) {
        self.subscribers.retain(|sub| !sub.is_closed());
        for sub in self.subscribers.iter_mut() {
            let _ = sub.unbounded_send(msg.clone());
        }
    }

    pub fn messages(&mut self, len: Usize) -> impl Stream<Item = VecDeque<Message>> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.subscribers.push(tx);
        rx.scan(VecDeque::new(), move |history, msg| {
            history.push_back(msg);
            if history.len() > *len {
                history.pop_front();
            }
            futures::future::ready(Some(history.clone()))
        })
    }
}

#[derive(zod::Backend)]
pub struct AppBackend(pub Chat);
