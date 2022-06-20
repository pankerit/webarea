import { Webview } from "./webview";
import * as _app from "./app";

export { Webview };

export const app = {
    quit: _app.quit,
    unsafe_quit: _app.unsafe_quit,
};

async function test() {
    console.log("app.init");

    for (let i = 0; i < 10; i++) {
        const window = new Webview({ devtools: true });
        await window.waitUntilReady();
        // await window.focus();
        // await window.setFrameless(true);
    }
    await app.quit();
    console.log("test");
}

// test();
