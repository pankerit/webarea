function preload() {
    // initialization
    let __NODE__ = {};
    __NODE__.dragWindow = () => {
        ipc.postMessage("drag-window");
    };
    __NODE__.send = (channel, payload) => {
        ipc.postMessage(`ipc:${JSON.stringify({ channel, payload })}`);
    };
    const callback = [];
    __NODE__.on = (channel, message) => {
        callback.push([channel, message]);
    };
    __NODE__.once = (channel, message) => {
        callback.push([channel, message, true]);
    };
    __NODE__.__emit = (channel, payload) => {
        callback.forEach((item) => {
            if (item[0] === channel) {
                item[1](JSON.parse(payload));
                if (item[2]) {
                    callback.splice(callback.indexOf(item), 1);
                }
            }
        });
    };
    window.__NODE__ = __NODE__;

    //event initialization
    // drag window
    window.addEventListener("mousedown", (e) => {
        if (e.target.className.includes("drag-window")) {
            ipc.postMessage("drag-window");
        }
    });
}

module.exports = preload.toString() + ";preload();";
