//@ts-check

"use strict";

const path = require("path");

/**@type {import('webpack').Configuration}*/
const config = {
  target: "node",
  entry: "./src/spIndex.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "spIndex.js",
    libraryTarget: "commonjs2",
    devtoolModuleFilenameTemplate: "../[resource-path]",
  },
  devtool: "source-map",
  externals: {
    vscode: "commonjs vscode",
  },
  resolve: {
    extensions: [".ts", ".js", ".node", ".wasm"],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: [
          {
            loader: "ts-loader",
          },
        ],
      },
      {
        test: /\.node$/,
        loader: "node-loader",
      },
      {
        test: /\.wasm$/,
        type: "asset/inline",
      },
    ],
  },
};
module.exports = config;
