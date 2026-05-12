use rpstate::store::builder::StoreBuilder;
use rpstate::{MigrationError, Store, migrate};
use rpstate_macros::rpstate;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

mod identity_v1 {
    use super::*;

    #[rpstate(prefix = "complex_identity", version = 1)]
    pub struct Identity {
        #[state(default = "guest".to_string())]
        pub login: String,

        #[state(default = "free".to_string())]
        pub tier: String,

        #[state(default = "legacy-token".to_string())]
        pub legacy_token: String,
    }
}

#[rpstate(prefix = "complex_identity", version = 2)]
pub struct Identity {
    #[state(default = "guest".to_string())]
    pub username: String,

    #[state(default = "free".to_string())]
    pub plan: String,

    #[state(default = 0u64)]
    pub created_at_ms: u64,
}

mod workspace_v1 {
    use super::*;

    #[rpstate(prefix = "complex_workspace", version = 1)]
    pub struct Workspace {
        #[state(default = "Untitled".to_string())]
        pub title: String,

        #[state(default = "dark".to_string())]
        pub theme: String,

        #[state(default = true)]
        pub stale_flag: bool,
    }
}

mod workspace_v2 {
    use super::*;

    #[rpstate(prefix = "complex_workspace", version = 2)]
    pub struct Workspace {
        #[state(default = "Untitled".to_string())]
        pub name: String,

        #[state(default = "dark".to_string())]
        pub appearance_theme: String,
    }
}

#[rpstate(prefix = "complex_workspace", version = 3)]
pub struct Workspace {
    #[state(default = "Untitled".to_string())]
    pub name: String,

    #[state(default = "dark".to_string())]
    pub appearance_theme: String,

    #[state(default = "Welcome".to_string())]
    pub welcome_title: String,
}

mod telemetry_v1 {
    use super::*;

    #[rpstate(prefix = "complex_telemetry", version = 1)]
    pub struct Telemetry {
        #[state(default = true)]
        pub enabled: bool,

        #[state(default = 5u16)]
        pub sample_rate: u16,
    }
}

#[rpstate(prefix = "complex_telemetry", version = 2)]
pub struct Telemetry {
    #[state(default = true)]
    pub enabled: bool,

    #[state(default = 50u16)]
    pub sample_rate_per_mille: u16,
}

mod profile_v1 {
    use super::*;

    #[rpstate(prefix = "complex_profile", version = 1)]
    pub struct Profile {
        #[state(default = "".to_string())]
        pub full_name: String,

        #[state(default = "".to_string())]
        pub age_text: String,
    }
}

#[rpstate(prefix = "complex_profile", version = 2)]
pub struct Profile {
    #[state(default = "".to_string())]
    pub first_name: String,

    #[state(default = "".to_string())]
    pub last_name: String,

    #[state(default = None::<u8>)]
    pub age: Option<u8>,

    #[state(default = "free".to_string())]
    pub plan_snapshot: String,
}

mod ui_v1 {
    use super::*;

    #[rpstate(prefix = "complex_ui", version = 1)]
    pub struct Ui {
        #[state(default = 0u16)]
        pub sidebar_px: u16,

        #[state(default = 1u16)]
        pub width_px: u16,

        #[state(key = "panels.left.visible", default = true)]
        pub left_panel_visible: bool,
    }
}

#[rpstate(prefix = "complex_ui", version = 2)]
pub struct Ui {
    #[state(default = 0.25f32)]
    pub sidebar_ratio: f32,

    #[state(default = true)]
    pub left_panel_visible: bool,
}

mod shortcuts_v1 {
    use super::*;

    #[rpstate(prefix = "complex_shortcuts", version = 1)]
    pub struct Shortcuts {
        #[state(default = Vec::<String>::new())]
        pub legacy_bindings: Vec<String>,
    }
}

#[rpstate(prefix = "complex_shortcuts", version = 2)]
pub struct Shortcuts {
    #[state(default = Vec::<String>::new())]
    pub bindings: Vec<String>,
}

mod broken_root_v1 {
    use super::*;

    #[rpstate(prefix = "complex_broken_root", version = 1)]
    pub struct BrokenRoot {
        #[state(default = "stable".to_string())]
        pub original: String,
    }
}

#[rpstate(prefix = "complex_broken_root", version = 2)]
pub struct BrokenRoot {
    #[state(default = "stable".to_string())]
    pub original: String,

    #[state(default = false)]
    pub staged: bool,
}

mod broken_child_v1 {
    use super::*;

    #[rpstate(prefix = "complex_broken_child", version = 1)]
    pub struct BrokenChild {
        #[state(default = false)]
        pub fail: bool,
    }
}

#[rpstate(prefix = "complex_broken_child", version = 2)]
pub struct BrokenChild {
    #[state(default = false)]
    pub fail: bool,
}

migrate! {
    identity_v1::Identity_Data => Identity_Data,
    rename: [login => username, tier => plan],
    |old| {
        Ok(Self {
            username: old.login,
            plan: match old.tier.as_str() {
                "pro" => "professional".to_string(),
                other => other.to_string(),
            },
            created_at_ms: 1_700_000_000_000,
        })
    }
}

migrate! {
    workspace_v1::Workspace_Data => workspace_v2::Workspace_Data,
    rename: [title => name, theme => appearance_theme],
    |old| {
        Ok(Self {
            name: old.title,
            appearance_theme: old.theme,
        })
    }
}

migrate! {
    telemetry_v1::Telemetry_Data => Telemetry_Data,
    rename:[sample_rate => sample_rate_per_mille],
    |old| {
        Ok(Self {
            enabled: old.enabled,
            sample_rate_per_mille: old.sample_rate.saturating_mul(10),
        })
    }
}

fn unique_path(suffix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time is after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("rpstate-{suffix}-{nanos}.redb"))
}

#[test]
fn complex_hybrid_migrations_handle_dependency_tree_and_rollback() {
    let path = unique_path("complex-migration");

    {
        let store = Arc::new(StoreBuilder::new(&path).build().unwrap());

        let identity = identity_v1::Identity::new(&store).unwrap();
        identity.login().set("ignat".to_string()).unwrap();
        identity.tier().set("pro".to_string()).unwrap();
        identity
            .legacy_token()
            .set("remove-me".to_string())
            .unwrap();

        let workspace = workspace_v1::Workspace::new(&store).unwrap();
        workspace
            .title()
            .set("Analytical Engine".to_string())
            .unwrap();
        workspace.theme().set("solarized".to_string()).unwrap();
        workspace.stale_flag().set(true).unwrap();

        let telemetry = telemetry_v1::Telemetry::new(&store).unwrap();
        telemetry.enabled().set(true).unwrap();
        telemetry.sample_rate().set(7u16).unwrap();

        let profile = profile_v1::Profile::new(&store).unwrap();
        profile.full_name().set("Ada Lovelace".to_string()).unwrap();
        profile.age_text().set("36".to_string()).unwrap();

        let ui = ui_v1::Ui::new(&store).unwrap();
        ui.sidebar_px().set(320u16).unwrap();
        ui.width_px().set(1280u16).unwrap();
        ui.left_panel_visible().set(false).unwrap();

        let shortcuts = shortcuts_v1::Shortcuts::new(&store).unwrap();
        shortcuts
            .legacy_bindings()
            .set(vec!["save=Ctrl+S".to_string(), "open=Ctrl+O".to_string()])
            .unwrap();

        let broken_root = broken_root_v1::BrokenRoot::new(&store).unwrap();
        broken_root.original().set("stable".to_string()).unwrap();

        let _broken_child = broken_child_v1::BrokenChild::new(&store).unwrap();
    }

    let store = Arc::new(
        StoreBuilder::new(&path)
            .migrations(|m| {
                m.collect_codegen();

                m.for_node::<Profile>().depends_on::<Identity>().step(
                    2,
                    "split full name and snapshot plan",
                    |ctx| {
                        let full_name = ctx
                            .get::<String>("full_name")?
                            .expect("seed should contain profile full_name");
                        let mut parts = full_name.splitn(2, ' ');
                        let first_name = parts.next().unwrap_or_default().to_string();
                        let last_name = parts.next().unwrap_or_default().to_string();
                        let age = ctx
                            .get::<String>("age_text")?
                            .and_then(|value| value.parse::<u8>().ok());
                        let plan_snapshot = ctx
                            .global_get::<String>("complex_identity.plan")?
                            .expect("identity codegen migration should run first");

                        ctx.set("first_name", &first_name)?;
                        ctx.set("last_name", &last_name)?;
                        ctx.set("age", &age)?;
                        ctx.set("plan_snapshot", &plan_snapshot)?;
                        ctx.delete("full_name")?;
                        ctx.delete("age_text")?;
                        Ok(())
                    },
                );

                m.for_node::<Workspace>().depends_on::<Profile>().step(
                    3,
                    "derive welcome title after profile migration",
                    |ctx| {
                        let name = ctx
                            .get::<String>("name")?
                            .expect("workspace codegen migration should create name");
                        let first_name = ctx
                            .global_get::<String>("complex_profile.first_name")?
                            .expect("profile migration should create first_name");
                        let welcome_title = format!("{name} for {first_name}");
                        ctx.set("welcome_title", &welcome_title)?;
                        Ok(())
                    },
                );

                m.for_node::<Ui>().depends_on::<Workspace>().step(
                    2,
                    "flatten panel state and normalize sidebar",
                    |ctx| {
                        let sidebar_px = ctx.get::<u16>("sidebar_px")?.unwrap_or(0);
                        let width_px = ctx.get::<u16>("width_px")?.unwrap_or(1);
                        let sidebar_ratio = sidebar_px as f32 / width_px as f32;
                        let left_panel_visible =
                            ctx.get::<bool>("panels.left.visible")?.unwrap_or(true);
                        ctx.set("sidebar_ratio", &sidebar_ratio)?;
                        ctx.set("left_panel_visible", &left_panel_visible)?;
                        ctx.delete("sidebar_px")?;
                        ctx.delete("width_px")?;
                        ctx.delete("panels.left.visible")?;
                        Ok(())
                    },
                );

                m.for_node::<Shortcuts>().depends_on::<Workspace>().step(
                    2,
                    "parse legacy shortcut bindings",
                    |ctx| {
                        let legacy = ctx
                            .get::<Vec<String>>("legacy_bindings")?
                            .unwrap_or_default();
                        let mut bindings = legacy
                            .into_iter()
                            .filter_map(|entry| {
                                let (action, binding) = entry.split_once('=')?;
                                Some(format!("{action}:{binding}"))
                            })
                            .collect::<Vec<_>>();
                        bindings.sort();
                        ctx.set("bindings", &bindings)?;
                        ctx.delete("legacy_bindings")?;
                        Ok(())
                    },
                );

                m.for_node::<BrokenRoot>()
                    .step(2, "stage broken branch mutation", |ctx| {
                        ctx.set("original", &"mutated".to_string())?;
                        ctx.set("staged", &true)?;
                        Ok(())
                    });

                m.for_node::<BrokenChild>().depends_on::<BrokenRoot>().step(
                    2,
                    "fail broken branch",
                    |_| Err(MigrationError::Custom("intentional failure".into()).into()),
                );
            })
            .build()
            .unwrap(),
    );

    let identity = Identity::new(&store).unwrap();
    // AI-Doxxed-Driven Development
    assert_eq!(identity.username().get(), "ignat");
    assert_eq!(identity.plan().get(), "professional");
    assert_eq!(identity.created_at_ms().get(), 1_700_000_000_000);

    let profile = Profile::new(&store).unwrap();
    assert_eq!(profile.first_name().get(), "Ada");
    assert_eq!(profile.last_name().get(), "Lovelace");
    assert_eq!(profile.age().get(), Some(36));
    assert_eq!(profile.plan_snapshot().get(), "professional");

    let workspace = Workspace::new(&store).unwrap();
    assert_eq!(workspace.name().get(), "Analytical Engine");
    assert_eq!(workspace.appearance_theme().get(), "solarized");
    assert_eq!(workspace.welcome_title().get(), "Analytical Engine for Ada");

    let ui = Ui::new(&store).unwrap();
    assert!((ui.sidebar_ratio().get() - 0.25).abs() < f32::EPSILON);
    assert!(!ui.left_panel_visible().get());

    let shortcuts = Shortcuts::new(&store).unwrap();
    assert_eq!(
        shortcuts.bindings().get(),
        vec!["open:Ctrl+O".to_string(), "save:Ctrl+S".to_string()]
    );

    let telemetry = Telemetry::new(&store).unwrap();
    assert!(telemetry.enabled().get());
    assert_eq!(telemetry.sample_rate_per_mille().get(), 70);

    assert_eq!(store.get::<String>("complex_identity.login").unwrap(), None);
    assert_eq!(
        store
            .get::<String>("complex_identity.legacy_token")
            .unwrap(),
        None
    );
    assert_eq!(
        store.get::<String>("complex_profile.full_name").unwrap(),
        None
    );
    assert_eq!(
        store.get::<String>("complex_profile.age_text").unwrap(),
        None
    );
    assert_eq!(
        store.get::<String>("complex_workspace.title").unwrap(),
        None
    );
    assert_eq!(
        store.get::<String>("complex_workspace.stale_flag").unwrap(),
        None
    );
    assert_eq!(store.get::<u16>("complex_ui.sidebar_px").unwrap(), None);
    assert_eq!(
        store
            .get::<Vec<String>>("complex_shortcuts.legacy_bindings")
            .unwrap(),
        None
    );
    assert_eq!(
        store.get::<u16>("complex_telemetry.sample_rate").unwrap(),
        None
    );

    assert_eq!(
        store.get::<String>("complex_broken_root.original").unwrap(),
        Some("stable".to_string())
    );
    assert_eq!(
        store.get::<bool>("complex_broken_root.staged").unwrap(),
        None
    );
}
