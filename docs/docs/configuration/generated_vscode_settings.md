---
id: vscode-settings-reference
title: VSCode Settings Reference
---

## AuthorName

**sourcepawn.AuthorName**

The name of the plugin's author (you).

_Default_: ``

## GithubName

**sourcepawn.GithubName**

The GitHub username of the plugin's author (you).

_Default_: ``

## MainPathCompilation

**sourcepawn.MainPathCompilation**

Whether the compile button always compiles the MainPath file (true) or the currently opened file (false).

_Default_: `true`

## showCompileIconInEditorTitleMenu

**sourcepawn.showCompileIconInEditorTitleMenu**

Whether to show the 'Compile Code' icon in editor title menu.

_Default_: `true`

## runServerCommandsAfterCompile

**sourcepawn.runServerCommandsAfterCompile**

Run RCON commands after compiling.

_Default_: `"false"`

## serverCommands

**sourcepawn.serverCommands**

A list of commands that will be sent to the server after a successful VSCode command or on `runServerCommands`.

_Default_: `["sm plugins refresh"]`

## uploadToServerAfterCompile

**sourcepawn.uploadToServerAfterCompile**

Upload files to FTP/SFTP after compiling.

_Default_: `false`

## enableLinter

**sourcepawn.enableLinter**

Toggle the linter on or off.

_Default_: `true`

## availableAPIs

**sourcepawn.availableAPIs**

Available Sourcemod APIs to quickly switch between them.

_Default_: `[{"name":"","includeDirectories":[],"compilerPath":"","outputDirectoryPath":"","compilerArguments":[]}]`

## outputDirectoryPath

**sourcepawn.outputDirectoryPath**

The path to the output directory for the compiled .smx file. Ends with a `/`.

_Default_: `""`

## UploadOptions

**sourcepawn.UploadOptions**

Upload options for the FTP/SFTP client.

_Default_: `{"host":"","port":21,"username":"","password":"","sftp":false,"remoteRoot":"/tf/addons/sourcemod","exclude":["scripting/**/",".vscode/**/",".github/**/",".gitignore","*.md",".git"]}`

## SourceServerOptions

**sourcepawn.SourceServerOptions**

Source server details to execute the commands on.

_Default_: `{"host":"","port":27015,"encoding":"ascii","timeout":1000,"password":""}`

## formatterSettings

**sourcepawn.formatterSettings**

Settings for the formatter. Any setting supported by Clang Format can be used here.

_Default_: `["AlignAfterOpenBracket: Align","AlignArrayOfStructures: Left","AlignConsecutiveAssignments: AcrossEmptyLinesAndComments","AlignConsecutiveBitFields: AcrossEmptyLinesAndComments","AlignConsecutiveDeclarations: AcrossEmptyLinesAndComments","AlignConsecutiveMacros: AcrossEmptyLinesAndComments","AlignEscapedNewlines: Left","AlignOperands: AlignAfterOperator","AlignTrailingComments: true","AllowAllArgumentsOnNextLine: true","AllowAllConstructorInitializersOnNextLine: true","AllowAllParametersOfDeclarationOnNextLine: true","AllowShortBlocksOnASingleLine: Always","AllowShortCaseLabelsOnASingleLine: true","AllowShortEnumsOnASingleLine: true","AllowShortFunctionsOnASingleLine: All","AllowShortIfStatementsOnASingleLine: AllIfsAndElse","AllowShortLambdasOnASingleLine: All","AllowShortLoopsOnASingleLine: false","AlwaysBreakAfterDefinitionReturnType: None","AlwaysBreakAfterReturnType: None","AlwaysBreakBeforeMultilineStrings: false","AlwaysBreakTemplateDeclarations: No","BasedOnStyle: Google","BinPackArguments: true","BinPackParameters: true","BreakBeforeBinaryOperators: NonAssignment","BreakBeforeBraces: Custom","BraceWrapping: { AfterCaseLabel: true","AfterClass: true","AfterControlStatement: Always","AfterEnum: true","AfterExternBlock: true","AfterFunction: true","AfterNamespace: true","AfterObjCDeclaration: false","AfterStruct: true","AfterUnion: true","BeforeCatch: true","BeforeElse: true","BeforeLambdaBody: true","BeforeWhile: true","IndentBraces: false","SplitEmptyFunction: false","SplitEmptyNamespace: false","SplitEmptyRecord: false }","BreakBeforeConceptDeclarations: false","BreakBeforeTernaryOperators: true","BreakConstructorInitializers: AfterColon","BreakInheritanceList: AfterComma","BreakStringLiterals: false","ColumnLimit: 0","CompactNamespaces: true","ConstructorInitializerAllOnOneLineOrOnePerLine: true","ConstructorInitializerIndentWidth: ${TabSize}","ContinuationIndentWidth: ${TabSize}","Cpp11BracedListStyle: false","EmptyLineBeforeAccessModifier: LogicalBlock","FixNamespaceComments: true","IncludeBlocks: Preserve","IndentAccessModifiers: false","IndentCaseBlocks: false","IndentCaseLabels: true","IndentExternBlock: Indent","IndentGotoLabels: false","IndentPPDirectives: BeforeHash","IndentRequires: true","IndentWidth: ${TabSize}","IndentWrappedFunctionNames: true","LambdaBodyIndentation: OuterScope","Language: Cpp","MaxEmptyLinesToKeep: 1","NamespaceIndentation: All","ObjCBinPackProtocolList: Always","ObjCBreakBeforeNestedBlockParam: false","ObjCSpaceBeforeProtocolList: false","ReflowComments: true","SortIncludes: Never","SpaceAfterCStyleCast: false","SpaceAfterLogicalNot: false","SpaceBeforeAssignmentOperators: true","SpaceBeforeCaseColon: false","SpaceBeforeCpp11BracedList: false","SpaceBeforeCtorInitializerColon: true","SpaceBeforeInheritanceColon: true","SpaceBeforeParens: ControlStatementsExceptControlMacros","SpaceBeforeRangeBasedForLoopColon: true","SpaceBeforeSquareBrackets: false","SpaceInEmptyBlock: false","SpaceInEmptyParentheses: false","SpacesBeforeTrailingComments: ${TabSize}","SpacesInConditionalStatement: false","SpacesInContainerLiterals: true","SpacesInCStyleCastParentheses: false","SpacesInParentheses: false","SpacesInSquareBrackets: false","Standard: Auto","TabWidth: ${TabSize}","UseTab: ${UseTab}","CommentPragmas: '^#define|#tryinclude'"]`

## trace.server

**sourcepawn.trace.server**

Set the logging level of the SourcePawnLanguageServer.

_Default_: `"info"`

