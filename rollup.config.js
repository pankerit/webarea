import path from "path";
import typescript from "@rollup/plugin-typescript";
import replace from "@rollup/plugin-replace";
import dts from "rollup-plugin-dts";
import pkg from "./package.json";
import { terser } from "rollup-plugin-terser";

const isDev = process.env.NODE_ENV === "development";

/**
 * @type {import('rollup').RollupOptions}
 */
const rollupData = {
    input: "./lib/index.ts",
    output: {
        file: "./dist/index.js",
        format: "cjs",
    },
    plugins: [
        typescript(),
        isDev ? false : terser(),
        replace({
            "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV),
        }),
    ].filter(Boolean),
};

export default [
    rollupData,
    {
        // path to your declaration files root
        input: "./dist/dts/index.d.ts",
        output: [{ file: "dist/index.d.ts", format: "es" }],
        plugins: [dts()],
    },
];
