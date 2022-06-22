import { EventEmitter } from "events";
import * as app from "./app";
import { preload } from "./preload";
import type { Bitmap, WebviewOptions } from "./types";
import lib from "./core";

class Ipc extends EventEmitter {
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
    static all: Webview[] = [];
    ready = false;
    private waits: (() => void)[] = [];
    private closed = false;
    ipc = new Ipc(this);
    boxedWindowId: any;

    constructor(private options: WebviewOptions = {}) {
        Webview.all.push(this);
        const defaultPayload = this.defaultOptions();
        const payload = { ...defaultPayload, ...options };

        // init app
        const init = async () => {
            if (app._isStarted()) {
                await app._waitUntilReady();
            } else {
                await app._init();
            }
            lib.create_new_window(
                app.getBoxedIpc(),
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
        };

        init();
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
        const index = Webview.all.indexOf(this);
        Webview.all.splice(index, 1);
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.close_window(app.getBoxedIpc(), this.boxedWindowId, res);
        });
    }

    async focus(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.focus_window(app.getBoxedIpc(), this.boxedWindowId, res);
        });
    }

    async center(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.center_window(app.getBoxedIpc(), this.boxedWindowId, res);
        });
    }

    async show(): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_visible_window(
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
                lib.open_devtools(app.getBoxedIpc(), this.boxedWindowId, res);
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
            lib.close_devtools(app.getBoxedIpc(), this.boxedWindowId, res);
        });
    }

    async setFrameless(frameless: boolean): Promise<void> {
        if (this.closed) {
            throw new Error("window is closed");
        }
        await this.waitUntilReady();
        return new Promise((res) => {
            lib.set_frameless_window(
                app.getBoxedIpc(),
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
                app.getBoxedIpc(),
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
