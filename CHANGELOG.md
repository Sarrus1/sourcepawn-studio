## Release Notes

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
