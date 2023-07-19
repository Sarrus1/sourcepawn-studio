## Release Notes

## [0.10.14]

### Fixed

-   Fixed tree-sitter bugs.
-   Fixed `sourcemod.inc` not being included automatically.

### Removed

-   Removed deprecated diagnostics in `.inc` files.

## [0.10.13]

### Added

-   Added support for arguments overflow in preprocessor macros.
-   Added #undef support in preprocessor.
-   Added semantic resolution for macros.
-   Added a debug request to get the preprocessed text for a document.

### Fixed

-   Fixed disable syntax linter not being respected.

## [0.10.12]

### Added

-   Added support for stringizing in preprocessor.

### Fixed

-   Fixed "receiving on an empty or disconnected channel" bug when stopping the server.

## [0.10.11]

### Fixed

-   Fixed a crash when loading files that were not yet saved to disk (see #31).

## [0.10.10]

### Fixed

-   Fixed resolving references for too many files (this can improve parsing times by 300%).
-   Improved method resolution to use the AST instead of primitive text parsing.
-   Removed dangerous unwraps.

## [0.10.9]

### Fixed

-   Fixed out of range error when doing semantic analysis.
-   Removed dangerous unwraps.

## [0.10.8]

### Fixed

-   Fixed reading files if they are not sourcepawn files.

## [0.10.7]

### Fixed

-   Fixed huge performance dropoffs when iterating through `.git` folders.
-   Fixed incorrect error handling.

## [0.10.6]

### Added

-   Added more debugging traces.

### Fixed

-   Fixed Sentry hostname data leak.
-   Fixed early returns in parser when encountering an error.

## [0.10.5]

### Fixed

-   Fixed Sentry instantiation.

## [0.10.4]

### Fixed

-   Fixed BOM support.
-   Fixed disabled code diagnostics being skipped by the preprocessor.

## [0.10.3]

### Added

-   Added optional crash reports telemetry.

## [0.10.2]

### Fixed

-   Fixed support for `#tryinclude`.
-   Fixed incorrect macro offsetting.
-   Fixed unknown tokens failing the preprocessor.
-   Fixed incorrect arguments indexing in macro expansion.
-   Fixed infinite recursion when resolving includes.
-   Removed anyhow errors from logs.

## [0.10.1]

### Fixed

-   Allow non define identifiers in macro expansions.
-   Fixed trailing macro comment expansion issue.
-   Fixed empty preprocessed text on preprocessing failure.
-   Fixed default completions in includes completions.
-   Fixed early aggressive propagation in providers.
-   Fixed some potential unwraps.

## [0.10.0]

### Added

-   Added full preprocessor support. The extension will now preprocess the files, by expanding macros and evaluating if conditions.
-   Added basic logging and tracing.

## [0.9.10]

### Fixed

-   Fixed descriptions not appearing on hover.

## [0.9.9]

### Added

-   Added better error reporting to the client.

### Fixed

-   Fixed a crash when parsing spcomp's output.
-   Fixed the server dying too easily.

## [0.9.8]

### Fixed

-   Fixed a crash when the mainPath setting was empty.

## [0.9.7]

### Added

-   Added a setting to disable the syntax linter.
-   Added support for mainPaths relative to the workspace's root.

### Fixed

-   Updated the parser to take into account new syntax elements.

## [0.9.6]

### Added

-   Added support for AMXXPawn.

## [0.9.5]

### Added

-   Added call hierarchy provider.

## [0.9.4]

### Added

-   Added support for linter arguments.
-   Added deprecated lint.
-   Added invalid syntax lint.

## [0.9.3]

### Added

-   Added inline comments support for variables, enum members and defines.
-   Added default completions (`sizeof`, `voids`, etc).

### Fixed

-   Fixed a potential panic in the resolver (see [#281](https://github.com/Sarrus1/sourcepawn-vscode/issues/281), thanks [Keldra](https://github.com/ddorab)!).

## [0.9.2]

### Added

-   Added rename provider.

### Fixed

-   Fixed incorrect heuristic when inferring the mainPath (Thanks [Suza](https://github.com/Zabaniya001)!).
-   Fixed spcomp linting on macOS and Linux.

## [0.9.1]

### Fixed

-   Fixed incorrect completion triggers.
-   Fixed unresolved variables in methods.
-   Fixed missing completions in methods.

## [0.9.0]

### Added

-   Added spcomp status report.
-   Added automatic mainpath detection when missing.

### Fixed

-   Fixed diagnostics not disappearing.

## [0.8.0]

### Added

-   Added a linter provider.
-   Added constructor completion when using the `new` keyword.
-   Added a document completion provider (type `/*` above a function/method declaration).

### Fixed

-   Fixed constructors appearing in method completions.
-   Fixed constructors being identified as methodmaps references.

## [0.7.0]

### Added

-   Added support for adding/deleting/editing documents outside of the editor (see #13).
-   Added support for refreshing semantic analysis outside of the edited file (see #12).

## [0.6.1]

### Added

-   Added status notifications.
-   Added support for folder rename in includeDirectories.

## [0.6.0]

### Added

-   Added `typedef` and `typeset` support.
-   Added callback completions.
-   Added file rename/deletion support in includeDirectories.

## [0.5.1]

### Added

-   Added notifications when a setting is invalid.

### Fixed

-   Fixed changes in IncludeDirectories not being detected.
-   Fixed some references not being resolved on the initial parse.

## [0.5.0]

### Added

-   Added Document Symbol provider.

## [0.4.0]

### Added

-   Added Reference provider.

### Fixed

-   Fixed invalid file reads when the file contains invalid UTF-8 characters.

## [0.3.1]

### Fixed

-   Fixed panic when opening a file without opening its parent folder first.

## [0.3.0]

### Added

-   Added SignatureHelp provider.

## [0.2.1]

### Fixed

-   Fixed a potential panic when reading an invalid file.

## [0.2.0]

### Added

-   Added Hover, GoToDefinition, and Semantic Highlighting.

## [0.1.0]

### Added

-   Added support for function completions.
