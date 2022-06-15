const fs = require("fs-extra");
const path = require("path");
const pkg = require("./package.json");
const { execSync } = require("child_process");

try {
    fs.rmSync("./dist", { recursive: true });
} catch (e) {}

execSync("yarn build-release:ia32");
execSync("yarn build-release:x64");
execSync("yarn tsc");

fs.copyFileSync("./README.md", "./dist/README.md");
fs.copyFileSync("./webarea_ia32.node", "./dist/webarea_ia32.node");
fs.copyFileSync("./webarea_x64.node", "./dist/webarea_x64.node");
fs.copyFileSync("./lib/core.js", "./dist/core.js");
fs.copyFileSync("./lib/preload.js", "./dist/preload.js");

delete pkg.devDependencies;
delete pkg.scripts;

fs.writeFileSync("./dist/package.json", JSON.stringify(pkg, null, 4));
