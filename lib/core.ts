let lib: any;
try {
    lib = require(`./webarea_${process.platform}_${process.arch}.node`);
} catch (e: any) {
    throw new Error(
        "This version of webarea is not compatible with your Node.js build:\n\n" +
            e.toString()
    );
}

export default lib;
