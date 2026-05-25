use dioxus::prelude::*;
use futures_util::StreamExt;
use rpstate::store::builder::StoreBuilder;
use rpstate::{rpstate, DefaultStore, Field, IntoPipeline, Pipeline, WritableMode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc;

#[rpstate(prefix = "dioxus_settings")]
#[derive(PartialEq)]
pub struct SettingsState {
    #[state(default = "127.0.0.1".to_string())]
    pub host: String,

    #[state(default = 8080)]
    pub port: u16,

    #[state(default = true)]
    pub dark_mode: bool,
}

// ─── Hook: field ────────────────────────────────────────────────────────────
//
// Bridges an rpstate `Field<T>` to a read-only signal and a write closure.
// Uses an async channel to safely update the signal from any thread.
fn use_rpstate_field<T>(field: Field<T, DefaultStore, WritableMode>) -> (ReadSignal<T>, impl Fn(T))
where
    T: DeserializeOwned + Serialize + Clone + Send + Sync + PartialEq + 'static,
{
    let mut signal = use_signal(|| field.get());

    let tx = use_hook(|| {
        let (tx, mut rx) = mpsc::unbounded_channel::<T>();

        // This task runs on Dioxus's UI thread, ensuring zero concurrent access to the signal.
        spawn(async move {
            while let Some(val) = rx.recv().await {
                signal.set(val);
            }
        });

        tx
    });

    let field_for_sub = field.clone();

    use_hook(move || {
        let sub = field_for_sub.subscribe(move |val| {
            let _ = tx.send(val);
        });
        std::sync::Arc::new(sub)
    });

    let setter = move |val: T| {
        let _ = field.set(val);
    };

    (signal.into(), setter)
}

// ─── Hook: pipeline ──────────────────────────────────────────────────────────
//
// Bridges a read-only rpstate `Pipeline<T>` to a Dioxus signal.
// Keeps the pipeline and its subscription alive in a hook slot.
fn use_rpstate_pipeline<T>(make: impl FnOnce() -> Pipeline<T>) -> ReadSignal<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    let slot = use_hook(|| {
        let pipeline = make();
        let mut sig = Signal::new(pipeline.get());
        let (tx, mut rx) = mpsc::unbounded_channel::<T>();

        spawn(async move {
            while let Some(val) = rx.recv().await {
                sig.set(val);
            }
        });

        let sub = pipeline.subscribe(move |val| {
            let _ = tx.send(val);
        });

        (sig, std::sync::Arc::new((pipeline, sub)))
    });

    slot.0.into()
}

// ─── Component ───────────────────────────────────────────────────────────────

#[component]
fn Settings(state: SettingsState) -> Element {
    let (host, set_host) = use_rpstate_field(state.host());
    let (port, set_port) = use_rpstate_field(state.port());
    let (dark_mode, set_dark_mode) = use_rpstate_field(state.dark_mode());

    let address = use_rpstate_pipeline(|| {
        (state.host(), state.port())
            .pipe()
            .map(|(h, p)| format!("{h}:{p}"))
            .dedupe()
    });

    // ── Coroutine: message-driven external updates ────────────────────────
    // Simulates an external push source (e.g., WebSocket).
    let state_clone = state.clone();
    let port_tx = use_coroutine(move |mut rx: UnboundedReceiver<u16>| {
        let state = state_clone.clone();
        async move {
            while let Some(new_port) = rx.next().await {
                let _ = state.port().set(new_port);
            }
        }
    });

    rsx! {
        div {
            class: if *dark_mode.read() { "app dark" } else { "app light" },

            h1 { "rpstate + Dioxus" }

            div { class: "field",
                label { "Host" }
                input {
                    value: "{host}",
                    oninput: move |e| set_host(e.value()),
                }
            }

            div { class: "field",
                label { "Port" }
                input {
                    r#type: "number",
                    min: "1024",
                    max: "65535",
                    value: "{port}",
                    oninput: move |e| {
                        if let Ok(p) = e.value().parse::<u16>() {
                            set_port(p);
                        }
                    },
                }
            }

            div { class: "field",
                label {
                    input {
                        r#type: "checkbox",
                        checked: *dark_mode.read(),
                        onchange: move |e| set_dark_mode(e.checked()),
                    }
                    " dark mode"
                }
            }

            p { "derived: {address}" }

            button {
                onclick: move |_| port_tx.send(7777),
                "Simulate external push → port 7777"
            }
        }
    }
}

#[component]
fn App() -> Element {
    let state = use_context_provider(|| {
        let store = StoreBuilder::new("./dioxus-settings.redb")
            .build()
            .expect("failed to open store");
        SettingsState::new(&store).expect("failed to init state")
    });

    rsx! { Settings { state } }
}

fn main() {
    dioxus::launch(App);
}