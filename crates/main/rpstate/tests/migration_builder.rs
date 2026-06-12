use rpstate::store::builder::StoreBuilder;
use rpstate::{RpData, Store, migrate};
use rpstate_core::test_utils::unique_path;
use rpstate_macros::rpstate;

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

#[migrate]
#[rename(full_name => display_name)]
fn migrate_profile_v1_to_v2(old: RpData<v1::Profile>) -> rpstate::Result<RpData<v2::Profile>> {
    Ok(RpData::<v2::Profile> {
        display_name: old.full_name,
    })
}

#[test]
fn migration_builder_mixes_codegen_and_manual_steps() {
    let path = unique_path("migration-builder");

    {
        let store = StoreBuilder::new(&path).build().unwrap();
        let profile = v1::Profile::new_with(&store).unwrap();
        profile.full_name().set("Grace Hopper".to_string()).unwrap();
        profile.legacy_flag().set(true).unwrap();
    }

    let (store, _) = StoreBuilder::new(&path)
        .migrations(|m| {
            m.collect_codegen();
            m.for_node::<Profile>()
                .step(3, "derive initials after codegen migration", |ctx| {
                    let display_name = ctx
                        .get::<String>("display_name")?
                        .expect("codegen step should create display_name");
                    let initials = display_name
                        .split_whitespace()
                        .filter_map(|part| part.chars().next())
                        .collect::<String>();
                    ctx.set("initials", &initials)?;
                    Ok(())
                });
        })
        .build()
        .unwrap();

    let profile = Profile::new_with(&store).unwrap();
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
