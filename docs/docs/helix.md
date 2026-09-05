---
sidebar_position: 5
---

# Helix

## Installation

Download sourcepawn-studio LSP binary and make it accessible in the `$PATH` environment variable.

## Configuration

To use Sourcepawn-studio with Helix editor open `$HOME/.config/helix/languages.toml` and add configuration for Sourcepawn language:

```toml
[[language]]
name = "sourcepawn"
scope = "source.sourcepawn"
injection-regex = "sourcepawn|sp"
file-types = ["sp", "inc"]
indent = { tab-width = 4, unit = "    " }
comment-token = "//"
block-comment-tokens = { start = "/*", end = "*/" }
language-servers = [ "sourcepawn-studio" ]
```

This adds Sourcepawn as recognizable language for Helix and registers `sourcepawn-studio` binary as LSP.

## Grammar highlighting

Add [Sourcepawn Treesitter](https://github.com/nilshelmig/tree-sitter-sourcepawn) for highlighting.

Do this in the same file:

```toml
[[grammar]]
name = "sourcepawn"
source = { git = "https://github.com/nilshelmig/tree-sitter-sourcepawn", rev = "2ef2d389c29b952f8b23909783f7d3c7972046f5" } # Choose revision number as you wish
```

After that do `hx --grammar fetch && hx --grammar build` to fetch and build grammar.

## Checking health

After theese steps check that configuration is correct by typing `hx --health sourcepawn`, healthy result should show you this:

```
Configured language servers:
  ✓ sourcepawn-studio: /home/bittermann/.nix-profile/bin/sourcepawn-studio
Configured debug adapter: None
Configured formatter: None
Tree-sitter parser: ✓
Highlight queries: ✓
Textobject queries: ✓
Indent queries: ✘
```

## Project configuration

To make Sourcepawn-studio find and recognize sourcestudio libraries, create `.helix/languages.toml` in the root of your project directory.

Inside, tell Helix absolute path to the `sourcemod/scripting/include` dir:

```toml
[language-server.sourcepawn-studio.config]
includeDirectories = [ "/home/bittermann/Projects/TF2/tf2-data/tf/addons/sourcemod/scripting/include" ] # Example
```
