const { linkSync } = require("fs-extra");
const lib = require("./index.node");

console.log(lib);

async function main() {
    let title = "AG";
    let devtools = true;
    let transparent = true;
    let frameless = false;
    let width = 800;
    let height = 600;
    let visible = true;
    let resizable = true;
    let initialization_script = "console.log('test')";

    const listener = console.log;
    lib.app_init(listener, (ipcBoxed) => {
        console.log(ipcBoxed);
        for (let i = 0; i < 1; i++) {
            lib.create_new_window(
                ipcBoxed,
                title,
                devtools,
                transparent,
                frameless,
                width,
                height,
                visible,
                resizable,
                initialization_script,
                (windowIdBoxed) => {
                    console.log("created");
                    console.log(windowIdBoxed);
                    console.log("created");
                    lib.center_window(ipcBoxed, windowIdBoxed, () => {
                        console.log("centered");
                    });
                    setTimeout(() => {
                        lib.close_window(ipcBoxed, windowIdBoxed, () => {
                            console.log("closed");
                        });
                    }, 1000);
                }
            );
        }
    });
}

main();
// const { Webview } = require("./dist/webview");

// async function main() {
// await web.close();
// await web.focus();
// await web.center();
// setTimeout(async () => {
//     await web.setFrameless(false);
// }, 2000);
// setTimeout(async () => {
//     await web.setFrameless(true);
// }, 4000);
// setInterval(async () => {
//     const size = await web.getInnerSize();
//     await web.setInnerSize(size.width + 1, size.height + 1);
//     await web.center();
// }, 1000);
// setInterval(() => {
//     console.log("test2");
// });
// }

// main();
