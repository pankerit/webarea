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
    exec(
        `yarn cross-env NODE_ENV=development rollup -c`,
        (err, stdout, stderr) => {
            if (err) {
                console.error(err);
                return;
            }
            console.log(stdout);
            console.log(stderr);
            console.log(`rollup build done`);
            fs.rmSync("./dist/dts", { recursive: true });
        }
    );
});

exec(`yarn build:binaries:dev`, (err, stdout, stderr) => {
    if (err) {
        console.error(err);
        return;
    }
    console.log(stdout);
    console.log(stderr);
    console.log(`binaries build done`);
});
