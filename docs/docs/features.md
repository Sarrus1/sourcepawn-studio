---
sidebar_position: 4
---

# Features

## Completions

When editing a file, the Language Server will suggest different completion suggestion based on the cursor's position and the infered surrounding context.

### Include completions

When writing an include statement, the Language Server will suggest available files and folders depending on what has already been typed. The results are different based on the include type (relative or absolute).

<div align="center">
![include completions example animation](./features_img/include-completion-example-1.gif)
</div>

### Callback completions

Starting to type the name of a forward or a typedef, typeset, functag or a funcenum will suggest a list of snippet completions, which, when triggered, will insert a callback declaration for the corresponding elelement.

<div align="center">
![callback completions example animation](./features_img/callback-completion-example-1.gif)
</div>

### Regular completions

Regular completions will suggest previously declared functions, variables, defines, etc. When writing a method or property access, only the relevant items will be suggested.

<div align="center">
![regular completions example animation](./features_img/regular-completion-example-1.gif)
</div>
