# Web

## Required Formats

### ESM

### CJS/CommonJS

### UMD

## Required Features

## Bundler Comparison

| Bundler       | ESM | CJS | Bundle Inlining |
|---------------|-----|-----|-----------------|
| Babel 1)      | ✅   | ❌   | ❌               |
| TypeScript 1) | ✅   | ❌   | ❌               |
| Webpack       | ❌   | ❌   | ❌ 2)            |
| Parcel        | ✅   | ✅   | 🛠️ 3)          |


> 1) Technically not a bundler but can be used to emit ES modules
> 2) Was Supported in Webpack 4, but currently is not supported
> 3) https://github.com/parcel-bundler/parcel/issues/8004

## WebWorker Inlining