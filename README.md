# SourcePawn

SourcePawn highlighting and autocompletion for Visual Studio Code. Supports the SourceMod 1.7+ syntax.

The extension automatically scans `.inc` files for functions and documentation comments to generate
autocompletion for your SourceMod projects. It scans for the SourceMod API functions in the `include/`
relative to the currently open SourcePawn file, if you keep your code seperate from the SourceMod
includes you can tell the extension where to find SourceMod with the `sourcepawnLanguageServer.sourcemod_home`
setting:

```json
{
    "sourcepawnLanguageServer.sourcemod_home": "/path/to/sourcemod/include"
}
```