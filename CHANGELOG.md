## Release Notes

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
