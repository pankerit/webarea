import { EventEmitter } from "events";
import type { Bitmap, WebviewOptions } from "./types";
const lib = require("./core");
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
            const listener = (event: string, ...args: any[]) => {
                switch (event) {
                    case "ipc": {
                        const [windowId, message] = args;
                        const { channel, payload } = JSON.parse(message);
                        const browserWindow = BrowserWindow.all.find((b) =>
                            lib.compare_window_id(b.boxedWindowId, windowId)
                        )!;
                        browserWindow.ipc.emit(channel, payload);
                        break;
                    }
                    case "close-window": {
                        const [windowId] = args;
                        const browserWindow = BrowserWindow.all.find((b) =>
                            lib.compare_window_id(b.boxedWindowId, windowId)
                        )!;
                        browserWindow.close();
                        break;
                    }
                    default: {
                        console.log(event, args);
                    }
                }
            };
            lib.app_init(listener, (boxedIpc: any) => {
                _boxedIpc = boxedIpc;
                res();
            });
        }),
    quit: () => {
        console.log(BrowserWindow.all);
        for (let browser of BrowserWindow.all) {
            if (!browser.ready) {
                throw new Error(
                    `can't close browser while it's in process of initialization`
                );
            }
        }
        return Promise.all(BrowserWindow.all.map((b) => b.close()));
    },
    unsafe_quit: async () => {
        await Promise.all(BrowserWindow.all.map((b) => b.close()));
        return new Promise((res) => {
            lib.unsafe_quit(getBoxedIpc(), res);
        });
    },
};

class Ipc extends EventEmitter {
    constructor(private browserWindow: BrowserWindow) {
        super();
    }

    async send(channel: string, message: any) {
        this.browserWindow.waitUntilReady();
        this.browserWindow.evaluateScript(
            `__NODE__.__emit('${channel}', '${JSON.stringify(message)}')`
        );
    }
}

export class BrowserWindow {
    static all: BrowserWindow[] = [];
    ready = false;
    private waits: (() => void)[] = [];
    private closed = false;
    ipc = new Ipc(this);
    boxedWindowId: any;
    #boxedIpc: any;

    constructor(private options: WebviewOptions = {}) {
        BrowserWindow.all.push(this);
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
            defaultPayload.preloadScript + (options.preloadScript || ""),
            (boxedWindowId: any) => {
                this.boxedWindowId = boxedWindowId;
                this.ready = true;
                this.waits.forEach((wait) => wait());
            }
        );
    }

    async loadURL(url: string) {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.evaluateScript(`location.replace("${url}");`);
    }

    async loadHTML(html: string) {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.evaluateScript(
            `document.documentElement.innerHTML = \`${html}\``
        );
    }

    async close(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        this.closed = true;
        this.ipc.removeAllListeners();
        const index = BrowserWindow.all.indexOf(this);
        BrowserWindow.all.splice(index, 1);
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.close_window(this.#boxedIpc, this.boxedWindowId, res);
        });
    }

    async focus(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.focus_window(this.#boxedIpc, this.boxedWindowId, res);
        });
    }

    async center(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.center_window(this.#boxedIpc, this.boxedWindowId, res);
        });
    }

    async show(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_visible_window(
                this.#boxedIpc,
                this.boxedWindowId,
                true,
                res
            );
        });
    }

    async hide(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_visible_window(
                this.#boxedIpc,
                this.boxedWindowId,
                false,
                res
            );
        });
    }

    async minimize(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_minimized_window(
                this.#boxedIpc,
                this.boxedWindowId,
                true,
                res
            );
        });
    }

    async maximize(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_minimized_window(
                this.#boxedIpc,
                this.boxedWindowId,
                false,
                res
            );
        });
    }

    async setTitle(title: string): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_title_window(
                this.#boxedIpc,
                this.boxedWindowId,
                title,
                res
            );
        });
    }

    async setResizable(resizable: boolean): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_resizable_window(
                this.#boxedIpc,
                this.boxedWindowId,
                resizable,
                res
            );
        });
    }

    async evaluateScript(script: string): Promise<any> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.evaluate_script(
                this.#boxedIpc,
                this.boxedWindowId,
                script,
                res
            );
        });
    }

    async setSize(width: number, height: number): Promise<any> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_window_size(
                this.#boxedIpc,
                this.boxedWindowId,
                width,
                height,
                res
            );
        });
    }

    async getSize(): Promise<[number, number]> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.get_window_size(
                this.#boxedIpc,
                this.boxedWindowId,
                (width: number, height: number) => {
                    res([width, height]);
                }
            );
        });
    }

    async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_always_on_top(
                this.#boxedIpc,
                this.boxedWindowId,
                alwaysOnTop,
                res
            );
        });
    }

    async setIgnoreCursorEvents(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_ignore_cursor_events(
                this.#boxedIpc,
                this.boxedWindowId,
                true,
                res
            );
        });
    }

    async openDevtools(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        if (this.options.devtools) {
            await this.waitUntilReady();
            return new Promise((res) => {
                lib.open_devtools(this.#boxedIpc, this.boxedWindowId, res);
            });
        } else {
            console.warn("Devtools are disabled");
        }
    }

    async closeDevtools(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.close_devtools(this.#boxedIpc, this.boxedWindowId, res);
        });
    }

    async setFrameless(frameless: boolean): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_frameless_window(
                this.#boxedIpc,
                this.boxedWindowId,
                frameless,
                res
            );
        });
    }

    async setIcon(bitmap: Bitmap): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_window_icon(
                this.#boxedIpc,
                this.boxedWindowId,
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

    waitUntilReady(): Promise<void> {
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

    for (let i = 0; i < 10; i++) {
        const window = new BrowserWindow({ devtools: true });
        await window.openDevtools();
        // await window.focus();
        // await window.setFrameless(true);
    }
    await app.quit();

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

// test();
