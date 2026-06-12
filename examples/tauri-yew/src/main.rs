use rpstate::{Field, IntoPipeline, Pipeline, ReactiveMap};
// FILE: C:\Users\ignat\projects\rpstate\examples\tauri-yew\src\main.rs
use std::collections::HashMap;
use yew::prelude::*;

// Импортируем структуры состояния из общей библиотеки (shared)
use shared::{AlertThresholds, NetworkState, SystemSettings};

// =========================================================================
// Реактивные хуки для rpstate & Yew (написаны на месте)
// =========================================================================

/// Хук для связывания поля `Field<T>` с локальным состоянием Yew.
/// Автоматически подписывается на изменения и обновляет компонент,
/// отменяя подписку при размонтировании.
#[hook]
pub fn use_rpstate_field<T>(field: Field<T>) -> (T, Callback<T>)
where
    T: Clone + 'static,
{
    let state = use_state(|| field.get());

    {
        let state = state.clone();
        let field = field.clone();
        use_effect_with((), move |_| {
            let subscription = field.subscribe(move |val| {
                state.set(val);
            });
            move || {
                // Subscription отписывается при Drop
                drop(subscription);
            }
        });
    }

    let setter = {
        let field = field.clone();
        Callback::from(move |val: T| {
            if let Err(err) = field.set(val) {
                log::error!("Failed to set rpstate field: {:?}", err);
            }
        })
    };

    ((*state).clone(), setter)
}

/// Хук для подписки компонента на реактивный конвейер (derived pipeline).
/// Возвращает текущее вычисленное значение.
#[hook]
pub fn use_rpstate_pipeline<T>(pipeline: Pipeline<T>) -> T
where
    T: Clone + 'static,
{
    let state = use_state(|| pipeline.get());

    {
        let state = state.clone();
        let pipeline = pipeline.clone();
        use_effect_with((), move |_| {
            let subscription = pipeline.subscribe(move |val| {
                state.set(val);
            });
            move || {
                drop(subscription);
            }
        });
    }

    (*state).clone()
}

/// Хук для работы с динамическими коллекциями `ReactiveMap<K, V>`.
/// Подписывается на любые изменения в карте и синхронизирует HashMap для рендеринга.
#[hook]
pub fn use_rpstate_map<K, V>(map: ReactiveMap<K, V>) -> HashMap<K, V>
where
    K: Clone + std::hash::Hash + Eq + 'static,
    V: Clone + 'static,
{
    let get_all_entries = |m: &ReactiveMap<K, V>| {
        m.entries()
            .unwrap_or_default()
            .into_iter()
            .collect::<HashMap<K, V>>()
    };

    let state = use_state(|| get_all_entries(&map));

    {
        let state = state.clone();
        let map = map.clone();
        use_effect_with((), move |_| {
            let subscription = map.subscribe_any(move |_change| {
                state.set(get_all_entries(&map));
            });
            move || {
                drop(subscription);
            }
        });
    }

    ((*state).clone())
}

// =========================================================================
// Компоненты интерфейса
// =========================================================================

#[derive(Properties, PartialEq, Clone)]
pub struct SettingsPanelProps {
    pub network_state: NetworkState,
    pub system_settings: SystemSettings,
}

#[function_component(SettingsPanel)]
pub fn settings_panel(props: &SettingsPanelProps) -> Html {
    // 1. Использование хука для простых реактивных полей
    let (host, set_host) = use_rpstate_field(props.network_state.host());
    let (port, set_port) = use_rpstate_field(props.network_state.port());

    // 2. Использование хука для вычисляемого конвейера (derived pipeline)
    let address_pipeline = (props.network_state.host(), props.network_state.port())
        .pipe()
        .map(|(h, p)| format!("{h}:{p}"));
    let address = use_rpstate_pipeline(address_pipeline);

    // 3. Использование хука для динамической карты ReactiveMap
    let limits = use_rpstate_map(props.system_settings.limits());

    // Обработчики ввода пользователя
    let on_host_input = {
        let set_host = set_host.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked();
            set_host.emit(input.value());
        })
    };

    let on_port_input = {
        let set_port = set_port.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked();
            if let Ok(p) = input.value().parse::<u16>() {
                set_port.emit(p);
            }
        })
    };

    // Добавление / обновление элемента в ReactiveMap
    let on_add_gpu_limit = {
        let limits_map = props.system_settings.limits().clone();
        Callback::from(move |_| {
            let new_threshold = AlertThresholds { warning: 75, critical: 95 };
            if let Err(e) = limits_map.set_or_create("gpu".to_string(), &new_threshold) {
                log::error!("Failed to update ReactiveMap: {:?}", e);
            }
        })
    };

    html! {
        <div class="container">
            <h1>{"Tauri + Yew + rpstate"}</h1>
            
            <div class="row">
                <!-- Сетевые настройки -->
                <div class="panel">
                    <h3>{"Network State (Fields)"}</h3>
                    <div style="margin-bottom: 15px;">
                        <label style="display: block;">{"Host"}</label>
                        <input type="text" value={host} oninput={on_host_input} />
                    </div>
                    <div style="margin-bottom: 15px;">
                        <label style="display: block;">{"Port"}</label>
                        <input type="number" value={port.to_string()} oninput={on_port_input} />
                    </div>
                    <hr />
                    <p>
                        <strong>{"Full Address (Derived Pipeline): "}</strong>
                        <code>{address}</code>
                    </p>
                </div>

                <!-- Настройки порогов (ReactiveMap) -->
                <div class="panel">
                    <h3>{"Alert Thresholds (ReactiveMap)"}</h3>
                    <ul>
                        {for limits.iter().map(|(key, threshold)| {
                            html! {
                                <li key={key.clone()}>
                                    <strong>{key}</strong>{": "}
                                    {"Warning "}{threshold.warning}{"%, "}
                                    {"Critical "}{threshold.critical}{"%"}
                                </li>
                            }
                        })}
                    </ul>
                    <button onclick={on_add_gpu_limit}>{"Add or Reset GPU threshold"}</button>
                </div>
            </div>
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(|| None::<(NetworkState, SystemSettings)>);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Инициализация клиентского хранилища rpstate, 
                // работающего поверх Tauri IPC.
                match rpstate_tauri::ClientStore::new().await {
                    Ok(client_store) => {
                        let net_state = NetworkState::new(&client_store).unwrap();
                        let sys_settings = SystemSettings::new(&client_store).unwrap();
                        state.set(Some((net_state, sys_settings)));
                    }
                    Err(err) => {
                        log::error!("Failed to connect to tauri rpstate store: {:?}", err);
                    }
                }
            });
            || ()
        });
    }

    match &*state {
        Some((net_state, sys_settings)) => {
            html! {
                <SettingsPanel 
                    network_state={net_state.clone()} 
                    system_settings={sys_settings.clone()} 
                />
            }
        }
        None => {
            html! {
                <div class="container">
                    <p>{"Connecting to Tauri rpstate backend..."}</p>
                </div>
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
    yew::Renderer::<App>::new().render();
}