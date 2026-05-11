use rpstate::store::builder::StoreBuilder;
use rpstate::{migrate, Store};
use rpstate_macros::rpstate;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

mod v1 {
    use super::*;

    #[rpstate(prefix = "hybrid_profile", version = 1)]
    pub struct Profile {
        #[state(default = "Ada Lovelace".to_string())]
        pub full_name: String,

        #[state(default = true)]
        pub legacy_flag: bool,
    }
}

mod v2 {
    use super::*;

    #[rpstate(prefix = "hybrid_profile", version = 2)]
    pub struct Profile {
        #[state(default = "Ada Lovelace".to_string())]
        pub display_name: String,
    }
}

#[rpstate(prefix = "hybrid_profile", version = 3)]
pub struct Profile {
    #[state(default = "Ada Lovelace".to_string())]
    pub display_name: String,

    #[state(default = "AL".to_string())]
    pub initials: String,
}

migrate! {
    v1::Profile_Data => v2::Profile_Data,
    rename: [full_name => display_name],
    |old| {
        Ok(Self {
            display_name: old.full_name,
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
fn migration_builder_mixes_codegen_and_manual_steps() {
    let path = unique_path("migration-builder");

    {
        let store = Arc::new(StoreBuilder::new(&path).build().unwrap());
        let profile = v1::Profile::new(&store).unwrap();
        profile.full_name().set("Grace Hopper".to_string()).unwrap();
        profile.legacy_flag().set(true).unwrap();
    }

    let store = Arc::new(
        StoreBuilder::new(&path)
            .migrations(|m| {
                m.collect_codegen();
                m.for_prefix("hybrid_profile").step(
                    3,
                    "derive initials after codegen migration",
                    |ctx| {
                        let display_name = ctx
                            .get::<String>("display_name")?
                            .expect("codegen step should create display_name");
                        let initials = display_name
                            .split_whitespace()
                            .filter_map(|part| part.chars().next())
                            .collect::<String>();
                        ctx.set("initials", &initials)?;
                        Ok(())
                    },
                );
            })
            .build()
            .unwrap(),
    );

    let profile = Profile::new(&store).unwrap();
    assert_eq!(profile.display_name().get(), "Grace Hopper");
    assert_eq!(profile.initials().get(), "GH");

    assert_eq!(
        store.get::<String>("hybrid_profile.full_name").unwrap(),
        None
    );
    assert_eq!(
        store.get::<bool>("hybrid_profile.legacy_flag").unwrap(),
        None
    );
}
