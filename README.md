
# SourcePawn for VSCode

SourcePawn highlighting and autocompletion for Visual Studio Code. Supports the SourceMod 1.7+ syntax.

![Version](https://vsmarketplacebadge.apphb.com/version/Sarrus.sourcepawn-vscode.svg) ![Installs](https://vsmarketplacebadge.apphb.com/installs-short/Sarrus.sourcepawn-vscode.svg) ![Last commit](https://img.shields.io/github/last-commit/Sarrus1/sourcepawn-vscode) ![Open issues](https://img.shields.io/github/issues/Sarrus1/sourcepawn-vscode) ![Closed issues](https://img.shields.io/github/issues-closed/Sarrus1/sourcepawn-vscode) ![Size](https://img.shields.io/github/repo-size/Sarrus1/sourcepawn-vscode) 

![Example](https://raw.githubusercontent.com/Sarrus1/sourcepawn-vscode/master/images/example.gif)

## Features
- Automatically scan include files for natives, defines, methodmaps and more.
- Useful snippets.
- Variables autocompletion.
- Functions autocompletion with arguments descriptions.
- Detailed highlighting.
- Allows to parse sourcemod files from a custom location.

## Settings
The only setting allows to define the position of the default sourcemod include files :
```json
{
    "sourcepawnLanguageServer.sourcemod_home": "/path/to/sourcemod/include"
}
```

## To do
- Add _hover_ help for functions.
- Add _Goto Definition_ for functions and natives.
- Add a compile from VSCode feature.
- Add auto-formatting.
- Add dynamic syntax highlighting for imported types.
- Add project templates.
- Add more snippets.

## Credits
This is an improved version of [Dreae's](https://github.com/Dreae/sourcepawn-vscode) extension which doesn't seem to be supported anymore.

## How to contribute
Pull requests and suggestions are welcome.
 - To make a suggestion or to report an issue, please create a new issue [here](https://github.com/Sarrus1/sourcepawn-vscode/issues).
 - To make a contribution, fork the reposetory, make the desired changes, and open a pull request.

## How to run
To run the extension in dev mode, do the following:
 - Install [node.js](https://nodejs.org/en/) on your machine.
 - Fork this reposetory and clone it to your machine (do this with VSCode for easier manipulation).
 - Run `npm install` from the root of the project folder.
 - Press `ctrl+B` and then `enter` to compile the typescript files.
 - Press `f5` to run the extension in dev mode.
 - Once you're done, save and commit and create a new pull request.