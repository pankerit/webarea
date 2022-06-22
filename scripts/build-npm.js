const fs = require("fs-extra");
const { exec } = require("child_process");
const pkg = require("../package.json");

try {
    fs.rmSync("./dist", { recursive: true });
} catch (e) {
    console.log(e);
}
fs.ensureDir("./dist");

exec(`yarn tsc`, (err, stdout, stderr) => {
    if (err) {
        console.error(err);
        return;
    }
    console.log(stdout);
    console.log(stderr);
    console.log(`tsc build done`);
    exec(`yarn rollup -c`, (err, stdout, stderr) => {
        if (err) {
            console.error(err);
            return;
        }
        console.log(stdout);
        console.log(stderr);
        console.log(`rollup build done`);
        fs.rmSync("./dist/dts", { recursive: true });
    });
});

fs.copyFileSync("./scripts/downloader.js", "./dist/downloader.js");
fs.copyFileSync("./README.md", "./dist/README.md");

delete pkg.devDependencies;
delete pkg.scripts;

pkg.scripts = {
    preinstall: "node ./downloader.js",
};

fs.writeFileSync("./dist/package.json", JSON.stringify(pkg, null, 4));
