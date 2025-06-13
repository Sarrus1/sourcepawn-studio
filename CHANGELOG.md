## Release Notes

## [0.13.12]

### Fixed

- Fixed crash when hovering over macro defined in function.
- Fixed a potential indexing with an invalid key.
- Fixed a panic due to invalid ranges.

## [0.13.11]

### Fixed

- Mitigated a panic due to an invalid offset.

## [0.13.10]

### Fixed

- Fixed an error when parsing event names.
- Fixed a few offset issues in the preprocessor.

## [0.13.9]

### Fixed

- Fixed files not being read in time.

## [0.13.8]

### Fixed

- Fixed a parsing issue (see #415).

## [0.13.6]

### Fixed

- Fixed an issue when resolving the type of a function parameter.

## [0.13.5]

### Fixed

- Fixed an edge case were the SourceMap would not be properly initialized.
- Fixed completions triggers not properly converting to a server range.

## [0.13.4]

### Fixed

- Fixed diagnostics not being converted to u_range.

## [0.13.3]

### Fixed

- Completely rewrote the source mapping system for better preprocessor integration.
- Fixed generic events not being included (see #411).
- Fixed missing IntelliSense for some invalid syntax (see #406).
- Fixed static expressions not being evaluated (see #412).
- Fixed a bug when parsing arguments of macros (see #413).

## [0.13.2]

### Fixed

- Fixed some unhandled panics.

## [0.13.1]

### Added

- Added a setting to set the maximum amount of projects during cache priming, reducing RAM usage for larger workspaces.
- Fixed some panics.

## [0.13.0]

### Added

- Added an incremental database using the Salsa incremental computation framework.
- Made the preprocessor incremental.
- Added a lot of unit tests.
- Made all the request handlers UnwindSafe to reduce the amount of crashes.

## [0.12.0]

### Fixed

- Fixed incorrect includes completion (see #332 for more details).
- Fixed infinite loops when resolving circular imports (see #342 for more details).
- Fixed macro parsing errors (see #313 and #340).
- Fixed a deadlock issue (see #327 for more details).

## [0.11.2]

### Fixed

- Fixed an include resolution breakage for `<>` type includes.

## [0.11.1]

### Fixed

- Fixed includes not being resolved.

## [0.11.0]

### Added

- Added automatic mainPath detection. The mainPath setting does not exist anymore!
- Added support for Apple Silicon.
- Switched to a path interner for better performance when resolving references (no longer using URIs).
- Added better progress report.

## [0.10.19]

### Added

- Added signatures and documentation for typedef, typeset and callback snippet completions (see #324).

### Fixed

- Fixed nested relative include paths not being resolved by the linter (see #311).
- Fixed GoToDefinition on includes only highlighting partial words (see #323).

## [0.10.18]

### Added

- Added fuller backtrace and more logs.

### Fixed

- Fixed some crashes (see #315, #316, #317, #318).

## [0.10.17]

### Fixed

- Fixed dependencies versions.

## [0.10.16]

### Fixed

- Fixed end of file defines from exiting the preprocessor too early when expanded (see #312).
- Fixed line comments being in the wrong order.

## [0.10.15]

### Added

- Added performance information in the logs.

### Fixed

- Fixed a major performance issue.

## [0.10.14]

### Fixed

- Fixed tree-sitter bugs.
- Fixed `sourcemod.inc` not being included automatically.

### Removed

- Removed deprecated diagnostics in `.inc` files.

## [0.10.13]

### Added

- Added support for arguments overflow in preprocessor macros.
- Added #undef support in preprocessor.
- Added semantic resolution for macros.
- Added a debug request to get the preprocessed text for a document.

### Fixed

- Fixed disable syntax linter not being respected.

## [0.10.12]

### Added

- Added support for stringizing in preprocessor.

### Fixed

- Fixed "receiving on an empty or disconnected channel" bug when stopping the server.

## [0.10.11]

### Fixed

- Fixed a crash when loading files that were not yet saved to disk (see #31).

## [0.10.10]

### Fixed

- Fixed resolving references for too many files (this can improve parsing times by 300%).
- Improved method resolution to use the AST instead of primitive text parsing.
- Removed dangerous unwraps.

## [0.10.9]

### Fixed

- Fixed out of range error when doing semantic analysis.
- Removed dangerous unwraps.

## [0.10.8]

### Fixed

- Fixed reading files if they are not sourcepawn files.

## [0.10.7]

### Fixed

- Fixed huge performance dropoffs when iterating through `.git` folders.
- Fixed incorrect error handling.

## [0.10.6]

### Added

- Added more debugging traces.

### Fixed

- Fixed Sentry hostname data leak.
- Fixed early returns in parser when encountering an error.

## [0.10.5]

### Fixed

- Fixed Sentry instantiation.

## [0.10.4]

### Fixed

- Fixed BOM support.
- Fixed disabled code diagnostics being skipped by the preprocessor.

## [0.10.3]

### Added

- Added optional crash reports telemetry.

## [0.10.2]

### Fixed

- Fixed support for `#tryinclude`.
- Fixed incorrect macro offsetting.
- Fixed unknown tokens failing the preprocessor.
- Fixed incorrect arguments indexing in macro expansion.
- Fixed infinite recursion when resolving includes.
- Removed anyhow errors from logs.

## [0.10.1]

### Fixed

- Allow non define identifiers in macro expansions.
- Fixed trailing macro comment expansion issue.
- Fixed empty preprocessed text on preprocessing failure.
- Fixed default completions in includes completions.
- Fixed early aggressive propagation in providers.
- Fixed some potential unwraps.

## [0.10.0]

### Added

- Added full preprocessor support. The extension will now preprocess the files, by expanding macros and evaluating if conditions.
- Added basic logging and tracing.

## [0.9.10]

### Fixed

- Fixed descriptions not appearing on hover.

## [0.9.9]

### Added

- Added better error reporting to the client.

### Fixed

- Fixed a crash when parsing spcomp's output.
- Fixed the server dying too easily.

## [0.9.8]

### Fixed

- Fixed a crash when the mainPath setting was empty.

## [0.9.7]

### Added

- Added a setting to disable the syntax linter.
- Added support for mainPaths relative to the workspace's root.

### Fixed

- Updated the parser to take into account new syntax elements.

## [0.9.6]

### Added

- Added support for AMXXPawn.

## [0.9.5]

### Added

- Added call hierarchy provider.

## [0.9.4]

### Added

- Added support for linter arguments.
- Added deprecated lint.
- Added invalid syntax lint.

## [0.9.3]

### Added

- Added inline comments support for variables, enum members and defines.
- Added default completions (`sizeof`, `voids`, etc).

### Fixed

- Fixed a potential panic in the resolver (see [#281](https://github.com/Sarrus1/sourcepawn-studio/issues/281), thanks [Keldra](https://github.com/ddorab)!).

## [0.9.2]

### Added

- Added rename provider.

### Fixed

- Fixed incorrect heuristic when inferring the mainPath (Thanks [Suza](https://github.com/Zabaniya001)!).
- Fixed spcomp linting on macOS and Linux.

## [0.9.1]

### Fixed

- Fixed incorrect completion triggers.
- Fixed unresolved variables in methods.
- Fixed missing completions in methods.

## [0.9.0]

### Added

- Added spcomp status report.
- Added automatic mainpath detection when missing.

### Fixed

- Fixed diagnostics not disappearing.

## [0.8.0]

### Added

- Added a linter provider.
- Added constructor completion when using the `new` keyword.
- Added a document completion provider (type `/*` above a function/method declaration).

### Fixed

- Fixed constructors appearing in method completions.
- Fixed constructors being identified as methodmaps references.

## [0.7.0]

### Added

- Added support for adding/deleting/editing documents outside of the editor (see #13).
- Added support for refreshing semantic analysis outside of the edited file (see #12).

## [0.6.1]

### Added

- Added status notifications.
- Added support for folder rename in includesDirectories.

## [0.6.0]

### Added

- Added `typedef` and `typeset` support.
- Added callback completions.
- Added file rename/deletion support in includesDirectories.

## [0.5.1]

### Added

- Added notifications when a setting is invalid.

### Fixed

- Fixed changes in IncludesDirectories not being detected.
- Fixed some references not being resolved on the initial parse.

## [0.5.0]

### Added

- Added Document Symbol provider.

## [0.4.0]

### Added

- Added Reference provider.

### Fixed

- Fixed invalid file reads when the file contains invalid UTF-8 characters.

## [0.3.1]

### Fixed

- Fixed panic when opening a file without opening its parent folder first.

## [0.3.0]

### Added

- Added SignatureHelp provider.

## [0.2.1]

### Fixed

- Fixed a potential panic when reading an invalid file.

## [0.2.0]

### Added

- Added Hover, GoToDefinition, and Semantic Highlighting.

## [0.1.0]

### Added

- Added support for function completions.
