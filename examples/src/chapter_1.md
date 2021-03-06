# mdbook-nix-eval

This is a [mdbook](https://crates.io/crates/mdbook) preprocessor designed to evaluate code blocks containing [nix](https://nixos.org/) expressions.

    ```test-file.nix
    builtins.langVersion
    ```

code blocks with nix expressions are evaluated using [nix](https://nixos.org/). 

    ```nix
    builtins.langVersion
    ```

## Simple Evaluation

```nix
builtins.langVersion
```

## Importing Files 

Code blocks with filenames ending in ".nix" will be put into a temporary directory for each chapter.

```langVersion.nix
builtins.langVersion
```

As the expression is written to a known file name, it's possible to import as usual.

```nix
let
    version = import ./langVersion.nix;
in  
"Nix Langauge Version: ${toString version}"
```

With evaluation errors passed through

```nix
let
    version = import ./langVersion.nix;
in
"Nix Langauge Version: ${version}"
```

Expressions can be functions

```nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.path
```

However, they should have defaults, otherwise they won't return an evaluation that makes sense to display.

```without-defaults.nix
{ pkgs }:
pkgs.path
```

Default-less function arg files can still be referenced later without issue

```nix
import ./without-defaults.nix { pkgs = import <nixpkgs> {}; }
```

## More Examples

If the nix-builder has sandboxing enabled, there *should* be limited access to sensitive info, but... it's probably best to only run trusted expressions.

```nix
let
run = (with import <nixpkgs> {}; runCommand "foo" {} "ls -l ~/.ssh > $out");
in
builtins.readFile run
```

Network access is allowed in some (most?) cases, so again trusted expressions only are advised.

```nix
{ pkgs ? import <nixpkgs> {} }:
let
  gitignore = builtins.fetchurl {
    url = "https://www.toptal.com/developers/gitignore/api/jetbrains,linux,macos,git";
    name = "gitignore";
    sha256 = "0fn3632fdz5rbvbkwnn82q6qsdsq2haxc7mlbm536g69zlr41c1z";
  };
in
{
    out = builtins.elemAt (pkgs.lib.splitString "\n" (builtins.readFile gitignore)) 1;
}
```

```nix
builtins.readFile (with import <nixpkgs> {}; runCommand "foo" {} "date > $out; ${fping}/bin/fping -c5 1.1.1.1 >> $out")
```

## Remote Build Machines

If you have remote building enabled,

```systems.nix
{
 linux = (with import <nixpkgs> { system = "x86_64-linux"; }; runCommand "foo" {} "uname > $out");
 darwin = (with import <nixpkgs> { system = "x86_64-darwin"; }; runCommand "foo" {} "uname > $out");
}
```

```nix
builtins.readFile (import ./systems.nix).linux
```

```nix
builtins.readFile (import ./systems.nix).darwin
```

## Supported output types

### null

```nix
null
```

### string

```nix
"this is a string"
```

(note: the formatting is slightly modified to better show multiline)

```nix
"this is\na multiline string\nnote that trailing whitespace is trimmed\n"
```

### numeric

```nix
12345
```

### lists (arrays)

```nix
[1234 6789]
```

### sets (objects)

```nix
{first = 1234; second = 6789;}
```

### functions? nope

```nix
builtins.readFile
```
