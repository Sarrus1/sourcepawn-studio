---
sidebar_position: 1
---

# Quick Start

## Installation

### Visual Studio Code

1. [Install VSCode](https://code.visualstudio.com/) if you haven't done so already.
2. Download and install the extension from the [Marketplace](https://marketplace.visualstudio.com/items?itemName=Sarrus.sourcepawn-vscode). Clicking on _Install_ should open VSCode and automatically install the extension for you.

### VSCodium

We publish the exact same version of the VSCode extension to [https://open-vsx.org/](https://open-vsx.org/). You can install the extension with the exact same feature as VSCode, using VSCodium.

### Other LSP Clients (Neovim, Lapce)

LSP binaries are shipped with each [release](https://github.com/Sarrus1/sourcepawn-vscode/releases/latest).

## Configuration

Once the VSCode extension is installed, the quickest way to setup your Sourcemod environment is to press `Ctrl + Shift + P` and type `SM: Install Sourcemod`, press enter and choose the version you wish to use. This will download and configure the desired latest build of `spcomp` and the Sourcemod includes.

For more information see the [Configuration](./configuration/generated_settings.md) section.

## Editing a project

After setting up your environment, you can open any SourcePawn project and start editing it!

:::caution

To get the best editing experience, make sure that you open the parent folder of your project and not just a single file in VSCode. Otherwise, some project references may not be resolved correctly.
