use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    AddEmoji(String),
    ToggleEmojiPicker,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

#[derive(Properties, PartialEq)]
pub struct EmojiPickerProps {
    on_select: Callback<String>,
}

#[function_component(EmojiPicker)]
fn emoji_picker(props: &EmojiPickerProps) -> Html {
    let emojis = vec!["😀", "😂", "😍", "🔥", "👍", "❤️", "🎉", "🤔", "👏", "🙌", "😎", "🤩", "🥳", "😊", "🤗"];
    
    html! {
        <div class="absolute bottom-16 right-16 bg-white shadow-xl rounded-lg p-2 grid grid-cols-5 gap-2 border border-gray-200 z-10">
            {for emojis.iter().map(|emoji_str| {
                let emoji = emoji_str.to_string();
                let emoji_for_closure = emoji.clone();
                let on_select = props.on_select.clone();

                html! {
                    <button 
                        onclick={move |_| on_select.emit(emoji_for_closure.clone())}
                        class="text-2xl hover:bg-gray-100 rounded p-1 transition-colors duration-200"
                    >
                        {emoji}
                    </button>
                }
            })}
        </div>
    }
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    show_emoji_picker: bool,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            show_emoji_picker: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        true
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        true
                    }
                    _ => false,
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                self.show_emoji_picker = false;
                false
            }
            Msg::AddEmoji(emoji) => {
                if let Some(input) = self.chat_input.cast::<HtmlInputElement>() {
                    let current_value = input.value();
                    input.set_value(&format!("{}{}", current_value, emoji));
                }
                self.show_emoji_picker = false;
                true
            }
            Msg::ToggleEmojiPicker => {
                self.show_emoji_picker = !self.show_emoji_picker;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        
        html! {
            <div class="flex w-screen bg-gradient-to-br from-purple-100 via-blue-100 to-pink-100 min-h-screen animate-gradient">
                <div class="flex-none w-56 h-screen bg-white shadow-lg">
                    <div class="text-xl p-3 font-bold text-blue-700">{"👥 Kontak"}</div>
                    {
                        self.users.clone().iter().map(|c| {
                            html!{
                                <div class="flex m-3 bg-blue-50 rounded-lg p-2 shadow-sm hover:bg-blue-100 transition-colors duration-200">
                                    <img class="w-10 h-10 rounded-full" src={c.avatar.clone()} alt="avatar"/>
                                    <div class="ml-3">
                                        <div class="text-sm font-semibold">{c.name.clone()}</div>
                                        <div class="text-xs text-gray-500">{"Online"}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-300 flex items-center px-4 bg-white shadow-sm">
                        <div class="text-xl font-semibold text-blue-800">{"💬 Percakapan"}</div>
                    </div>
                    <div class="w-full grow overflow-auto px-6 py-4 space-y-4">
                        {
                            self.messages.iter().map(|p| {
                                let contact = self.users.iter().find(|c| c.name == p.from).unwrap_or_else(|| {
                                    Box::leak(Box::new(UserProfile {
                                        name: p.from.clone(),
                                        avatar: format!("https://avatars.dicebear.com/api/adventurer-neutral/{}.svg", p.from),
                                    }))
                                });
                                let is_me = p.from == "saya";
                                let bubble_align = if is_me { "justify-end" } else { "justify-start" };
                                let bubble_color = if is_me { "bg-blue-200 text-right rounded-tl-lg rounded-bl-lg rounded-br-lg" } else { "bg-white text-left rounded-tr-lg rounded-br-lg rounded-bl-lg" };

                                let content = if p.message.ends_with(".gif") {
                                    html! { <img class="mt-2 rounded-lg max-w-xs" src={p.message.clone()} /> }
                                } else {
                                    let replaced = p.message.replace(":senyum:", "😊").replace(":hati:", "❤️");
                                    html! { <div class="text-sm text-gray-800">{replaced}</div> }
                                };

                                html!{
                                    <div class={classes!("flex", bubble_align)}>
                                        <div class={classes!("max-w-md", "flex", "items-end", bubble_color, "p-4", "shadow", "space-x-2", "hover:shadow-md", "transition-shadow")}>
                                            { if !is_me {
                                                html!{ <img class="w-8 h-8 rounded-full" src={contact.avatar.clone()} alt="avatar"/> }
                                            } else {
                                                html!{}
                                            }}
                                            <div>
                                                { if !is_me {
                                                    html!{ <div class="text-xs font-bold text-gray-600">{p.from.clone()}</div> }
                                                } else {
                                                    html!{}
                                                }}
                                                { content }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-16 bg-white shadow-inner flex items-center px-4 relative">
                        <button 
                            onclick={ctx.link().callback(|_| Msg::ToggleEmojiPicker)}
                            class="p-2 text-gray-500 hover:text-blue-600 transition-colors duration-200"
                        >
                            {"😊"}
                        </button>
                        {if self.show_emoji_picker {
                            html! {
                                <EmojiPicker on_select={ctx.link().callback(Msg::AddEmoji)} />
                            }
                        } else {
                            html! {}
                        }}
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Ketik pesan atau kirim .gif..."
                            class="block w-full py-2 pl-4 mx-3 bg-gray-100 rounded-full outline-none focus:text-gray-700 focus:ring-2 focus:ring-blue-300 transition-all duration-200"
                            name="pesan"
                            required=true
                        />
                        <button
                            onclick={submit}
                            class="p-3 bg-blue-600 hover:bg-blue-700 transition rounded-full flex justify-center items-center text-white"
                        >
                            <svg fill="currentColor" viewBox="0 0 24 24" class="w-5 h-5">
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}