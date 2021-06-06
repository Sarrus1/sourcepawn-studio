## Release Notes

## [1.12.4]
### Added
 - Addedd auto-closing `>` to include completions.

### Fixed
 - Partially fixed a highlighting issue where the bitwise operator `&` would be interpreted as a pointer derefencement.
 - Fixed a parsing bug for array declarations separated by a `,`.
 - Fixed incomplete JSDoc completions.
 - Fixed a bug with old-style function declarations parsing.

### Removed
 - Removed linter support for .inc files.

## [1.12.3]
### Fixed
 - Fixed scopped variables not being parsed correctly if below a `for` loop.
 - Fixed newly added includes not being parsed automatically.
 - Fixed formatter overwritting unsaved changes (#44). Thanks to [Adrianilloo](https://github.com/Adrianilloo) for reporting.

## [1.12.2]
### Added
 - Added autocompletion for `#include`.

### Fixed
 - Fixed issue #34 thanks to [BoomShotKapow](https://github.com/BoomShotKapow).
 - Fixed issue #35 related to highlighting glitches.
 - Switched default keybind for `smInsertParameters` from `tab` to `ctrl+shift+i`.
 - Fixed a bug causing control statements to be interpreted as functions by the go-to-definition function parser.
 - Fixed an error when parsing arrays in enum struct.

## [1.12.1]
### Fixed
 - Fixed a missing dependency.

## [1.12.0]
### Added
 - More detailed error messages for the linter, thanks to [ShufflexDD's post](https://forums.alliedmods.net/showthread.php?t=201044).
 - Support for go-to-definition for scopped variable.
 - More `#pragma` snippets.
 - Support for range in enum (struct), enum members, variable and define definitions.
 - New command to set the current file as main.
 - Dev builds are now released automatically.
 - Support for variables completion across multiple .sp files
 - Command to download sourcemod automatically.

### Fixed
 - SM Compile will now always point to the current file.
 - The linter now runs asynchronously, thanks to [CirnoV](https://github.com/CirnoV).
 - Fixed an issue where the signature of a function would not reappear when typing a comma.
 - Fixed an issue where function definitions could collide with other definitions.
 - Fixed the linter's regex.
 - Fixed enum parsing regex.
 - Fixed a bug that occured when parsing arrays.
 - Fixed a highlighting bug for numeric constants in arrays' size declarations.
 - Fixed a highlighting bug for turnary operators, thanks to [Холя](https://github.com/HolyHender) for reporting this.
 - Fixed crashes on extension startup when no folder were opened.

## [1.11.5]
### Added
 - Added support for range in function definitions.
 - Added hover support for enum (struct) and properties.
 - Dropped support for semantic syntax highlighting as it was too unreliable.
 - Added support for better token bases syntax highlighting (types like `JSONObject` are now highlighted correctly).
 - Fixed a parsing bug where functions definitions would get overwritten.
 - Fixed syntax highlighting for old style declarations.
 - Fixed typeset being parsed as functions, causing problems with `int` pointing to a definition.

## [1.11.4]
 - Added a new command to automatically insert function parameters (thanks to [BoomShotKapow](https://github.com/BoomShotKapow) for the implementation !)
 - Fixed an issue where multiline function arguments would appear twice (#30).
 - Fixed an issue where two functions (usually natives) declared on two consecutive lines would not be parsed properly.
 - Fixed an issue where a function overcall (like `OnPluginStart`) would not always override the inc definition.
 - Implemented semantic syntax highlighting.
 - Static syntax highlighting tweaks.

## [1.11.3]
 - Fixed a major bug that would cause infinite parsing and crash the extension after a while, thanks a lot to Adrián, sze and JustSad for helping me to fix it!

## [1.11.2]
 - Fixed a bug where enums without a space after the name wouldn't be parsed.
 - Added an include guard to avoid parsing the same files multiple times.
 - Parsing performances improvements.
 - Fixed a bug where a line number would sometimes be negative, causing an extension crash.
 - Added some debugging messages for contributors.

## [1.11.1]
 - Better support for nested includes.
 - Added main file includes completions in secondary .sp files.
 - Fixed a bug where only one letter of the method name was parsed for the autocompletion.
 - Added an error message when opening a .sp file only.

## [1.11.0]
 - Added automatic documentation generation for functions.
 - Added a command to refresh plugins on a Source server.
 - Added a command to upload plugins to an FTP/SFTP server.
 - Added a setting to automatically deploy the plugin to an FTP/SFTP server after a successful compile.
 - Added a setting to automatically refresh the Source server's plugins list after a successful upload.
 - Added basic support for properties autocompletion.
 - Added support for functions without a `public/stock/static/native` prefix.
 - Added support for enum's member documentation on hover.
 - Fixed wrong name parsing or old syntax functions.
 - Methods will now show their parent Methodmap as detail in the autocompletion prompt.
 - Improved the parsing of already parsed includes.
 - Changed the name of the settings prefix for better consistency.
 - The `sourcepawn.MainPath` setting can now be relative or absolute.
 - Fixed the linter on Windows.
 - Sub .sp files now inherit from MainPath's completions.

## [1.10.6]
 - Added a config to toggle the linter on and off.
 - The `MainPath` setting now applies to the compile command as well.
 - Unit tests have been implemented for better reliability.
 - The code has been cleaned up.

## [1.10.5]
 - Second hotfix for commands not working anymore.

## [1.10.4]
 - Hotfix for commands not working anymore.

## [1.10.3]
 - Added syntax highlighting for translations.
 - Added syntax highlighting for gamedata files.
 - Added syntax highlighting for cfg files.
 - Added a fileicon for .cfg files.
 - Added linter and builtin compiler options.

## [1.10.2]
 - Added link to documentation in hover help.
 - Fixed a keybinding issue.

## [1.10.1]
 - Added file origin as function and enums autocompletion details.
 - Added parent enum name as enum members autocompletions details.
 - Improved the parsing of functions.
 - Improved the hover informations styling.
 - Improving the signature informations styling.
 - Added go to definition and hover help for defines, enum and enum members.

## [1.10.0]
 - Fixed a bug where files would not get parsed properly sometimes.
 - Fixed nested includes not parsing.
 - Added a formatter based on clang-format, which the user can (almost) fully customize.
 - Added a way to disable the linter per documents : add `//linter=false` at the top of the document you want to block (Suggested by [Kyle](https://github.com/Kxnrl)).
 - Added a setting for specifying the `main.sp` file in a project with multiple `.sp` files. Please note that all files have to be saved in order for the linter to work if that setting is configured.
 - Added Hover description and help for functions.

## [1.9.2]
 - Improved styling of signature helps.
 - Improved include parsing speed and reliability, no longer random guessing.

## [1.9.1]
 - Hotfix for relative includes not working if they are .sp files (Pointed out by [Bara](https://github.com/Bara)).

## [1.9.0]
 - Added Go-To-Definition for functions and for global variables (in the same file).
 - Added forward parsing.
 - Added better description support.
 - Added better iterative parsing, it is no longer required to save the document for completions to take effect.
 - Improved the overall quality and readability of the code.

## [1.8.4]
 - Switched to a client-based extension, removing support for LSP.
 - Switched to an iterative parser, instead of a recursive one, thus fixing Call Stack Overflow errors when parsing large files.
 - Added an option to hide the compile button (Suggested by [NullifidianSF](https://github.com/NullifidianSF)).
 - Added an option to add additional include folders location (Suggested by [Bara](https://github.com/Bara)).
 - Fixed an error where the compiler would not resolve the path correctly (Fixed by [Natanel-Shitrit](https://github.com/Natanel-Shitrit)).

## [1.8.3]
 - Fixed an error on Windows when generating files.
 - Fixed a syntax error on Windows for paths in json files.

## [1.8.2]
 - Fixed an error where the linter was unable to write the compiled file.

## [1.8.1]
 - Fixed a key bind issue.
 - Fixed a linter error for include files.

## [1.8.0]
 - Added support for for loops variable completion.
 - Added support for enums parsing.
 - Added a command for tasks.json generation from a template.
 - Added a command for .sp file generation from a template.
 - Added a command for README.md file generation from a template.
 - Added a command for github Actions file generation from a template.
 - Added a command to generate a Project from a template.
 - Added a linter based on spcomp.
 - Added a command and a button to compile the current .sp file.
 - Added more settings to support the above features.

## [1.7.1]
 - Added full icons support.
 - Fixed potential missing dependencies crash.

## [1.7.0]

 - Added multiline function parsing.
 - Improved snippets and added new ones.
 - Added support for simple `//` descriptions above functions.
 - Fixed internal sourcemod functions being parsed.
 - Added beginner friendly include parsing.
 - Fixed descriptions not showing.

## [1.6.0]

 - Fixed parsing from include files.
 - Added variables autocompletion.
 - Added a few snippets.

## [1.4.0]
 - Add a massive number of new keywords and constants (thanks to [@Obuss](https://github.com/Obuss))

## [1.3.0]
 - Fix infinite recursion in parsing child folders in `/include/`
 - Fix parse errors parsing included files that use the old syntax
 - Fix error loading `sourcemod_home` when opening a flat `.sp` file
 - Improve loading of large dependency trees
 - Add a number of new sytax definitions (thanks to [@Technoblazed](https://github.com/Technoblazed))

## [1.0.0]
 - Add support for simple autocompletion

## [0.3.0]
 - Add support for a variety of enums and constants

## [0.2.0]
 - Add support for `#include` and the `FeatureType` and `FeatureStatus` enums
 - Add `Action` as a core type
  
## [0.1.0]
Initial release with basic SourcePawn highlighting
