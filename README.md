<div align="center">
  <h1><code>SourcePawn Language Server</code></h1>
  <p>
    <strong>Language Server Protocol implementation for the SourcePawn programming language written in Rust</strong>
  </p>
  <p style="margin-bottom: 0.5ex;">
    <a href="https://github.com/Sarrus1/sourcepawn-vscode/releases/">
      <img
        src="https://img.shields.io/visual-studio-marketplace/v/Sarrus.sourcepawn-vscode?include_prereleases"
        />
    </a>
    <a href="https://github.com/Sarrus1/sourcepawn-vscode/releases/latest">
      <img
        src="https://img.shields.io/visual-studio-marketplace/i/Sarrus.sourcepawn-vscode"
        />
    </a>
    <a href="https://github.com/Sarrus1/sourcepawn-vscode/releases/latest">
      <img
        src="https://img.shields.io/visual-studio-marketplace/d/Sarrus.sourcepawn-vscode"
        />
    </a>
    <a href="https://github.com/Sarrus1/sourcepawn-vscode/actions/workflows/ci.yml">
      <img
        alt="CI status"
        src="https://github.com/Sarrus1/sourcepawn-vscode/actions/workflows/ci.yml/badge.svg"
      />
    </a>
    <img alt="GitHub" src="https://img.shields.io/github/license/Sarrus1/sourcepawn-lsp">
  </p>
  <br>
  <br>
  <p>
    <img alt="Example" src="https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/example.gif">
  </p>
  <!-- <img src="https://raw.githubusercontent.com/Sarrus1/sourcepawn-lsp/main/img/logo.png" alt="Logo"> -->
</div>

## Features

|      **Feature**      | **Status** |
| :-------------------: | :--------: |
|      Completion       |     ✅     |
|   Go To Definition    |     ✅     |
|    Find References    |     ✅     |
|         Hover         |     ✅     |
|        Rename         |     ✅     |
| Semantic Highlighting |     ✅     |
|   Document Symbols    |     ✅     |
|    Call Hierarchy     |     ✅     |
|    Signature Help     |     ✅     |
|       Reference       |     ✅     |
|      Diagnostics      |     ✅     |

**Details on how to use the extension available in the [Wiki](https://github.com/Sarrus1/sourcepawn-vscode/wiki)**.
**If you encounter an issue, press CTRL+SHIFT+P and type "SM: Doctor" to diagnose the problem.**

- [Features](#features)
- [Screenshots](#screenshots)
  - [.sp and .inc file highlighting](#sp-and-inc-file-highlighting)
  - [.cfg file highlighting](#cfg-file-highlighting)
  - [Code auto-completion](#code-auto-completion)
  - [Event auto-completion](#event-auto-completion)
  - [Include auto-completion](#include-auto-completion)
  - [Callback auto-completion](#callback-auto-completion)
  - [Code outline](#code-outline)
  - [Call Hierarchy](#call-hierarchy)
  - [Symbol references](#symbol-references)
  - [Symbol renaming](#symbol-renaming)
  - [Functions signature help](#functions-signature-help)
  - [Hover help](#hover-help)
  - [Go To Definition](#go-to-definition)
  - [Linter](#linter)
- [Credits](#credits)
- [Frequently Asked Questions](#frequently-asked-questions)
  - [How can I donate ?](#how-can-i-donate-)
  - [How to install the beta build ?](#how-to-install-the-beta-build-)
  - [How to contribute ?](#how-to-contribute-)
  - [How to run the extension from its source code ?](#how-to-run-the-extension-from-its-source-code-)

## Screenshots

### .sp and .inc file highlighting

![.sp highlighting example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/highlighting-example-1.png)

### .cfg file highlighting

![Highlighting example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/highlighting-example-2.png)

### Code auto-completion

![Completion example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/completion-example-1.png)

### Event auto-completion

![Completion example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/completion-example-2.png)

### Include auto-completion

![Completion example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/completion-example-3.png)

### Callback auto-completion

![Completion example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/completion-example-4.gif)

### Code outline

![Outline example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/outline-example-1.png)

### Call Hierarchy

![Hierarchy example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/hierarchy-example-1.gif)

### Symbol references

![References example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/references-example-1.png)

### Symbol renaming

![Renaming example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/rename-example-1.png)

### Functions signature help

![Signature example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/signature-example-1.png)

### Hover help

![Hover example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/hover-example-1.png)

### Go To Definition

![Go to definition example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/go-to-definition-example-1.png)

### Linter

![Linter example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/linter-example-1.png)

## Credits

- [Dreae's](https://github.com/Dreae/sourcepawn-vscode) extension which is not supported anymore, and on which this extension is based on.
- [Deathreus'](https://github.com/Deathreus/SPLinter) extension which helped me to implement the linting feature.
- Everybody that has helped me improved the extension on the [discord server](https://discord.com/invite/u2Z7dfk).

## Frequently Asked Questions

- [How can I donate ?](#how-can-i-donate-)
- [How to install the beta build ?](#how-to-install-the-beta-build-)
- [How to contribute ?](#how-to-contribute-)
- [How to run the extension from its source code ?](#how-to-run-the-extension-from-its-source-code-)

### How can I donate ?

Thanks for considering this. But please remember that all of this wouldn't be possible without the Alliedmodders community. If you wish to make a donation for this community, you can make it [here](https://sourcemod.net/donate.php).
If you prefer to make a donation to me for this project, you can [buy me a coffee](https://www.buymeacoffee.com/sarrus)

### How to install the beta build ?

1. Open VSCode and go to the marketplace.
2. In the search bar, type `SourcePawn` and select the extension.
3. Click on `Switch to Pre-Release` (see screenshot below).

![Pre-release](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/pre-release.png)

This will install the latest development build of the extension.

Note that a new build might take a few minutes (up to 15) to propagate to the Marketplace after the commit has been pushed to the dev branch.

You can also install the latest pre-release by downloading the .vsix from the [releases page](https://github.com/Sarrus1/sourcepawn-vscode/releases) and [installing it manually](https://code.visualstudio.com/docs/editor/extension-marketplace#_install-from-a-vsix).

### How to contribute ?

Pull requests and suggestions are welcome.

- To make a suggestion or to report an issue, please create a new issue [here](https://github.com/Sarrus1/sourcepawn-vscode/issues).
- To make a contribution, fork the repository, make the desired changes, and open a pull request.

### How to run the extension from its source code ?

To run the extension from the source, do the following:

- Install [node.js](https://nodejs.org) on your machine. Make sure [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm/) is installed as well.
- Fork this repository and clone it on your machine (do this with VSCode for easier manipulation).
- Run `npm install` from the root of the project folder.
- Run `npm run watch`.
- Press `f5` to run the extension in dev mode.

</div>
