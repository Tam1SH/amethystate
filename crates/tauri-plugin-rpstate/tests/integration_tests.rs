use rpstate::{DefaultStore, Store, StoreBuilder, rpstate};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_rpstate::codegen;

#[rpstate]
pub struct TestNested {
    #[state(default = "nested_val".to_string())]
    pub name: String,
}

#[rpstate(prefix = "test_root")]
pub struct TestRoot {
    #[state(default = 42)]
    pub value: i32,

    #[state(default = "volatile_val".to_string(), volatile)]
    pub session: String,

    #[state(nested)]
    pub child: TestNested,
}

#[test]
fn test_schema_inventory_registrations() {
    let mut found_root = false;
    let mut found_nested = false;

    for entry in inventory::iter::<rpstate::tauri_codegen::SchemaExportEntry>() {
        if entry.struct_name == "TestRoot" {
            found_root = true;
            assert_eq!(entry.prefix, Some("test_root"));
            assert_eq!(entry.fields.len(), 3);
            assert_eq!(entry.fields[0].name, "value");
            assert_eq!(entry.fields[1].name, "session");
            assert_eq!(entry.fields[2].name, "child");
        } else if entry.struct_name == "TestNested" {
            found_nested = true;
            assert_eq!(entry.prefix, None);
            assert_eq!(entry.fields.len(), 1);
            assert_eq!(entry.fields[0].name, "name");
        }
    }

    assert!(found_root, "TestRoot was not registered in inventory!");
    assert!(found_nested, "TestNested was not registered in inventory!");
}

#[test]
fn test_typescript_codegen_export() {
    let out_path = std::env::temp_dir().join("rpstate_test_export.ts");
    if out_path.exists() {
        let _ = std::fs::remove_file(&out_path);
    }

    codegen::export(&out_path).expect("Failed to export TS bindings");

    assert!(out_path.exists());
    let ts_content = std::fs::read_to_string(&out_path).expect("Failed to read exported TS file");

    assert!(ts_content.contains("export class Field<T>"));
    assert!(ts_content.contains("export class ReadonlyField<T>"));
    assert!(ts_content.contains("export class ReactiveMapField<K extends string, V>"));

    assert!(ts_content.contains("\"test_root.value\": number;"));
    assert!(ts_content.contains("\"test_root.session\": string;"));
    assert!(ts_content.contains("\"test_root.child.name\": string;"));

    assert!(ts_content.contains("export class TestRoot {"));
    assert!(ts_content.contains("readonly value: Field<number>;"));
    assert!(ts_content.contains(
        "this.value = new Field<number>(\"test_root.value\", initialValues?.[\"test_root.value\"]);"
    ));
    assert!(ts_content.contains("readonly session: Field<string>;"));
    assert!(ts_content.contains("this.session = new Field<string>(\"test_root.session\", initialValues?.[\"test_root.session\"]);"));
    assert!(ts_content.contains("readonly child: TestNestedFields;"));
    assert!(
        ts_content
            .contains("this.child = new TestNestedFields(\"test_root.child\", initialValues);")
    );
    assert!(ts_content.contains("class TestNestedFields {"));
    assert!(ts_content.contains("readonly name: Field<string>;"));
    assert!(ts_content.contains(
        r#"this.name = new Field(`${prefix}.name`, initialValues?.[`${prefix}.name`]);"#
    ));
}

#[tokio::test]
async fn test_tauri_plugin_commands() {
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

    let val = tauri_plugin_rpstate::commands::rpstate_get(
        state_store.clone(),
        "test_root.value".to_string(),
    )
    .await;
    assert_eq!(val, Ok(Some(serde_json::json!(100))));

    let set_res = tauri_plugin_rpstate::commands::rpstate_set(
        state_store.clone(),
        "test_root.value".to_string(),
        serde_json::json!(200),
    )
    .await;
    assert_eq!(set_res, Ok(()));

    let updated_val: Option<i32> = store.get("test_root.value").unwrap();
    assert_eq!(updated_val, Some(200));
}
