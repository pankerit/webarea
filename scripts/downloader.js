const http = require("https"); // or 'https' for https:// URLs
const fs = require("fs");
const path = require("path");
const pkg = require("./package.json");

const platform = process.platform;
const arch = process.arch;
const fileName = `webarea_${platform}_${arch}.node`;
const file = fs.createWriteStream(path.join(__dirname, fileName));
const url = `https://github.com/pankerit/webarea/releases/download/v${pkg.version}/${fileName}`;
console.log(`Downloading ${url}`);
http.get(url, function (response) {
    http.get(response.headers.location, (res) => {
        res.pipe(file);
        file.on("finish", () => {
            file.close();
            console.log(`Download Completed ${fileName}`);
        });
    });
});
