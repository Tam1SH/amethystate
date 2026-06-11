import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";

export type MapChange<K, V> =
    | { type: "Insert"; key: K; value: V }
    | { type: "Update"; key: K; oldValue: V; newValue: V }
    | { type: "Remove"; key: K; oldValue: V }
    | { type: "Clear" };

export class ReactiveMap<K extends string, V> {
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