use leptos::prelude::*;
use leptos::task::spawn_local;
use tauri_plugin_rpstate::client::Pipeline;
use tauri_plugin_rpstate::client::{Field, IntoPipeline, MapChange, ReactiveMap};

use crate::bindings::{AppSettings, ThemeFields};
use shared::ProxyProfile;

fn use_rpstate_field<T>(field: Field<T>) -> (ReadSignal<T>, impl Fn(T) + Clone + 'static)
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
{
    let (read, write) = signal(field.value());
let path_sub = field.path.to_string();
    let sub = field.subscribe(move |val| {
        log::debug!("[subscribe] fired: {path_sub}");
        write.set(val);
    });
    on_cleanup(move || drop(sub));

    let field_setter = field.clone();
    let setter = move |val: T| {
        let f = field_setter.clone();
        log::debug!("[setter] called: {}", f.path);
        spawn_local(async move {
            match f.set(val).await {
                Ok(_) => log::debug!("[setter] ok: {}", f.path),
                Err(e) => log::error!("[setter] err: {}: {e}", f.path),
            }
        });
    };

    (read, setter)
}

fn use_rpstate_pipeline<T>(make: impl FnOnce() -> Pipeline<T>) -> ReadSignal<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    let pipeline = make();
    let (read, write) = signal(pipeline.get());

    let sub = pipeline.subscribe(move |val| {
        write.set(val);
    });
    on_cleanup(move || {
        drop(sub);
        drop(pipeline);
    });

    read
}

#[component]
fn EnvMapEditor(env: ReactiveMap<String, String>) -> impl IntoView {
    let (entries, set_entries) = signal(Vec::<(String, String)>::new());
    let (new_key, set_new_key) = signal(String::new());
    let (new_val, set_new_val) = signal(String::new());

    let sub = env.subscribe_any(move |change| {
        log::debug!("[env] change: {change:?}");
        match change {
            MapChange::Insert { key, value } | MapChange::Update { key, new_value: value, .. } => {
                set_entries.update(|e| {
                    if let Some(entry) = e.iter_mut().find(|(k, _)| k == key) {
                        entry.1 = value.clone();
                    } else {
                        e.push((key.clone(), value.clone()));
                    }
                });
            }
            MapChange::Remove { key, .. } => {
                set_entries.update(|e| e.retain(|(k, _)| k != key));
            }
            MapChange::Clear => set_entries.set(vec![]),
        }
    });
    on_cleanup(move || drop(sub));

    let env_add = env.clone();
    let on_add = move |_| {
        let key = new_key.get_untracked();
        let val = new_val.get_untracked();
        if key.is_empty() { return; }
        let env = env_add.clone();
        spawn_local(async move {
            let _ = env.set(key, &val).await;
        });
        set_new_key.set(String::new());
        set_new_val.set(String::new());
    };

    
    let env_init = env.clone();
    spawn_local(async move {
        if let Ok(map) = env_init.entries().await {
            set_entries.set(map.into_iter().collect());
        }
    });

    let env_remove = env.clone();
    let on_remove = move |key: String| {
        let env = env_remove.clone();
        spawn_local(async move {
            let _ = env.remove(key).await;
        });
    };


    view! {
        <div class="section">
            <h3>"Environment Variables"</h3>
            <For
                each=move || entries.get()
                key=|(k, _)| k.clone()
                children=move |(k, v)| {
                    let key = k.clone();
                    let on_rm = on_remove.clone();
                    view! {
                        <div class="env-row">
                            <code>{k}</code>
                            <span>" = "</span>
                            <code>{v}</code>
                            <button on:click=move |_| on_rm(key.clone())>"✕"</button>
                        </div>
                    }
                }
            />
            <div class="env-add">
                <input
                    placeholder="KEY"
                    prop:value=new_key
                    on:input=move |e| set_new_key.set(event_target_value(&e))
                />
                <input
                    placeholder="value"
                    prop:value=new_val
                    on:input=move |e| set_new_val.set(event_target_value(&e))
                />
                <button on:click=on_add>"Add"</button>
            </div>
        </div>
    }
}

#[component]
fn ThemeEditor(theme: ThemeFields) -> impl IntoView {
    let (mode, set_mode) = use_rpstate_field(theme.mode);
    let (bg, set_bg) = use_rpstate_field(theme.background);
    let (fg, set_fg) = use_rpstate_field(theme.foreground);

    view! {
        <div class="section">
            <h3>"Theme (nested)"</h3>
            <div class="field">
                <label>"Mode"</label>
                <select
                    prop:value=move || {
                        let v = mode.get();
                        log::debug!("[view] mode read: {v}");
                        v
                    }
                    on:change=move |e| set_mode(event_target_value(&e))
                >
                    <option value="light">"light"</option>
                    <option value="dark">"dark"</option>
                </select>
            </div>
            <div class="field">
                <label>"Background"</label>
                <input
                    type="color"
                    prop:value=bg
                    on:input=move |e| set_bg(event_target_value(&e))
                />
            </div>
            <div class="field">
                <label>"Foreground"</label>
                <input
                    type="color"
                    prop:value=fg
                    on:input=move |e| set_fg(event_target_value(&e))
                />
            </div>
        </div>
    }
}

#[component]
fn ProxyEditor(proxy: Field<ProxyProfile>) -> impl IntoView {
    let _intercept = proxy.intercept(|change| {
        if change.new_value.port == 0 { None } else { Some(change) }
    });
    on_cleanup(move || drop(_intercept));

    let (prof, set_prof) = use_rpstate_field(proxy);

    let set_name = set_prof.clone();
    let set_addr = set_prof.clone();
    let set_port = set_prof.clone();
    let set_enabled = set_prof.clone();

    view! {
        <div class="section">
            <h3>"Proxy (plain type)"</h3>
            <div class="field">
                <label>"Name"</label>
                <input
                    prop:value=move || prof.get().name
                    on:input=move |e| {
                        let mut p = prof.get_untracked();
                        p.name = event_target_value(&e);
                        set_name(p);
                    }
                />
            </div>
            <div class="field">
                <label>"Address"</label>
                <input
                    prop:value=move || prof.get().address
                    on:input=move |e| {
                        let mut p = prof.get_untracked();
                        p.address = event_target_value(&e);
                        set_addr(p);
                    }
                />
            </div>
            <div class="field">
                <label>"Port"</label>
                <input
                    type="number"
                    prop:value=move || prof.get().port.to_string()
                    on:input=move |e| {
                        if let Ok(port) = event_target_value(&e).parse::<u16>() {
                            let mut p = prof.get_untracked();
                            p.port = port;
                            set_port(p);
                        }
                    }
                />
            </div>
            <div class="field">
                <label>
                    <input
                        type="checkbox"
                        prop:checked=move || prof.get().enabled
                        on:change=move |e| {
                            let mut p = prof.get_untracked();
                            p.enabled = event_target_checked(&e);
                            set_enabled(p);
                        }
                    />
                    " Enabled"
                </label>
            </div>
            <p style=move || format!(
                "color: {}",
                if prof.get().enabled { "green" } else { "red" }
            )>
                {move || format!("{}:{} — {}",
                    prof.get().address,
                    prof.get().port,
                    if prof.get().enabled { "active" } else { "inactive" }
                )}
            </p>
        </div>
    }
}

#[component]
pub fn Settings(state: AppSettings) -> impl IntoView {
    let (username, set_username) = use_rpstate_field(state.username.clone());
    let (counter, set_counter) = use_rpstate_field(state.counter.clone());

    let address = use_rpstate_pipeline({
        let u = state.username.clone();
        let c = state.counter.clone();
        move || {
            (u, c).pipe()
                .map(|(u, c)| format!("{u}:{c}"))
                .dedupe()
        }
    });

    view! {
        <div>
            <h1>"rpstate + Leptos"</h1>

            <div class="section">
                <h3>"Basic fields"</h3>
                <div class="field">
                    <label>"Username"</label>
                    <input
                        prop:value=username
                        on:input=move |e| set_username(event_target_value(&e))
                    />
                </div>
                <div class="field">
                    <label>"Counter"</label>
                    <input
                        type="number"
                        prop:value=move || counter.get().to_string()
                        on:input=move |e| {
                            if let Ok(n) = event_target_value(&e).parse::<i32>() {
                                set_counter(n);
                            }
                        }
                    />
                </div>
                <p>"Pipeline → " <strong>{address}</strong></p>
            </div>

            <ThemeEditor theme=state.theme />
            <ProxyEditor proxy=state.proxy />
            <EnvMapEditor env=state.env />
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (state, set_state) = signal(None::<AppSettings>);

    Effect::new(move |_| {
        spawn_local(async move {
            match AppSettings::load().await {
                Ok(loaded) => set_state.set(Some(loaded)),
                Err(e) => log::error!("Failed to load: {e}"),
            }
        });
    });

    view! {
        {move || match state.get() {
            Some(s) => view! { <Settings state=s /> }.into_any(),
            None => view! { <p>"Loading..."</p> }.into_any(),
        }}
    }
}