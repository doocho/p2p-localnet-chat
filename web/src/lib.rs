use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{BroadcastChannel, MessageEvent};
use local_chat_core::message::Message;

thread_local! {
    static BROADCAST: std::cell::RefCell<Option<BroadcastChannel>> = const { std::cell::RefCell::new(None) };
    static NICK: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    static CHANNEL_NAME: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

fn compute_channel_id(channel: &Option<String>) -> String {
    match channel {
        Some(name) if !name.is_empty() => format!("local-chat:channel:{}", name),
        _ => "local-chat:channel:global".to_string(),
    }
}

#[wasm_bindgen]
pub fn init(nick: String, channel: Option<String>) -> Result<(), JsValue> {
    let bc_name = compute_channel_id(&channel);
    let bc = BroadcastChannel::new(&bc_name)?;

    NICK.with(|n| *n.borrow_mut() = nick);
    CHANNEL_NAME.with(|c| *c.borrow_mut() = channel);
    BROADCAST.with(|b| *b.borrow_mut() = Some(bc.clone()));

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        if let Ok(data) = event.data().dyn_into::<js_sys::JsString>() {
            let window = web_sys::window().unwrap();
            let mut init = web_sys::CustomEventInit::new();
            init.set_detail(&JsValue::from(data));
            let event = web_sys::CustomEvent::new_with_event_init_dict("local-chat-message", &init).unwrap();
            let _ = window.dispatch_event(&event);
        }
    });
    bc.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    Ok(())
}

#[wasm_bindgen]
pub fn set_nick(nick: String) {
    NICK.with(|n| *n.borrow_mut() = nick);
}

#[wasm_bindgen]
pub fn set_channel(channel: Option<String>) -> Result<(), JsValue> {
    let bc_name = compute_channel_id(&channel);
    let bc = BroadcastChannel::new(&bc_name)?;
    BROADCAST.with(|b| *b.borrow_mut() = Some(bc.clone()));
    CHANNEL_NAME.with(|c| *c.borrow_mut() = channel);
    Ok(())
}

#[wasm_bindgen]
pub fn send_message(content: String) -> Result<(), JsValue> {
    let nick = NICK.with(|n| n.borrow().clone());
    let channel = CHANNEL_NAME.with(|c| c.borrow().clone());
    let msg = Message::chat_message(nick, "all".to_string(), content, channel);
    let json = serde_json::to_string(&msg).map_err(|e| JsValue::from_str(&e.to_string()))?;
    BROADCAST.with(|b| {
        if let Some(bc) = b.borrow().as_ref() { let _ = bc.post_message(&JsValue::from_str(&json)); }
    });
    Ok(())
}

#[wasm_bindgen]
pub fn discovery_ping() -> Result<(), JsValue> {
    let nick = NICK.with(|n| n.borrow().clone());
    let channel = CHANNEL_NAME.with(|c| c.borrow().clone());
    let payload = serde_json::json!({
        "type": "discovery",
        "username": nick,
        "port": 0u16,
        "peer_id": uuid::Uuid::new_v4(),
        "channel": channel,
    });
    BROADCAST.with(|b| {
        if let Some(bc) = b.borrow().as_ref() { let _ = bc.post_message(&JsValue::from_str(&payload.to_string())); }
    });
    Ok(())
}

