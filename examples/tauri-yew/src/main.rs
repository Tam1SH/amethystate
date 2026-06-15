use crate::app::App;

mod app;
mod bindings;

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
    yew::Renderer::<App>::new().render();
}