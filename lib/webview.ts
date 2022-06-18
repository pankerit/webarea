import { EventEmitter } from "events";
import type { Bitmap, WebviewOptions } from "./types";
const lib = require("../index.node");
const preload = require("./preload");

let _boxedIpc: any = null;
function getBoxedIpc() {
    if (!_boxedIpc) {
        throw new Error("app must be initialized before use");
    }
    return _boxedIpc;
}

export const app = {
    init: () =>
        new Promise<void>((res) => {
            lib.app_init(console.log, (boxedIpc: any) => {
                _boxedIpc = boxedIpc;
                res();
            });
        }),
};

export class BrowserWindow {
    private ready = false;
    private waits: (() => void)[] = [];
    #boxedWindowId: any;
    #boxedIpc: any;

    constructor(private options: WebviewOptions = {}) {
        const defaultPayload = this.defaultOptions();
        const payload = { ...defaultPayload, ...options };
        this.#boxedIpc = getBoxedIpc();

        lib.create_new_window(
            this.#boxedIpc,
            payload.title,
            payload.devtools,
            payload.transparent,
            payload.frameless,
            payload.width,
            payload.height,
            payload.visible,
            payload.resizable,
            defaultPayload + (options.preloadScript || ""),
            (boxedWindowId: any) => {
                this.#boxedWindowId = boxedWindowId;
                this.ready = true;
                this.waits.forEach((wait) => wait());
            }
        );
    }

    async close(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.close_window(this.#boxedIpc, this.#boxedWindowId, res);
        });
    }

    async focus(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.focus_window(this.#boxedIpc, this.#boxedWindowId, res);
        });
    }

    async center(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.center_window(this.#boxedIpc, this.#boxedWindowId, res);
        });
    }

    async show(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_visible_window(
                this.#boxedIpc,
                this.#boxedWindowId,
                true,
                res
            );
        });
    }

    async hide(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_visible_window(
                this.#boxedIpc,
                this.#boxedWindowId,
                false,
                res
            );
        });
    }

    async minimize(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_minimized_window(
                this.#boxedIpc,
                this.#boxedWindowId,
                true,
                res
            );
        });
    }

    async maximize(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_minimized_window(
                this.#boxedIpc,
                this.#boxedWindowId,
                false,
                res
            );
        });
    }

    async setTitle(title: string): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_title_window(
                this.#boxedIpc,
                this.#boxedWindowId,
                title,
                res
            );
        });
    }

    async setResizable(resizable: boolean): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_resizable_window(
                this.#boxedIpc,
                this.#boxedWindowId,
                resizable,
                res
            );
        });
    }

    async evaluate_script(script: string): Promise<any> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.evaluate_script(
                this.#boxedIpc,
                this.#boxedWindowId,
                script,
                res
            );
        });
    }

    async setSize(width: number, height: number): Promise<any> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_window_size(
                this.#boxedIpc,
                this.#boxedWindowId,
                width,
                height,
                res
            );
        });
    }

    async getSize(): Promise<[number, number]> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.get_window_size(
                this.#boxedIpc,
                this.#boxedWindowId,
                (width: number, height: number) => {
                    res([width, height]);
                }
            );
        });
    }

    async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_always_on_top(
                this.#boxedIpc,
                this.#boxedWindowId,
                alwaysOnTop,
                res
            );
        });
    }

    async setIgnoreCursorEvents(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_ignore_cursor_events(
                this.#boxedIpc,
                this.#boxedWindowId,
                true,
                res
            );
        });
    }

    async openDevtools(): Promise<void> {
        if (this.options.devtools) {
            await this.waitUntilReady();
            return new Promise((res) => {
                lib.open_devtools(this.#boxedIpc, this.#boxedWindowId, res);
            });
        } else {
            console.warn("Devtools are disabled");
        }
    }

    async closeDevtools(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.close_devtools(this.#boxedIpc, this.#boxedWindowId, res);
        });
    }

    async setFrameless(frameless: boolean): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_frameless_window(
                this.#boxedIpc,
                this.#boxedWindowId,
                frameless,
                res
            );
        });
    }

    async setIcon(bitmap: Bitmap): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_window_icon(
                this.#boxedIpc,
                this.#boxedWindowId,
                bitmap.data,
                bitmap.height,
                bitmap.width,
                res
            );
        });
    }

    private defaultOptions() {
        const defaultPayload: Required<WebviewOptions> = {
            title: "My app",
            devtools: true,
            transparent: false,
            frameless: false,
            visible: true,
            resizable: true,
            width: 800,
            height: 600,
            preloadScript: preload,
        };
        return defaultPayload;
    }

    private waitUntilReady(): Promise<void> {
        if (this.ready) {
            return Promise.resolve();
        }
        return new Promise((resolve) => {
            this.waits.push(resolve);
        });
    }
}

async function test() {
    await app.init();
    console.log("app.init");

    const window = new BrowserWindow();
    await window.center();
    await window.setFrameless(true);

    // setInterval(() => {});
    // const webview = new Webview({
    //     // frameless: true,
    //     transparent: true,
    //     resizable: true,
    //     devtools: true,
    // });
    // webview.setTitle("Hello world");
    // webview.setVisible(true);
    // webview.focus();
    // webview.openDevtools();
    // webview.loadURL("https://www.google.com");
    // let a = 0;
    // setInterval(() => {
    //     webview.setInnerSize(a, a);
    //     a += 10;
    // }, 1000);
    // webview.setResizable(true);
}

test();
