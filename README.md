# SourcePawn for VSCode

SourcePawn highlighting and autocompletion for Visual Studio Code. Supports the SourceMod 1.7+ syntax.

![Version](https://vsmarketplacebadge.apphb.com/version/Sarrus.sourcepawn-vscode.svg) ![Installs](https://vsmarketplacebadge.apphb.com/installs-short/Sarrus.sourcepawn-vscode.svg) ![Last commit](https://img.shields.io/github/last-commit/Sarrus1/sourcepawn-vscode) ![Open issues](https://img.shields.io/github/issues/Sarrus1/sourcepawn-vscode) ![Closed issues](https://img.shields.io/github/issues-closed/Sarrus1/sourcepawn-vscode) ![Size](https://img.shields.io/github/repo-size/Sarrus1/sourcepawn-vscode) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Sarrus1/sourcepawn-vscode/Package%20Extension)

![Example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/master/images/example.gif)

## Features

- Compile from VSCode with a simple button.
- Autocompletion.
- Go to definition.
- Add icons for `.smx`, `.sp` and `.inc` files
- Automatically scan include files for natives, defines, methodmaps and more.
- Useful snippets.
- Hover for details.
- Detailed highlighting for `.sp`, `.inc`, `.cfg`, `.games.txt` and `.phrases.txt`.
- Parse sourcemod files from a custom location.
- Linting capabilities.

## Settings

The only setting allows to define the position of the default sourcemod include files :

```json
{
  "sourcepawn.SourcemodHome": "/path/to/sourcemod/include"
}
{
	"sourcepawn.AuthorName": "The name of the plugin's author (you)."
},
{
	"sourcepawn.GithubName": "The GitHub username of the plugin's author (you)."
},
{
	"sourcepawn.SpcompPath": "The location of the SourceMod compiler"
},
{
	"sourcepawn.showCompileIconInEditorTitleMenu": "Whether to show the 'Compile Code' icon in editor title menu."
},
{
	"sourcepawn.optionalIncludeDirsPaths": "Optional additional include folders paths for the compiler. Use this if you know what you are doing. Leave blank to disable."
}
```

## To do

- Incrementally Format Code as the User Types
- Automatic callback generation.
- Automatic convert to new syntax (partially).

## Credits

- [Dreae's](https://github.com/Dreae/sourcepawn-vscode) extension which is not supported anymore, and on which this extension is based on.
- [Deathreus'](https://github.com/Deathreus/SPLinter) extension which helped me to implement the linting feature.
- Everybody that has helped me improved the extension on the [discord server](https://discord.tensor.fr).

## Frequently Asked Questions

- [How to fix "Not a .sp file, aborting" ?](#How-to-fix-"Not-a-.sp-file,-aborting"-?)
- [How to fix "Command not found" ?](#How-to-fix-"command-not-found"-?)
- [How to install the beta build ?](#How-to-install-the-beta-build-?)
- [How can I contribute ?](#How-to-contribute-?)
- [How to run the dev version ?](#How-to-run-?)

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

### How to run ?

To run the extension in dev mode, do the following:

- Install [node.js](https://nodejs.org/en/) on your machine.
- Fork this repository and clone it to your machine (do this with VSCode for easier manipulation).
- Run `npm install` from the root of the project folder.
- Run `npm run watch`
- Press `f5` to run the extension in dev mode.
- Once you're done, save and commit and create a new pull request.
