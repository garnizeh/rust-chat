use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub send_message_callback: Callback<String>,
    pub change_username_callback: Callback<String>,
    pub username: String,
}

#[function_component(SendDialog)]
pub fn send_dialog(props: &Props) -> Html {
    let new_message_handler = use_state(String::default);
    let new_message = (*new_message_handler).clone();
    let new_username_handler = use_state(|| props.username.clone());
    let new_username = (*new_username_handler).clone();
    let is_editing_username_handler = use_state(bool::default);
    let is_editing_username = (*is_editing_username_handler).clone();

    let cloned_new_username_handler = new_username_handler.clone();
    let on_new_username = Callback::from(move |e: Event| {
        let target = e.target_dyn_into::<HtmlInputElement>();
        if let Some(input) = target {
            cloned_new_username_handler.set(input.value());
        }
    });

    let cloned_new_message_handler = new_message_handler.clone();
    let on_new_message = Callback::from(move |e: Event| {
        let target = e.target_dyn_into::<HtmlTextAreaElement>();
        if let Some(textarea) = target {
            cloned_new_message_handler.set(textarea.value());
        }
    });

    let message = new_message.clone();
    let callback = props.send_message_callback.clone();
    let on_submit = Callback::from(move |_: MouseEvent| {
        callback.emit(message.clone());
        new_message_handler.set("".to_string());
    });

    let edit_handler = is_editing_username_handler.clone();
    let on_edit_usename = Callback::from(move |_: MouseEvent| {
        edit_handler.set(true);
    });

    let callback = props.change_username_callback.clone();
    let apply_handler = is_editing_username_handler.clone();
    let cloned_new_username = new_username.clone();
    let on_aplly_usename = Callback::from(move |_: MouseEvent| {
        callback.emit(cloned_new_username.clone());
        apply_handler.set(false);
    });

    let cancel_handler = is_editing_username_handler.clone();
    let on_cancel_usename = Callback::from(move |_: MouseEvent| {
        cancel_handler.set(false);
    });

    html! {
        <div class="input-group">
            if is_editing_username {
                <input type="text" class="form-control" onchange={on_new_username} value={new_username} />
                <button class="btn btn-warning" onclick={on_aplly_usename}>
                    {"apply"}
                </button>
                <button class="btn btn-danger" onclick={on_cancel_usename}>
                    {"cancel"}
                </button>
            } else {
                <button class="btn btn-secondary" onclick={on_edit_usename}>
                    {props.username.clone()}
                </button>
                <span class="input-group-text">{"your message"}</span>
                <textarea class="form-control" onchange={on_new_message} value={new_message}></textarea>
                <button type="submit" class="btn btn-primary" onclick={on_submit}>
                    {"send"}
                </button>
            }
        </div>
    }
}
