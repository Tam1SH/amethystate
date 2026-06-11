import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";

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

    subscribe(cb: (value: T) => void): () => (Promise<void> | void) {
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
                return invoke("plugin:rpstate|rpstate_unsubscribe", { key: this.key })
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