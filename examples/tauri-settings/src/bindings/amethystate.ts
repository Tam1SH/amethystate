/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
// src/bindings/amethystate.ts DO NOT EDIT
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import { ReactiveField, ReadonlyReactiveField, ReactiveMap } from "amethystate";

export type StateSchema = {
    "settings.username": string;
    "settings.counter": number;
    "settings.theme.mode": string;
    "settings.theme.background": string;
    "settings.theme.foreground": string;
    "settings.proxy": ProxyProfile;
    /** reactive map */
    "settings.env.[key]": Record<string, string>;
};

export type ProxyProfile = {
    name: string;
    address: string;
    port: number;
    enabled: boolean;
};

export class ProxyProfileFields {
    readonly name: ReactiveField<string>;
    readonly address: ReactiveField<string>;
    readonly port: ReactiveField<number>;
    readonly enabled: ReactiveField<boolean>;
    constructor(prefix: string, initialValues?: Record<string, any>) {
        this.name = new ReactiveField(`${prefix}.name`, initialValues?.[`${prefix}.name`]);
        this.address = new ReactiveField(`${prefix}.address`, initialValues?.[`${prefix}.address`]);
        this.port = new ReactiveField(`${prefix}.port`, initialValues?.[`${prefix}.port`]);
        this.enabled = new ReactiveField(`${prefix}.enabled`, initialValues?.[`${prefix}.enabled`]);
    }
}

export type Theme = {
    mode: string;
    background: string;
    foreground: string;
};

export class ThemeFields {
    readonly mode: ReactiveField<string>;
    readonly background: ReactiveField<string>;
    readonly foreground: ReactiveField<string>;
    constructor(prefix: string, initialValues?: Record<string, any>) {
        this.mode = new ReactiveField(`${prefix}.mode`, initialValues?.[`${prefix}.mode`]);
        this.background = new ReactiveField(`${prefix}.background`, initialValues?.[`${prefix}.background`]);
        this.foreground = new ReactiveField(`${prefix}.foreground`, initialValues?.[`${prefix}.foreground`]);
    }
}

export type AppSettingsPlain = {
    username: string;
    counter: number;
    theme: Theme;
    proxy: ProxyProfile;
    env: Record<string, string>;
};

export type AppSettingsSchema = {
    "settings.username": string;
    "settings.counter": number;
    "settings.theme.mode": string;
    "settings.theme.background": string;
    "settings.theme.foreground": string;
    "settings.proxy": ProxyProfile;
    /** reactive map */
    "settings.env.[key]": Record<string, string>;
};

export class AppSettings {
    readonly username: ReactiveField<string>;
    readonly counter: ReactiveField<number>;
    readonly theme: ThemeFields;
    readonly proxy: ReactiveField<ProxyProfile>;
    readonly env: ReactiveMap<string, string>;
    constructor(initialValues?: Partial<AppSettingsSchema>) {
        this.username = new ReactiveField<string>("settings.username", initialValues?.["settings.username"]);
        this.counter = new ReactiveField<number>("settings.counter", initialValues?.["settings.counter"]);
        this.theme = new ThemeFields("settings.theme", initialValues);
        this.proxy = new ReactiveField<ProxyProfile>("settings.proxy", initialValues?.["settings.proxy"]);
        this.env = new ReactiveMap<string, string>("settings.env", initialValues);
    }

    static async load(): Promise<AppSettings> {
        const initialValues = await invoke<Partial<AppSettingsSchema>>("plugin:amethystate|amethystate_get_prefix", { prefix: "settings" });
        return new AppSettings(initialValues);
    }

    /**
     * Flushes all pending changes under this slice's prefix to disk immediately.
     * Resolves only when the persistent store has successfully flushed to disk.
     */
    async save(): Promise<void> {
        return invoke("plugin:amethystate|amethystate_flush", { prefix: "settings" });
    }
}

