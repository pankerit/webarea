import { EventEmitter } from "events";
import type { Size } from "./types";
const lib = require("./core");
const preload = require("./preload");

interface Bitmap {
    width: number;
    height: number;
    data: Buffer;
}

interface Context {
    preload?: string;
}

interface WebviewOptions {
    title?: string;
    devtools?: boolean;
    transparent?: boolean;
    frameless?: boolean;
    visible?: boolean;
    resizable?: boolean;
    innerSize?: Size;
    context?: Context;
}

const defaultPayload: Required<WebviewOptions> = {
    title: "My app",
    devtools: true,
    transparent: false,
    frameless: false,
    visible: true,
    resizable: true,
    innerSize: {
        width: 800,
        height: 600,
    },
    context: {
        preload,
    },
};

class EE extends EventEmitter {
    constructor(private webview: Webview) {
        super();
    }

    async send(channel: string, message: any) {
        this.webview.waitUntilReady();
        this.webview.evaluateScript(
            `__NODE__.__emit('${channel}', '${JSON.stringify(message)}')`
        );
    }
}

export class Webview {
    private ready = false;
    private waits: (() => void)[] = [];
    private internalEvents = new EventEmitter();
    ipc = new EE(this);
    #box: any;

    constructor(private option?: WebviewOptions) {
        if (option?.context?.preload) {
            defaultPayload.context.preload += option.context.preload;
        }
        const payload = Object.assign(defaultPayload, option);
        (async () => {
            this.#box = await lib.create(
                this.listener,
                payload.title,
                payload.devtools,
                payload.transparent,
                payload.frameless,
                payload.innerSize.width,
                payload.innerSize.height,
                payload.visible,
                payload.resizable,
                payload.context.preload
            );
            this.ready = true;
            this.waits.forEach((wait) => {
                wait();
            });
        })();
    }

    private listener = (type: string, data: any) => {
        console.log(type, data);
        switch (type) {
            case "getInnerSize": {
                const { width, height } = data;
                this.internalEvents.emit("getInnerSize", { width, height });
                break;
            }
            case "ipc": {
                const { channel, payload } = JSON.parse(data);
                this.ipc.emit(channel, payload);
                break;
            }
        }
    };

    async close(): Promise<void> {
        await this.waitUntilReady();
        this.ipc.removeAllListeners();
        return new Promise((resolve) => {
            lib.close(this.#box, resolve);
        });
    }

    async focus(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_focus(this.#box, res);
        });
    }

    async center(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_center(this.#box, res);
        });
    }

    async minimized(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_minimized(this.#box, true, res);
        });
    }

    async maximized(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_minimized(this.#box, false, res);
        });
    }

    async setTitle(title: string): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_title(this.#box, title, res);
        });
    }

    async setVisible(visible: boolean): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_visible(this.#box, visible, res);
        });
    }

    async setResizable(resizable: boolean): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_resizable(this.#box, resizable, res);
        });
    }

    async openDevtools(): Promise<void> {
        if (this.option?.devtools) {
            await this.waitUntilReady();
            return new Promise((res) => {
                lib.open_devtools(this.#box, res);
            });
        } else {
            console.warn("Devtools are disabled");
        }
    }

    async closeDevtools(): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.close_devtools(this.#box, res);
        });
    }

    async setInnerSize(width: number, height: number): Promise<void> {
        await this.waitUntilReady();
        return new Promise((resolve) => {
            lib.set_inner_size(this.#box, width, height, resolve);
        });
    }

    async getInnerSize(): Promise<Size> {
        await this.waitUntilReady();
        return new Promise((resolve) => {
            lib.get_inner_size(this.#box, resolve);
        });
    }

    // getOuterSize() {}

    // setMinInnerSize(width: number, height: number) {}

    // setMaxInnerSize(width: number, height: number) {}

    // setWindowFrameless(frameless: boolean) {}
    async setIgnoreCursorEvents(ignoreCursorEvents: boolean): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_ignore_cursor_events(this.#box, ignoreCursorEvents, res);
        });
    }

    async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_always_on_top(this.#box, alwaysOnTop, res);
        });
    }

    async loadURL(url: string) {
        await this.waitUntilReady();
        await this.evaluateScript(`location.replace("${url}");`);
    }

    async loadHTML(html: string) {
        await this.waitUntilReady();
        await this.evaluateScript(
            `document.documentElement.innerHTML = \`${html}\``
        );
    }

    async evaluateScript(script: string): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.evaluate_script(this.#box, script, res);
        });
    }

    async setIcon(bitmap: Bitmap): Promise<void> {
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_icon(
                this.#box,
                bitmap.data,
                bitmap.width,
                bitmap.height,
                res
            );
        });
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

// async function test() {
//     const webview = new Webview({
//         // frameless: true,
//         transparent: true,
//         resizable: true,
//         devtools: true,
//     });
//     webview.setTitle("Hello world");
//     webview.setVisible(true);
//     webview.focus();
//     webview.openDevtools();
//     webview.loadURL("https://www.google.com");
//     let a = 0;
//     setInterval(() => {
//         webview.setInnerSize(a, a);
//         a += 10;
//     }, 1000);
//     webview.setResizable(true);
// }

// test();
