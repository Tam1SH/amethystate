use rpstate::Store;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_tauri_plugin_commands() {
    use rpstate::{DefaultStore, StoreBuilder};
    use std::sync::Arc;
    use tauri::Manager;
    let db_path = std::env::temp_dir().join("rpstate_tauri_test_store.redb");
    if db_path.exists() {
        let _ = std::fs::remove_file(&db_path);
    }

    let store = StoreBuilder::new(&db_path).build().unwrap();

    store.set("test_root.value", &100i32).unwrap();
    store.save_now().unwrap();

    let app = tauri::test::mock_app();
    app.manage(store.clone());

    let state_store = app.state::<Arc<DefaultStore>>();

    let val = tauri_plugin_rpstate::backend::commands::rpstate_get(
        state_store.clone(),
        "test_root.value".to_string(),
    )
    .await;
    assert_eq!(val, Ok(Some(serde_json::json!(100))));

    let set_res = tauri_plugin_rpstate::backend::commands::rpstate_set(
        state_store.clone(),
        "test_root.value".to_string(),
        serde_json::json!(200),
    )
    .await;
    assert_eq!(set_res, Ok(()));

    let updated_val: Option<i32> = store.get("test_root.value").unwrap();
    assert_eq!(updated_val, Some(200));
}
