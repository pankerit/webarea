import { Webview } from "./webview";
import lib from "./core";
import { EventEmitter } from "events";

export const events = new EventEmitter();

let _boxedIpc: any = null;
let _waits: (() => void)[] = [];
let _ready = false;
let _start = false;

const listener = (event: string, ...args: any[]) => {
    switch (event) {
        case "error": {
            const message = args[0];
            events.emit("error", new Error(message));
            break;
        }
        case "ipc": {
            const [windowId, message] = args;
            const { channel, payload } = JSON.parse(message);
            const browserWindow = Webview.all.find((b) =>
                lib.compare_window_id(b.boxedWindowId, windowId)
            )!;
            browserWindow.ipc.emit(channel, payload);
            break;
        }
        case "close-window": {
            const [windowId] = args;
            const browserWindow = Webview.all.find((b) =>
                lib.compare_window_id(b.boxedWindowId, windowId)
            )!;

            browserWindow.close().then(() => {
                browserWindow.emit("close");
            });
            break;
        }
        case "resize-window": {
            const [windowId, width, height] = args;
            const browserWindow = Webview.all.find((b) =>
                lib.compare_window_id(b.boxedWindowId, windowId)
            )!;
            browserWindow.emit("resize", width, height);
            break;
        }
        case "move-window": {
            const [windowId, x, y] = args;
            const browserWindow = Webview.all.find((b) =>
                lib.compare_window_id(b.boxedWindowId, windowId)
            )!;
            browserWindow.emit("move", x, y);
            break;
        }
        default: {
            console.log(event, args);
        }
    }
};

export const _init = () => {
    _start = true;
    return new Promise<void>((res) => {
        lib.app_init(listener, (boxedIpc: any) => {
            _boxedIpc = boxedIpc;
            _ready = true;
            res();
            _waits.forEach((resolve) => resolve());
        });
    });
};

export const quit = async () => {
    const webviews = [...Webview.all];
    for (let browser of webviews) {
        if (!browser.ready) {
            throw new Error(
                `can't close browser while it's in process of initialization`
            );
        }
    }

    return Promise.all(webviews.map((b) => b.close()));
};

export const unsafe_quit = async () => {
    await quit();
    return new Promise((res) => {
        lib.unsafe_quit(getBoxedIpc(), res);
    });
};

export const getBoxedIpc = () => {
    if (!_boxedIpc) {
        throw new Error("app must be initialized before use");
    }
    return _boxedIpc;
};

export const _isStarted = () => {
    return _start;
};

export const _waitUntilReady = () => {
    if (_ready) {
        return Promise.resolve();
    }
    return new Promise<void>((resolve) => {
        _waits.push(resolve);
    });
};
