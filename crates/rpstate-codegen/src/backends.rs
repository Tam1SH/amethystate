#[cfg(feature = "dioxus")]
pub struct TauriDioxusCodegen;
#[cfg(feature = "dioxus")]
impl crate::FrameworkCodegen for TauriDioxusCodegen {
    fn imports(&self) -> &str {
        "use rpstate_arena::rpstate_framework_arena;\n"
    }

    fn extra_attrs(&self) -> &[&str] {
        &["#[rpstate_framework_arena]"]
    }
}
#[cfg(feature = "leptos")]
pub struct TauriLeptosCodegen;
#[cfg(feature = "leptos")]
impl crate::FrameworkCodegen for TauriLeptosCodegen {
    fn imports(&self) -> &str {
        "use rpstate_arena::rpstate_framework_arena;\n"
    }

    fn extra_attrs(&self) -> &[&str] {
        &["#[rpstate_framework_arena]"]
    }
}

pub struct TauriVanillaCodegen;

impl crate::FrameworkCodegen for TauriVanillaCodegen {
    fn imports(&self) -> &str {
        "use rpstate::client::*;\n"
    }
}
