#[cfg(feature = "dioxus")]
pub struct TauriDioxusCodegen;
#[cfg(feature = "dioxus")]
impl crate::FrameworkCodegen for TauriDioxusCodegen {
    fn imports(&self) -> &str {
        "use amethystate_arena::amethystate_framework_arena;\n"
    }

    fn extra_attrs(&self) -> &[&str] {
        &["#[amethystate_framework_arena]"]
    }
}
#[cfg(feature = "leptos")]
pub struct TauriLeptosCodegen;
#[cfg(feature = "leptos")]
impl crate::FrameworkCodegen for TauriLeptosCodegen {
    fn imports(&self) -> &str {
        "use amethystate_arena::amethystate_framework_arena;\n"
    }

    fn extra_attrs(&self) -> &[&str] {
        &["#[amethystate_framework_arena]"]
    }
}

#[cfg(feature = "yew")]
pub struct TauriYewCodegen;
#[cfg(feature = "yew")]
impl crate::FrameworkCodegen for TauriYewCodegen {}

pub struct TauriVanillaCodegen;

impl crate::FrameworkCodegen for TauriVanillaCodegen {}
