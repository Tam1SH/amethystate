/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
// src/bindings/rpstate.ts DO NOT EDIT
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
     * Synchronous optimistic getter.
     *
     * @returns The optimistically updated local value.
     * @tradeoff Resolved in-memory. Might not reflect the actual persistent store
     * if background sync is pending or failed. Use `get()` for transaction-safe checks.
     */
    get value(): T | null {
        return this._value;
    }

    /**
     * Synchronous optimistic setter.
     *
     * @param newValue The new value to assign.
     * @tradeoff Immediately updates the local cache to keep the UI lag-free (optimistic update),
     * while firing an asynchronous write in the background. Note that writes are debounced/buffered;
     * call and await the `save()` method on the parent slice class to guarantee immediate disk persistence.
     */
    set value(newValue: T) {
        this._value = newValue;
        this.set(newValue).catch((err) => {
            console.error(`Optimistic update failed for key ${this.key}:`, err);
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
     * Synchronous optimistic getter.
     *
     * @returns The optimistically updated local value.
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
     * Synchronous optimistic getter.
     *
     * @param key The map key to look up.
     * @returns The locally cached value.
     * @tradeoff Resolved in-memory. Might lag behind actual persistent store transactions. Use `get(key)` for absolute checks.
     */
    getSync(key: K): V | null {
        return this._map.get(key) ?? null;
    }

    /**
     * Synchronous optimistic setter.
     *
     * @param key The map key to assign.
     * @param value The value to write.
     * @tradeoff Instantly updates the local memory map (optimistic update) while initiating background write.
     * Writes are debounced/buffered; call and await `save()` on the parent slice class to flush changes to disk.
     */
    setSync(key: K, value: V): void {
        this._map.set(key, value);
        this.set(key, value).catch((err) => {
            console.error(`Optimistic map update failed for ${this.prefix}.${key}:`, err);
        });
    }

    /**
     * Synchronous optimistic key lookup.
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

export type StateSchema = {
    "settings.username": string;
    "settings.counter": number;
    "settings.theme": string;
};

export type AppSettingsSchema = {
    "settings.username": string;
    "settings.counter": number;
    "settings.theme": string;
};

export class AppSettings {
    readonly username: Field<string>;
    readonly counter: Field<number>;
    readonly theme: Field<string>;
    constructor(initialValues?: Partial<AppSettingsSchema>) {
        this.username = new Field<string>("settings.username", initialValues?.["settings.username"]);
        this.counter = new Field<number>("settings.counter", initialValues?.["settings.counter"]);
        this.theme = new Field<string>("settings.theme", initialValues?.["settings.theme"]);
    }

    static async load(): Promise<AppSettings> {
        const initialValues = await invoke<Partial<AppSettingsSchema>>("plugin:rpstate|rpstate_get_prefix", { prefix: "settings" });
        return new AppSettings(initialValues);
    }

    /**
     * Flushes all pending changes under this slice's prefix to disk immediately.
     * Resolves only when the persistent store has successfully flushed to disk.
     */
    async save(): Promise<void> {
        return invoke("plugin:rpstate|rpstate_flush", { prefix: "settings" });
    }
}

