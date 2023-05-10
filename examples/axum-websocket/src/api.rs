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
    color: Color,
    content: String,
}

#[derive(RequestType, ResponseType, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[zod(namespace = "Chat")]
#[serde(try_from = "String", into = "String")]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl From<Color> for String {
    fn from(value: Color) -> Self {
        format!(
            "#{red:02x}{green:02x}{blue:02x}",
            red = value.red,
            green = value.green,
            blue = value.blue
        )
    }
}

impl TryFrom<String> for Color {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.starts_with('#') {
            return Err(String::from("Colors must start with a '#'"));
        }

        if value.len() == 7 {
            let red = u8::from_str_radix(&value[1..=2], 16).map_err(|err| err.to_string())?;
            let green = u8::from_str_radix(&value[3..=4], 16).map_err(|err| err.to_string())?;
            let blue = u8::from_str_radix(&value[5..=6], 16).map_err(|err| err.to_string())?;
            return Ok(Self { red, green, blue });
        }

        return Err(String::from("Invalid color format"));
    }
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
