<img src="https://i.imgur.com/StGaL7A.png" alt="Frame 321" style="zoom:80%;" /> 

------



Webarea is a lightweight desktop application development library. It lets you write cross-platform applications using JavaScript, HTML and CSS. With the power of NodeJS and WebView you can write production apps with small bundle size.

- **Fast**: Webarea is is built in Rust and is powered by NodeJS's runtime.
- **Easy to use**: The API to create and modify windows is simple, developer friendly and similar to Electron.
- **Light**: Webarea uses WebView to render web pages, and has a size of **1MB.**
- **Simple**: No external binaries or dependencies, just one native module.



## Installation

To install prebuilt Webarea modules, use `npm` or `yarn`. Because it's a library, it's meant to be shipped with the bundled application therefore install it as a normal dependency.

```
yarn add webarea
```

or

```
npm install webarea
```



## Usage

To create a window, import the Webview class

```ts
import { Webview } from "webarea";
```



Create a variable and initiate Webview.

```ts
const window = new Webview({
    title: "Sample app"
})
```



To modify the window use available methods

```ts
await window.setTitle("Hello world")
```



For more information, please visit our documentation.



## Features

- Renderer and Backend IPC

- Window creation

- Window modification

- Local or Remote page loading

- Multiple windows

- Rich API

  

## Platform and Compatibility

> Webarea is built in Rust, but compiled into a single .node file which is loaded into NodeJS via default importing methods, therefore it works on all platforms which support NodeJS and WebView.

- Windows 7,8,10,11
- Linux
- macOS



## Why Webarea ?

**The Problem**

There are many frameworks for building desktop apps with rich API for creating and modifying existing windows, but most of them focus on the development process, therefore ignoring the bundling part.

**The Solution**

Webarea is a ultra-lightweight library, powered by WebView engine and NodeJS's runtime. It's fully developed in Rust and compiled **into a single .node file which has a size of 1MB**.

| Framework/Library | NodeJS Backend | Lightweight | No external binaries/files | Cross-Platform |
| ----------------- | -------------- | ----------- | -------------------------- | -------------- |
| Webarea           | ✅              | ✅           | ✅                          | ✅              |
| Neutralino        | ✅              | ✅           | ❌                          | ✅              |
| Electron          | ✅              | ❌           | ❌                          | ✅              |
| NW.JS             | ✅              | ❌           | ❌                          | ✅              |



| Framework/Library | Bundling Size |
| ----------------- | ------------- |
| Webarea           | ~40mb         |
| Neutralino        | ~40mb         |
| Electron          | ~150mb        |
| NW.JS             | ~150mb        |





## Bundling

Because Webarea does not ship with any external binaries or files, it can be easily packaged using any NodeJS packager.

- [Caxa](https://github.com/leafac/caxa)
- [Boxednode](https://github.com/mongodb-js/boxednode)
- [pkg](https://github.com/vercel/pkg)
