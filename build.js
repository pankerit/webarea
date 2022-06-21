const fs = require("fs-extra");
const path = require("path");
const pkg = require("./package.json");
const { exec } = require("child_process");

try {
    fs.rmSync("./dist", { recursive: true });
} catch (e) {}

const platforms = [
    {
        name: "win32",
        arch: {
            x64: "x86_64-pc-windows-msvc",
            ia32: "i686-pc-windows-msvc",
        },
    },
];
const buildBinaries = async () => {
    const binaries = [];
    for (let platform of platforms) {
        for (let arch in platform.arch) {
            const rustTarget = platform.arch[arch];
            const target = `${rustTarget}-${platform.name}-${arch}`;
            binaries.push(
                new Promise((res) => {
                    const cmd = `cargo build --target ${rustTarget} --release --verbose`;
                    console.log(`Building ${target}...`);
                    exec(cmd, (err, stdout, stderr) => {
                        if (err) {
                            console.error(err);
                            return;
                        }
                        // console.log(stdout);
                        // console.log(stderr);
                        console.log(`Building ${target}... done`);
                        res();
                    });
                }).then(() => {
                    const input = `./target/${rustTarget}/release/webarea.dll`;
                    const output = `./binaries/webarea_${platform.name}_${arch}.node`;
                    const binariesData = fs.readFileSync(input);
                    fs.ensureFileSync(output);
                    fs.writeFileSync(output, binariesData);
                })
            );
        }
    }
    await Promise.all(binaries);
};

buildBinaries().then(() => {
    console.log("Build done, ready for publishing");
});

// execSync("yarn build-release:ia32");
// execSync("yarn build-release:x64");
// execSync("yarn tsc");

// fs.copyFileSync("./README.md", "./dist/README.md");
// fs.copyFileSync("./webarea_ia32.node", "./dist/webarea_ia32.node");
// fs.copyFileSync("./webarea_x64.node", "./dist/webarea_x64.node");
// fs.copyFileSync("./lib/core.js", "./dist/core.js");
// fs.copyFileSync("./lib/preload.js", "./dist/preload.js");

// delete pkg.devDependencies;
// delete pkg.scripts;

// fs.writeFileSync("./dist/package.json", JSON.stringify(pkg, null, 4));
