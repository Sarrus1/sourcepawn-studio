## Release Notes

### 1.1O.4
 - Hotfix for commands not working anymore.

### 1.10.3
 - Added syntax highlighting for translations.
 - Added syntax highlighting for gamedata files.
 - Added syntax highlighting for cfg files.
 - Added a fileicon for .cfg files.
 - Added linter and builtin compiler options.

### 1.10.2
 - Added link to documentation in hover help.
 - Fixed a keybinding issue.

### 1.10.1
 - Added file origin as function and enums autocompletion details.
 - Added parent enum name as enum members autocompletions details.
 - Improved the parsing of functions.
 - Improved the hover informations styling.
 - Improving the signature informations styling.
 - Added go to definition and hover help for defines, enum and enum members.

### 1.10.0
 - Fixed a bug where files would not get parsed properly sometimes.
 - Fixed nested includes not parsing.
 - Added a formatter based on clang-format, which the user can (almost) fully customize.
 - Added a way to disable the linter per documents : add `//linter=false` at the top of the document you want to block (Suggested by [Kyle](https://github.com/Kxnrl)).
 - Added a setting for specifying the `main.sp` file in a project with multiple `.sp` files. Please note that all files have to be saved in order for the linter to work if that setting is configured.
 - Added Hover description and help for functions.

### 1.9.2
 - Improved styling of signature helps.
 - Improved include parsing speed and reliability, no longer random guessing.

### 1.9.1
 - Hotfix for relative includes not working if they are .sp files (Pointed out by [Bara](https://github.com/Bara)).

### 1.9.0
 - Added Go-To-Definition for functions and for global variables (in the same file).
 - Added forward parsing.
 - Added better description support.
 - Added better iterative parsing, it is no longer required to save the document for completions to take effect.
 - Improved the overall quality and readability of the code.

### 1.8.4
 - Switched to a client-based extension, removing support for LSP.
 - Switched to an iterative parser, instead of a recursive one, thus fixing Call Stack Overflow errors when parsing large files.
 - Added an option to hide the compile button (Suggested by [NullifidianSF](https://github.com/NullifidianSF)).
 - Added an option to add additional include folders location (Suggested by [Bara](https://github.com/Bara)).
 - Fixed an error where the compiler would not resolve the path correctly (Fixed by [Natanel-Shitrit](https://github.com/Natanel-Shitrit)).

### 1.8.3
 - Fixed an error on Windows when generating files.
 - Fixed a syntax error on Windows for paths in json files.

### 1.8.2
 - Fixed an error where the linter was unable to write the compiled file.

### 1.8.1
 - Fixed a key bind issue.
 - Fixed a linter error for include files.

### 1.8.0
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

### 1.7.1
 - Added full icons support.
 - Fixed potential missing dependencies crash.

### 1.7.0

 - Added multiline function parsing.
 - Improved snippets and added new ones.
 - Added support for simple `//` descriptions above functions.
 - Fixed internal sourcemod functions being parsed.
 - Added beginner friendly include parsing.
 - Fixed descriptions not showing.

### 1.6.0

 - Fixed parsing from include files.
 - Added variables autocompletion.
 - Added a few snippets.

### 1.4.0
 - Add a massive number of new keywords and constants (thanks to [@Obuss](https://github.com/Obuss))

### 1.3.0
 - Fix infinite recursion in parsing child folders in `/include/`
 - Fix parse errors parsing included files that use the old syntax
 - Fix error loading `sourcemod_home` when opening a flat `.sp` file
 - Improve loading of large dependency trees
 - Add a number of new sytax definitions (thanks to [@Technoblazed](https://github.com/Technoblazed))

### 1.0.0
 - Add support for simple autocompletion

### 0.3.0
 - Add support for a variety of enums and constants

### 0.2.0
 - Add support for `#include` and the `FeatureType` and `FeatureStatus` enums
 - Add `Action` as a core type
  
### 0.1.0
Initial release with basic SourcePawn highlighting
