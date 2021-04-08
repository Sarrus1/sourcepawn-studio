
# SourcePawn for VSCode

SourcePawn highlighting and autocompletion for Visual Studio Code. Supports the SourceMod 1.7+ syntax.

![Version](https://vsmarketplacebadge.apphb.com/version/Sarrus.sourcepawn-vscode.svg) ![Installs](https://vsmarketplacebadge.apphb.com/installs-short/Sarrus.sourcepawn-vscode.svg) ![Last commit](https://img.shields.io/github/last-commit/Sarrus1/sourcepawn-vscode) ![Open issues](https://img.shields.io/github/issues/Sarrus1/sourcepawn-vscode) ![Closed issues](https://img.shields.io/github/issues-closed/Sarrus1/sourcepawn-vscode) ![Size](https://img.shields.io/github/repo-size/Sarrus1/sourcepawn-vscode) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Sarrus1/sourcepawn-vscode/Package%20Extension)

![Example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/master/images/example.gif)

## Features
- Compile from vscode with a simple button.
- Go to definition.
- Add icons for `.smx`, `.sp` and `.inc` files
- Automatically scan include files for natives, defines, methodmaps and more.
- Useful snippets.
- Variables autocompletion.
- Functions autocompletion with arguments descriptions.
- Detailed highlighting.
- Allows to parse sourcemod files from a custom location.
- Linter capabilities.

## Settings
The only setting allows to define the position of the default sourcemod include files :
```json
{
  "sourcepawnLanguageServer.sourcemod_home": "/path/to/sourcemod/include"
}
{
	"sourcepawnLanguageServer.author_name": "The name of the plugin's author (you)."
},
{
	"sourcepawnLanguageServer.github_name": "The GitHub username of the plugin's author (you)."
},
{
	"sourcepawnLanguageServer.spcomp_path": "The location of the SourceMod compiler"
},
{
	"sourcepawnLanguageServer.showCompileIconInEditorTitleMenu": "Whether to show the 'Compile Code' icon in editor title menu."
},
{
	"sourcepawnLanguageServer.optionalIncludeDirsPaths": "Optional additional include folders paths for the compiler. Use this if you know what you are doing. Leave blank to disable."
}
```

## To do
- Incrementally Format Code as the User Types
- Add FTP/SFTP support and auto `sm_refresh`.
- Add dynamic syntax highlighting for imported types.
- Download sourcemod files automatically.
- Add syntax highlighting for .cfg files.
- Automatic callback generation.
- Automatic convert to new syntax (partially).

## Credits
 - [Dreae's](https://github.com/Dreae/sourcepawn-vscode) extension which is not supported anymore, and on which this extension is based on.
 - [Deathreus'](https://github.com/Deathreus/SPLinter) extension which helped me to implement the linting feature.


## How to contribute
Pull requests and suggestions are welcome.
 - To make a suggestion or to report an issue, please create a new issue [here](https://github.com/Sarrus1/sourcepawn-vscode/issues).
 - To make a contribution, fork the repository, make the desired changes, and open a pull request.

## How to run
To run the extension in dev mode, do the following:
 - Install [node.js](https://nodejs.org/en/) on your machine.
 - Fork this repository and clone it to your machine (do this with VSCode for easier manipulation).
 - Run `npm install` from the root of the project folder.
 - Press `Ctrl+Shift+B` and then `enter` to compile the typescript files.
 - Press `f5` to run the extension in dev mode.
 - Once you're done, save and commit and create a new pull request.
