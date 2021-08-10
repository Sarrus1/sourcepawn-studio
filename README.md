# SourcePawn for VSCode

SourcePawn highlighting and autocompletion for Visual Studio Code. Supports the SourceMod 1.7+ syntax.

![Version](https://vsmarketplacebadge.apphb.com/version/Sarrus.sourcepawn-vscode.svg) ![Installs](https://vsmarketplacebadge.apphb.com/installs-short/Sarrus.sourcepawn-vscode.svg) ![Last commit](https://img.shields.io/github/last-commit/Sarrus1/sourcepawn-vscode) ![Open issues](https://img.shields.io/github/issues/Sarrus1/sourcepawn-vscode) ![Closed issues](https://img.shields.io/github/issues-closed/Sarrus1/sourcepawn-vscode) ![Size](https://img.shields.io/github/repo-size/Sarrus1/sourcepawn-vscode) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Sarrus1/sourcepawn-vscode/Package%20Extension)

![Example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/master/images/example.gif)

## Features

- Compile from VSCode with a simple button.
- Autocompletion.
- Go to definition.
- Upload to an FTP/SFTP server on successful compile.
- Automatically run `sm plugins refresh` on a successful upload.
- Hover for help.
- Add icons for `.smx`, `.sp` and `.inc` files
- Automatically scan include files for natives, defines, methodmaps and more.
- Useful snippets.
- Hover for details.
- Detailed highlighting for `.sp`, `.inc`, `.cfg`, `.games.txt` and `.phrases.txt`.
- Parse sourcemod files from a custom location.
- Linting capabilities.

## Screenshots

### .sp and .inc file highlighting

![.sp highlighting example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/highlighting-example-1.png)

### .cfg file highlighting

![Highlighting example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/highlighting-example-2.png)

### Code auto-completion

![Completion example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/completion-example-1.png)

### Event auto-completion

![Completion example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/completion-example-2.png)

### Include auto-completion

![Completion example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/completion-example-3.png)

### Functions signature help

![Signature example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/signature-example-1.png)

### Hover help

![Hover example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/hover-example-1.png)

### Go To Definition

![Go to definition example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/go-to-definition-example-1.png)

### Linter

![Linter example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/dev/images/linter-example-1.png)

## Credits

- [Dreae's](https://github.com/Dreae/sourcepawn-vscode) extension which is not supported anymore, and on which this extension is based on.
- [Deathreus'](https://github.com/Deathreus/SPLinter) extension which helped me to implement the linting feature.
- Everybody that has helped me improved the extension on the [discord server](https://discord.tensor.fr).

## Frequently Asked Questions

- [How can I donate ?](#how-can-i-donate-)
- [How to fix "Not a .sp file, aborting" ?](#how-to-fix-not-a-sp-file-aborting-")
- [How to fix "Command not found" ?](#how-to-fix-command-not-found-)
- [How to install the beta build ?](#how-to-install-the-beta-build-)
- [How can I contribute ?](#how-to-contribute-)
- [How to run the dev version ?](#how-to-run-)

### How can I donate ?

Thanks for considering this. But please remember that all of this wouldn't be possible without the Alliedmodders community. If you wish to make a donation for this community, you can make it [here](https://sourcemod.net/donate.php).
If you prefer to make a donation to me for this project, you can [buy me a coffee](https://www.buymeacoffee.com/sarrus)

### How to fix "Not a .sp file, aborting" ?

This usually happens when you have `sourcepawn.MainPath` set but don't actually need it. This setting is only to be used for large projects with multiple `.sp` files, as a way to provide an entry point for the compiler.
Don't use this setting if you don't need to.
NOTE: This error shouldn't appear as much as of version 1.12.0.

### How to fix "Command not found" ?

The `Command not found` error means that the extension could not start properly.
This is usually due to the user trying to open a single file only and not a folder.
As of version 1.12.0, this is partly supported, however, for full support, please open a folder when editing a project, and not just the `.sp` file.

### How to install the beta build ?

First, go to the [releases page](https://github.com/Sarrus1/sourcepawn-vscode/releases) and download the `.vsix` file attached to the latest release.
Then, open VSCode and in the extension manager, click on the `...` icon and select install from VSIX.
Select the file you've just downloaded and you're done.

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
