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

# Features

<div align="center">
<table class="tg">
<thead>
  <tr>
    <th><span style="font-weight:bold"><b>Feature</b></span></th>
    <th><span style="font-weight:bold"><b>Status</b></span></th>
  </tr>
</thead>
<tbody>
  <tr>
    <td>Completion</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Go To Definition</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Find References</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Hover</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Rename</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Semantic Highlighting</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Document Symbols</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Call Hierarchy</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Signature Help</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Reference</td>
    <td>✅</td>
  </tr>
  <tr>
    <td>Diagnostics</td>
    <td>✅</td>
  </tr>
</tbody>
</table>
</div>

# Text editor support

The Language Server is compatible with any text editor that implements the Language Server Protocol.

## VSCode support

For ease of use with [Visual Studio Code](https://code.visualstudio.com/), an extension bundles `sourcepawn-lsp` with a build for each major platform and OS. You can download it from the marketplace [here](https://marketplace.visualstudio.com/items?itemName=Sarrus.sourcepawn-vscode).

# AMXXPawn support

Partial support for `AMXXPawn` is implemented through the `--amxxpawn-mode` flag. The only difference between, the "SourcePawn mode" and the "AMXXPawn mode" boils down to what file extension the server will be looking for (`.sp` vs `.sma`).

A VSCode extension exists which bundles the server with the correct launch flags. You can find it [here](https://marketplace.visualstudio.com/items?itemName=Sarrus.amxxpawn-vscode).

# Frequently Asked Questions

- [How can I donate ?](#how-can-i-donate-)
- [How to install the beta build ?](#how-to-install-the-beta-build-)
- [How to contribute ?](#how-to-contribute-)
- [How to run the extension from its source code ?](#how-to-run-the-extension-from-its-source-code-)

## How can I donate ?

Thanks for considering this. But please remember that all of this wouldn't be possible without the Alliedmodders community. If you wish to make a donation for this community, you can make it [here](https://sourcemod.net/donate.php).
If you prefer to make a donation to me for this project, you can [buy me a coffee](https://www.buymeacoffee.com/sarrus)

## How to install the beta build ?

### Standalone Language Server

1. Download and install the language server from the [releases page](https://github.com/Sarrus1/sourcepawn-vscode/releases).

### VSCode extension

1. Open VSCode and go to the marketplace.
2. In the search bar, type `SourcePawn` and select the extension.
3. Click on `Switch to Pre-Release` (see screenshot below).

![Pre-release](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/main/images/pre-release.png)

This will install the latest development build of the extension.

Note that a new build might take a few minutes (up to 15) to propagate to the Marketplace after the release has been pushed to the releases page.

You can also install the latest pre-release by downloading the .vsix from the [releases page](https://github.com/Sarrus1/sourcepawn-vscode/releases) and [installing it manually](https://code.visualstudio.com/docs/editor/extension-marketplace#_install-from-a-vsix).

## How to contribute ?

Pull requests and suggestions are welcome.

- To make a suggestion or to report an issue, please create a new issue [here](https://github.com/Sarrus1/sourcepawn-vscode/issues).
- To make a contribution, fork the repository, make the desired changes, and open a pull request.

## How to run the Language Server from its source code ?

To run the Language Server from the source, do the following:

- Install the rust toolchain using [rustup](https://rustup.rs/).
- Install [node.js](https://nodejs.org) on your machine. Make sure [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm/) is installed as well.
- Fork this repository and clone it on your machine (do this with VSCode for easier manipulation).
- From the root of the repository, run the following commands to install npm dependencies: `cd editors/code && npm i`.
- Press `f5` and select the `Run Extension (Debug Build)` launch task in the prompt. This will compile the Language Server server from the source code and package the VSCode extension's source code.
- (Optional) You can attach a debugger to the Language Server by doing the step above, then running another task called `Attach To Server` or `Win Attach To Server` on Windows and typing `sourcepawn-lsp` in the prompt.

</div>
