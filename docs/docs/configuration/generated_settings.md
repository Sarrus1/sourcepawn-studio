---
id: lsp-settings-reference
title: Server Settings Reference
---

## cachePriming.enable

**SourcePawnLanguageServer.cachePriming.enable**

Warm up caches on project load.

_Default_: `true`

## cachePriming.numThreads

**SourcePawnLanguageServer.cachePriming.numThreads**

How many worker threads to handle priming caches. The default `0` means to pick automatically.

_Default_: `0`

## compiler.arguments

**SourcePawnLanguageServer.compiler.arguments**

Linter arguments that will be passed to spcomp.
Note that the compilation target, include directories and output path are already handled by the server.

_Default_: `[]`

## compiler.onSave

**SourcePawnLanguageServer.compiler.onSave**

Compute spcomp diagnostics on save.

_Default_: `true`

## compiler.path

**SourcePawnLanguageServer.compiler.path**

Path to the SourcePawn compiler (spcomp).

_Default_: `null`

## eventsGameName

**SourcePawnLanguageServer.eventsGameName**

Name of the game we want the events for, as it appears on the Alliedmodders website.
For example, "Counter-Strike: Global Offensive" or "Team Fortress 2".

_Default_: `null`

## hover.actions.debug.enable

**SourcePawnLanguageServer.hover.actions.debug.enable**

Whether to show `Debug` action. Only applies when
[`SourcePawnLanguageServer.hover.actions.enable`](#hoveractionsenable) is set.

_Default_: `true`

## hover.actions.enable

**SourcePawnLanguageServer.hover.actions.enable**

Whether to show HoverActions in Sourcepawn files.

_Default_: `true`

## hover.actions.gotoTypeDef.enable

**SourcePawnLanguageServer.hover.actions.gotoTypeDef.enable**

Whether to show `Go to Type Definition` action. Only applies when
[`SourcePawnLanguageServer.hover.actions.enable`](#hoveractionsenable) is set.

_Default_: `true`

## hover.actions.implementations.enable

**SourcePawnLanguageServer.hover.actions.implementations.enable**

Whether to show `Implementations` action. Only applies when
[`SourcePawnLanguageServer.hover.actions.enable`](#hoveractionsenable) is set.

_Default_: `true`

## hover.actions.references.enable

**SourcePawnLanguageServer.hover.actions.references.enable**

Whether to show `References` action. Only applies when
[`SourcePawnLanguageServer.hover.actions.enable`](#hoveractionsenable) is set.

_Default_: `false`

## hover.actions.run.enable

**SourcePawnLanguageServer.hover.actions.run.enable**

Whether to show `Run` action. Only applies when
[`SourcePawnLanguageServer.hover.actions.enable`](#hoveractionsenable) is set.

_Default_: `true`

## includeDirectories

**SourcePawnLanguageServer.includeDirectories**

Include directories paths for the compiler and the linter.

_Default_: `[]`

## linter.disable

**SourcePawnLanguageServer.linter.disable**

Disable the language server's syntax linter. This is independant from spcomp.

_Default_: `false`

## numThreads

**SourcePawnLanguageServer.numThreads**

How many worker threads in the main loop. The default `null` means to pick automatically.

_Default_: `null`

