mod backends;
pub use backends::*;

use heck::ToLowerCamelCase;
use rpstate_core::{FieldExportMeta, FieldKind, SchemaExportEntry};
use std::collections::BTreeMap;
use std::path::Path;

pub trait FrameworkCodegen {
    fn imports(&self) -> &str {
        ""
    }
    fn extra_derives(&self) -> &[&str] {
        &[]
    }
    fn extra_attrs(&self) -> &[&str] {
        &[]
    }
}

pub struct CodegenRegistry {
    registry: BTreeMap<&'static str, &'static SchemaExportEntry>,
}

impl CodegenRegistry {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut registry = BTreeMap::new();
        for entry in inventory::iter::<SchemaExportEntry> {
            registry.insert(entry.struct_name, entry);
        }
        Self { registry }
    }

    pub fn export_ts(&self, out_path: impl AsRef<Path>) -> std::io::Result<()> {
        let mut ts = String::new();
        ts.push_str("/* eslint-disable */\n/* tslint:disable */\n// @ts-nocheck\n");
        ts.push_str(
            r#"// src/bindings/rpstate.ts DO NOT EDIT
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";


export type MapChange<K, V> =
    | { type: "Insert"; key: K; value: V }
    | { type: "Update"; key: K; oldValue: V; newValue: V }
    | { type: "Remove"; key: K; oldValue: V }
    | { type: "Clear" };

export class Field<T> {
    private _value: T | null = null;
    private _unlisten: (() => void) | null = null;

    constructor(public readonly key: string, initialValue?: T) {
        if (initialValue !== undefined) {
            this._value = initialValue;
        } else {
            this.get()
                .then((val) => {
                    this._value = val;
                })
                .catch(() => {});
        }

        this._unlisten = this.subscribe((val) => {
            this._value = val;
        });
    }

    /**
     * Synchronous in-memory getter.
     *
     * @returns The locally cached value.
     * @tradeoff Resolved in-memory. Might not reflect the actual persistent store
     * if background sync is pending or failed. Use `get()` for transaction-safe checks.
     */
    get value(): T | null {
        return this._value;
    }

    /**
     * Synchronous in-memory setter.
     *
     * @param newValue The new value to assign.
     * @tradeoff Immediately updates the local cache to keep the UI lag-free,
     * while firing an asynchronous write in the background. Note that writes are debounced/buffered;
     * call and await the `save()` method on the parent slice class to guarantee immediate disk persistence.
     */
    set value(newValue: T) {
        this._value = newValue;
        this.set(newValue).catch((err) => {
            console.error(`Sync write failed for key ${this.key}:`, err);
        });
    }

    /**
     * Absolute asynchronous getter.
     *
     * @returns A promise resolving to the most up-to-date value queried directly from the persistent store.
     * @benefit Transaction-safe. Guarantees that the retrieved value is persisted on disk.
     */
    async get(): Promise<T> {
        return invoke("plugin:rpstate|rpstate_get", { key: this.key });
    }

    /**
     * Absolute asynchronous setter.
     *
     * @param value The value to persist.
     * @returns A promise resolving when the value is queued for writing in the persistent store.
     * @note Writes are debounced/buffered. To guarantee immediate persistence on disk,
     * call and await the `save()` method on the parent slice class.
     */
    async set(value: T): Promise<void> {
        return invoke("plugin:rpstate|rpstate_set", { key: this.key, value });
    }

    destroy() {
        if (this._unlisten) {
            this._unlisten();
        }
    }

    subscribe(cb: (value: T) => void): () => void {
        invoke("plugin:rpstate|rpstate_subscribe", { key: this.key });
        let unlisten: (() => void) | null = null;
        let cancelled = false;

        const channel = `rpstate://${this.key.replace(/\./g, ":")}`;

        listen<T>(channel, (e) => cb(e.payload))
            .then((fn) => {
                if (cancelled) {
                    fn();
                    invoke("plugin:rpstate|rpstate_unsubscribe", { key: this.key });
                } else {
                    unlisten = fn;
                }
            });

        return () => {
            cancelled = true;
            if (unlisten) {
                unlisten();
                invoke("plugin:rpstate|rpstate_unsubscribe", { key: this.key });
            }
        };
    }
}

export class ReadonlyField<T> {
    private _value: T | null = null;
    private _unlisten: (() => void) | null = null;

    constructor(public readonly key: string, initialValue?: T) {
        if (initialValue !== undefined) {
            this._value = initialValue;
        } else {
            this.get()
                .then((val) => {
                    this._value = val;
                })
                .catch(() => {});
        }

        this._unlisten = this.subscribe((val) => {
            this._value = val;
        });
    }

    /**
     * Synchronous in-memory getter.
     *
     * @returns The locally cached value.
     * @tradeoff Resolved in-memory. Might not reflect the actual persistent store
     * if background sync is pending or failed. Use `get()` for transaction-safe checks.
     */
    get value(): T | null {
        return this._value;
    }

    /**
     * Absolute asynchronous getter.
     *
     * @returns A promise resolving to the most up-to-date value queried directly from the persistent store.
     * @benefit Transaction-safe. Guarantees that the retrieved value is persisted on disk.
     */
    async get(): Promise<T> {
        return invoke("plugin:rpstate|rpstate_get", { key: this.key });
    }

    destroy() {
        if (this._unlisten) {
            this._unlisten();
        }
    }

    subscribe(cb: (value: T) => void): () => void {
        invoke("plugin:rpstate|rpstate_subscribe", { key: this.key });
        let unlisten: (() => void) | null = null;
        let cancelled = false;

        const channel = `rpstate://${this.key.replace(/\./g, ":")}`;

        listen<T>(channel, (e) => cb(e.payload))
            .then((fn) => {
                if (cancelled) {
                    fn();
                    invoke("plugin:rpstate|rpstate_unsubscribe", { key: this.key });
                } else {
                    unlisten = fn;
                }
            });

        return () => {
            cancelled = true;
            if (unlisten) {
                unlisten();
                invoke("plugin:rpstate|rpstate_unsubscribe", { key: this.key });
            }
        };
    }
}

export class ReactiveMapField<K extends string, V> {
    private _map = new Map<K, V>();
    private _unlisten: (() => void) | null = null;

    constructor(public readonly prefix: string, initialValues?: Record<string, any>) {
        if (initialValues) {
            const dotPrefix = `${this.prefix}.`;
            for (const [key, value] of Object.entries(initialValues)) {
                if (key.startsWith(dotPrefix)) {
                    const subKey = key.slice(dotPrefix.length) as K;
                    this._map.set(subKey, value);
                }
            }
        }

        this._unlisten = this.subscribeAny((change) => {
            if (change.type === "Insert") {
                this._map.set(change.key, change.value);
            } else if (change.type === "Update") {
                this._map.set(change.key, change.newValue);
            } else if (change.type === "Remove") {
                this._map.delete(change.key);
            } else if (change.type === "Clear") {
                this._map.clear();
            }
        });
    }

    destroy() {
        if (this._unlisten) {
            this._unlisten();
        }
    }

    /**
     * Absolute asynchronous getter.
     *
     * @param key The map key to look up.
     * @returns A promise resolving to the value queried directly from the backend.
     * @benefit Transaction-safe. Guarantees data fresh from the persistent store.
     */
    async get(key: K): Promise<V | null> {
        return invoke("plugin:rpstate|rpstate_get", { key: `${this.prefix}.${key}` });
    }

    /**
     * Absolute asynchronous setter.
     *
     * @param key The map key to assign.
     * @param value The value to persist.
     * @returns A promise resolving when the value is queued for writing.
     * @note Writes are debounced/buffered. To guarantee immediate persistence on disk,
     * call and await the `save()` method on the parent slice class.
     */
    async set(key: K, value: V): Promise<void> {
        return invoke("plugin:rpstate|rpstate_set", { key: `${this.prefix}.${key}`, value });
    }

    /**
     * Synchronous in-memory getter.
     *
     * @param key The map key to look up.
     * @returns The locally cached value.
     * @tradeoff Resolved in-memory. Might lag behind actual persistent store transactions. Use `get(key)` for absolute checks.
     */
    getSync(key: K): V | null {
        return this._map.get(key) ?? null;
    }

    /**
     * Synchronous in-memory setter.
     *
     * @param key The map key to assign.
     * @param value The value to write.
     * @tradeoff Instantly updates the local memory map while initiating background write.
     * Writes are debounced/buffered; call and await `save()` on the parent slice class to flush changes to disk.
     */
    setSync(key: K, value: V): void {
        this._map.set(key, value);
        this.set(key, value).catch((err) => {
            console.error(`Sync map write failed for ${this.prefix}.${key}:`, err);
        });
    }

    /**
     * Synchronous in-memory key lookup.
     *
     * @param key The map key to check.
     * @returns True if the key is present in the local cache.
     * @tradeoff Resolved in-memory. Might not reflect pending backend transactions.
     */
    hasSync(key: K): boolean {
        return this._map.has(key);
    }

    /**
     * Returns the read-only, native JavaScript Map entries currently synchronized.
     */
    get entries(): ReadonlyMap<K, V> {
        return this._map;
    }

    subscribeKey(key: K, cb: (value: V) => void): () => void {
        const fullKey = `${this.prefix}.${key}`;
        invoke("plugin:rpstate|rpstate_subscribe", { key: fullKey });
        let unlisten: (() => void) | null = null;
        let cancelled = false;

        const channel = `rpstate://${fullKey.replace(/\./g, ":")}`;

        listen<V>(channel, (e) => cb(e.payload))
            .then((fn) => {
                if (cancelled) {
                    fn();
                    invoke("plugin:rpstate|rpstate_unsubscribe", { key: fullKey });
                } else {
                    unlisten = fn;
                }
            });

        return () => {
            cancelled = true;
            if (unlisten) {
                unlisten();
                invoke("plugin:rpstate|rpstate_unsubscribe", { key: fullKey });
            }
        };
    }

    subscribeAny(cb: (change: MapChange<K, V>) => void): () => void {
        const fullKey = this.prefix;
        invoke("plugin:rpstate|rpstate_subscribe", { key: fullKey });
        let unlisten: (() => void) | null = null;
        let cancelled = false;

        const channel = `rpstate://${fullKey.replace(/\./g, ":")}`;

        listen<MapChange<K, V>>(channel, (e) => cb(e.payload))
            .then((fn) => {
                if (cancelled) {
                    fn();
                    invoke("plugin:rpstate|rpstate_unsubscribe", { key: fullKey });
                } else {
                    unlisten = fn;
                }
            });

        return () => {
            cancelled = true;
            if (unlisten) {
                unlisten();
                invoke("plugin:rpstate|rpstate_unsubscribe", { key: fullKey });
            }
        };
    }
}

"#,
        );

        let mut schema_lines = Vec::new();
        for entry in self.registry.values() {
            if let Some(prefix) = entry.prefix {
                let mut resolved = Vec::new();
                self.resolve_fields_ts(prefix, entry.fields, &mut resolved);
                for (key, ts_type, comment) in resolved {
                    if let Some(cmt) = comment {
                        schema_lines
                            .push(format!("    /** {} */\n    \"{}\": {};", cmt, key, ts_type));
                    } else {
                        schema_lines.push(format!("    \"{}\": {};", key, ts_type));
                    }
                }
            }
        }

        ts.push_str("export type StateSchema = {\n");
        for line in schema_lines {
            ts.push_str(&line);
            ts.push('\n');
        }
        ts.push_str("};\n\n");

        let mut nested_classes = String::new();
        let mut root_classes = String::new();

        for entry in self.registry.values() {
            match entry.prefix {
                None => {
                    nested_classes.push_str(&format!("class {}Fields {{\n", entry.struct_name));
                    for field in entry.fields {
                        let prop_name = field.name.to_lower_camel_case();
                        let prop_type = match &field.kind {
                            FieldKind::Plain | FieldKind::Volatile => {
                                format!("Field<{}>", field.full_ts_type)
                            }
                            FieldKind::Nested { struct_name } => format!("{}Fields", struct_name),
                            FieldKind::Lookup { mutable, .. } => {
                                if *mutable {
                                    format!("Field<{}>", field.full_ts_type)
                                } else {
                                    format!("ReadonlyField<{}>", field.full_ts_type)
                                }
                            }
                            FieldKind::LookupNode { struct_name, .. } => {
                                format!("{}Fields", struct_name)
                            }
                            FieldKind::ReactiveMap {
                                key_type,
                                value_type,
                                ..
                            } => {
                                format!("ReactiveMapField<{}, {}>", key_type, value_type)
                            }
                        };
                        nested_classes
                            .push_str(&format!("    readonly {}: {};\n", prop_name, prop_type));
                    }

                    nested_classes.push_str(
                        "    constructor(prefix: string, initialValues?: Record<string, any>) {\n",
                    );
                    for field in entry.fields {
                        let prop_name = field.name.to_lower_camel_case();
                        match &field.kind {
                            FieldKind::Plain | FieldKind::Volatile => {
                                nested_classes.push_str(&format!(
                                    "        this.{} = new Field(`${{prefix}}.{}`, initialValues?.[`${{prefix}}.{}`]);\n",
                                    prop_name, field.name, field.name
                                ));
                            }
                            FieldKind::Nested { struct_name } => {
                                nested_classes.push_str(&format!(
                                    "        this.{} = new {}Fields(`${{prefix}}.{}`, initialValues);\n",
                                    prop_name, struct_name, field.name
                                ));
                            }
                            FieldKind::Lookup {
                                target_key,
                                mutable,
                            } => {
                                let class_name = if *mutable { "Field" } else { "ReadonlyField" };
                                nested_classes.push_str(&format!(
                                    "        this.{} = new {}<{}>(\"{}\", initialValues?.[\"{}\"]);\n",
                                    prop_name, class_name, field.full_ts_type, target_key, target_key
                                ));
                            }
                            FieldKind::LookupNode {
                                target_prefix,
                                struct_name,
                            } => {
                                nested_classes.push_str(&format!(
                                    "        this.{} = new {}Fields(\"{}\", initialValues);\n",
                                    prop_name, struct_name, target_prefix
                                ));
                            }
                            FieldKind::ReactiveMap {
                                key_type,
                                value_type,
                                ..
                            } => {
                                nested_classes.push_str(&format!(
                                    "        this.{} = new ReactiveMapField<{}, {}>(`${{prefix}}.{}`, initialValues);\n",
                                    prop_name, key_type, value_type, field.name
                                ));
                            }
                        }
                    }
                    nested_classes.push_str("    }\n}\n\n");
                }
                Some(prefix) => {
                    let mut resolved = Vec::new();
                    self.resolve_fields_ts(prefix, entry.fields, &mut resolved);

                    let schema_name = format!("{}Schema", entry.struct_name);
                    root_classes.push_str(&format!("export type {} = {{\n", schema_name));
                    for (key, ts_type, comment) in &resolved {
                        if let Some(cmt) = comment {
                            root_classes.push_str(&format!(
                                "    /** {} */\n    \"{}\": {};\n",
                                cmt, key, ts_type
                            ));
                        } else {
                            root_classes.push_str(&format!("    \"{}\": {};\n", key, ts_type));
                        }
                    }
                    root_classes.push_str("};\n\n");

                    root_classes.push_str(&format!("export class {} {{\n", entry.struct_name));

                    for field in entry.fields {
                        let prop_name = field.name.to_lower_camel_case();
                        let prop_type = match &field.kind {
                            FieldKind::Plain | FieldKind::Volatile => {
                                format!("Field<{}>", field.full_ts_type)
                            }
                            FieldKind::Nested { struct_name } => format!("{}Fields", struct_name),
                            FieldKind::Lookup { mutable, .. } => {
                                if *mutable {
                                    format!("Field<{}>", field.full_ts_type)
                                } else {
                                    format!("ReadonlyField<{}>", field.full_ts_type)
                                }
                            }
                            FieldKind::LookupNode { struct_name, .. } => {
                                format!("{}Fields", struct_name)
                            }
                            FieldKind::ReactiveMap {
                                key_type,
                                value_type,
                                ..
                            } => {
                                format!("ReactiveMapField<{}, {}>", key_type, value_type)
                            }
                        };
                        root_classes
                            .push_str(&format!("    readonly {}: {};\n", prop_name, prop_type));
                    }

                    root_classes.push_str(&format!(
                        "    constructor(initialValues?: Partial<{}>) {{\n",
                        schema_name
                    ));
                    for field in entry.fields {
                        let prop_name = field.name.to_lower_camel_case();
                        let full_key = format!("{}.{}", prefix, field.name);
                        match &field.kind {
                            FieldKind::Plain | FieldKind::Volatile => {
                                root_classes.push_str(&format!(
                                    "        this.{} = new Field<{}>(\"{}\", initialValues?.[\"{}\"]);\n",
                                    prop_name, field.full_ts_type, full_key, full_key
                                ));
                            }
                            FieldKind::Nested { struct_name } => {
                                root_classes.push_str(&format!(
                                    "        this.{} = new {}Fields(\"{}\", initialValues);\n",
                                    prop_name, struct_name, full_key
                                ));
                            }
                            FieldKind::Lookup {
                                target_key,
                                mutable,
                            } => {
                                let class_name = if *mutable { "Field" } else { "ReadonlyField" };
                                root_classes.push_str(&format!(
                                    "        this.{} = new {}<{}>(\"{}\", initialValues?.[\"{}\" as any]);\n",
                                    prop_name, class_name, field.full_ts_type, target_key, target_key
                                ));
                            }
                            FieldKind::LookupNode {
                                target_prefix,
                                struct_name,
                            } => {
                                root_classes.push_str(&format!(
                                    "        this.{} = new {}Fields(\"{}\", initialValues);\n",
                                    prop_name, struct_name, target_prefix
                                ));
                            }
                            FieldKind::ReactiveMap {
                                key_type,
                                value_type,
                                ..
                            } => {
                                root_classes.push_str(&format!(
                                    "        this.{} = new ReactiveMapField<{}, {}>(\"{}\", initialValues);\n",
                                    prop_name, key_type, value_type, full_key
                                ));
                            }
                        }
                    }
                    root_classes.push_str("    }\n\n");

                    root_classes.push_str(&format!(
                        "    static async load(): Promise<{}> {{\n",
                        entry.struct_name
                    ));
                    root_classes.push_str(&format!(
                        "        const initialValues = await invoke<Partial<{}>>(\"plugin:rpstate|rpstate_get_prefix\", {{ prefix: \"{}\" }});\n",
                        schema_name, prefix
                    ));
                    root_classes.push_str(&format!(
                        "        return new {}(initialValues);\n",
                        entry.struct_name
                    ));
                    root_classes.push_str("    }\n\n");

                    root_classes.push_str("    /**\n");
                    root_classes.push_str("     * Flushes all pending changes under this slice's prefix to disk immediately.\n");
                    root_classes.push_str("     * Resolves only when the persistent store has successfully flushed to disk.\n");
                    root_classes.push_str("     */\n");
                    root_classes.push_str("    async save(): Promise<void> {\n");
                    root_classes.push_str(&format!(
                        "        return invoke(\"plugin:rpstate|rpstate_flush\", {{ prefix: \"{}\" }});\n",
                        prefix
                    ));
                    root_classes.push_str("    }\n");

                    root_classes.push_str("}\n\n");
                }
            }
        }

        ts.push_str(&nested_classes);
        ts.push_str(&root_classes);

        if let Some(parent) = out_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_path, ts)?;
        Ok(())
    }

    pub fn export_rust(
        &self,
        out_path: impl AsRef<Path>,
        fw: &dyn FrameworkCodegen,
    ) -> std::io::Result<()> {
        let mut code = String::new();
        code.push_str("// GENERATED AUTOMATICALLY. DO NOT EDIT.\n");
        code.push_str(fw.imports());
        code.push('\n');

        for entry in self.registry.values() {
            for attr in fw.extra_attrs() {
                code.push_str(&format!("{}\n", attr));
            }

            if let Some(prefix) = entry.prefix {
                code.push_str(&format!(
                    "#[::rpstate::rpstate(prefix = \"{}\", target = \"tauri-wasm\")]\n",
                    prefix
                ));
            } else {
                code.push_str("#[::rpstate::rpstate(target = \"tauri-wasm\")]\n");
            }

            for derive in fw.extra_derives() {
                code.push_str(&format!("#[derive({})]\n", derive));
            }

            code.push_str(&format!("pub struct {} {{\n", entry.struct_name));

            for field in entry.fields {
                let mut attributes = Vec::new();
                match &field.kind {
                    FieldKind::Volatile => attributes.push("volatile".to_string()),
                    FieldKind::Nested { .. } => attributes.push("nested".to_string()),
                    FieldKind::Lookup {
                        target_key,
                        mutable,
                    } => {
                        if *mutable {
                            attributes.push(format!("lookup = \"{}\", export_mut", target_key));
                        } else {
                            attributes.push(format!("lookup = \"{}\"", target_key));
                        }
                    }
                    FieldKind::LookupNode { target_prefix, .. } => {
                        attributes.push(format!("lookup_node = \"{}\"", target_prefix));
                    }
                    _ => {}
                }

                if !attributes.is_empty() {
                    code.push_str(&format!("    #[state({})]\n", attributes.join(", ")));
                }

                code.push_str(&format!("    pub {}: {},\n", field.name, field.rust_type));
            }
            code.push_str("}\n\n");
        }

        if let Some(parent) = out_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_path, code)?;
        Ok(())
    }

    fn resolve_fields_ts(
        &self,
        prefix: &str,
        fields: &[FieldExportMeta],
        resolved: &mut Vec<(String, String, Option<String>)>,
    ) {
        for field in fields {
            match &field.kind {
                FieldKind::Plain => {
                    resolved.push((
                        format!("{}.{}", prefix, field.name),
                        field.full_ts_type.to_string(),
                        None,
                    ));
                }
                FieldKind::Volatile => {
                    resolved.push((
                        format!("{}.{}", prefix, field.name),
                        field.full_ts_type.to_string(),
                        Some("volatile".to_string()),
                    ));
                }
                FieldKind::Nested { struct_name } => {
                    if let Some(nested) = self.registry.get(struct_name) {
                        self.resolve_fields_ts(
                            &format!("{}.{}", prefix, field.name),
                            nested.fields,
                            resolved,
                        );
                    }
                }
                FieldKind::Lookup { target_key, .. } => {
                    resolved.push((
                        format!("{}.{}", prefix, field.name),
                        field.full_ts_type.to_string(),
                        Some(format!("@alias {}", target_key)),
                    ));
                }
                FieldKind::LookupNode {
                    target_prefix,
                    struct_name,
                } => {
                    if let Some(nested) = self.registry.get(struct_name) {
                        self.resolve_fields_ts(target_prefix, nested.fields, resolved);
                    }
                }
                FieldKind::ReactiveMap {
                    key_type,
                    value_type,
                    ..
                } => {
                    resolved.push((
                        format!("{}.{}.[key]", prefix, field.name),
                        format!("Record<{}, {}>", key_type, value_type),
                        Some("reactive map".to_string()),
                    ));
                }
            }
        }
    }
}

#[doc = include_str!("../README.md")]
#[macro_export]
macro_rules! rpstate_codegen {
    (rs_out = $rs_out:expr, framework = $fw:ident) => {
        $crate::rpstate_codegen!(@run $rs_out, $fw, None::<&str>)
    };
    (rs_out = $rs_out:expr, framework = $fw:ident, ts_out = $ts_out:expr) => {
        $crate::rpstate_codegen!(@run $rs_out, $fw, Some($ts_out))
    };

    (@run $rs_out:expr, $fw:ident, $ts_opt:expr) => {
        $crate::rpstate_codegen!(@exec $rs_out, $fw, $ts_opt)
    };

    (@exec $rs_out:expr, dioxus, $ts_opt:expr) => {
        {
            let reg = $crate::CodegenRegistry::new();
            reg.export_rust($rs_out, &$crate::TauriDioxusCodegen).expect("Rust codegen failed");
            if let Some(ts) = $ts_opt {
                reg.export_ts(ts).expect("TS codegen failed");
            }
        }
    };

    (@exec $rs_out:expr, leptos, $ts_opt:expr) => {
        {
            let reg = $crate::CodegenRegistry::new();
            reg.export_rust($rs_out, &$crate::TauriLeptosCodegen).expect("Rust codegen failed");
            if let Some(ts) = $ts_opt {
                reg.export_ts(ts).expect("TS codegen failed");
            }
        }
    };

    (@exec $rs_out:expr, vanilla, $ts_opt:expr) => {
        {
            let reg = $crate::CodegenRegistry::new();
            reg.export_rust($rs_out, &$crate::TauriVanillaCodegen).expect("Rust codegen failed");
            if let Some(ts) = $ts_opt {
                reg.export_ts(ts).expect("TS codegen failed");
            }
        }
    };
}

#[doc = include_str!("../README.md")]
#[macro_export]
macro_rules! rpstate_codegen_main {
    (rs_out = $rs_out:expr, framework = $fw:ident) => {
        fn main() {
            $crate::rpstate_codegen!(rs_out = $rs_out, framework = $fw);
        }
    };
    (rs_out = $rs_out:expr, framework = $fw:ident, ts_out = $ts_out:expr) => {
        fn main() {
            $crate::rpstate_codegen!(rs_out = $rs_out, framework = $fw, ts_out = $ts_out);
        }
    };
}
