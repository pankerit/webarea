module.exports = (() => {
    try {
        let lib = require(`./webarea_${process.arch}.node`);
        return lib;
    } catch (e) {
        throw new Error(
            "This version of mem-tool is not compatible with your Node.js build:\n\n" +
                e.toString()
        );
    }
})();
