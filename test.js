// const lib = require("./index.node");
// const jimp = require("jimp");

// console.log(lib);

// async function main() {
//     const cb = (payload) => {
//         console.log(payload);
//     };
//     let title = "AG";
//     let devtools = false;
//     let transparent = true;
//     let frameless = false;
//     let width = 800;
//     let height = 600;

//     const box = await lib.create(
//         cb,
//         title,
//         devtools,
//         transparent,
//         frameless,
//         width,
//         height
//     );
//     if (devtools) {
//         lib.open_devtools(box);
//     }
//     //   setTimeout(async () => {
//     //     const img = await jimp.read("./ag-logo.png");
//     //     lib.change_icon(box, img.bitmap.data, img.bitmap.width, img.bitmap.height);
//     //   }, 1000);

//     //   setTimeout(() => {
//     //     console.log("closing");
//     //     lib.close(box);
//     //   }, 2000);
// }

// main();
const { Webview } = require("./dist/webview");

async function main() {
    const web = new Webview({
        visible: false,
    });
    await web.center();
    await web.setVisible(true);
    // await web.setVisible(true);
    // web.close();
    // web.setTitle("test");
    // setInterval(() => {});
}

main();
