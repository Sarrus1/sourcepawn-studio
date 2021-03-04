
# SourcePawn for VSCode

SourcePawn highlighting and autocompletion for Visual Studio Code. Supports the SourceMod 1.7+ syntax.

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

## Credits
This is an improved version of [Dreae's](https://github.com/Dreae/sourcepawn-vscode) extension which doesn't seem to be supported anymore.

## To do
- Add _hover_ help for functions.
- Add _Goto Definition_ for functions and natives.
- Add a compile from VSCode feature.
- Add auto-formatting.
- Add dynamic syntax highlighting for imported types.
- Add project templates.
- Add more snippets.
