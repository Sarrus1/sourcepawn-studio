---
sidebar_position: 2
---

# Visual Studio Code

Through the extension, VSCode has the best support for SourcePawn development out of all the other editors. The extension provides features that the Language Server alone cannot provide such as plugin compilation and uploading files to a server automatically.

## Installation

The extension can be installed from the [VSCode Marketplace](https://marketplace.visualstudio.com/items?itemName=Sarrus.sourcepawn-vscode). If you are using VSCodium, you can install the same extension from the [Open VSX](https://open-vsx.org/?search=sourcepawn) registry.

## Supported platforms

The extension bundles a binary of the latest version of the Language Server. This binary is located in a directory which name's starts with `sarrus.sourcepawn-vscode` in:

- **Linux:** `~/.vscode/extensions`
- **Linux (Remote, such as WSL):** `~/.vscode-server/extensions`
- **macOS:** `~/.vscode/extensions`
- **Windows:** `%USERPROFILE%\.vscode\extensions`

Inside that directory, the binary is stored in the `languageServer` directory.

We only built for the most popular platforms and OS, therefore the extension is available on:

- **Linux:** arm64, armhf, x64
- **macOS:** arm64, x64
- **Windows:** arm64, ia32, x64

## Updates

The extension is updated automatically when a new release is published to the Marketplace, with the associated Language Server binary.

## Pre-releases

We sometimes publish pre-releases to the Marketplace to experiment with new features. You can opt-in or out at any time from the extension's page in VSCode or VSCodium.

## Manual installation

Alternatively, download a VSIX corresponding to your platform from the [releases](https://github.com/Sarrus1/sourcepawn-vscode/releases/latest) page.

Install the extension with the `Extensions: Install from VSIX` command within VS Code, or from the command line via:

```shell
code --install-extension /path/to/sourcepawn-studio.vsix
```

You can also drag and drop the VSIX file into the VSCode Extensions menu.
