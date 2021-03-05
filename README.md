# mdbook-nix-eval

This is a simple [mdbook](https://crates.io/crates/mdbook) preprocessor designed to write and evaluate code blocks containing [nix](https://nixos.org/) expressions to files.

    ```test-file.nix
    builtins.langVersion
    ```

Code blocks with nix expressions are evaluated.

    ```nix
    builtins.langVersion
    ```

## Installation

If you want to use only this preprocessor, install the tool:

```sh
cargo install mdbook-nix-eval
```

Add it as a preprocessor to your `book.toml`:

```toml
[preprocessor.nix-eval]
command = "mdbook-nix-eval"
renderer = ["html"]
```

Finally, build your book as normal:

```sh
mdbook path/to/book
```

## Warnings

* If the nix-builder has sandboxing enabled, there *should* be limited access to sensitive info, but... it's probably best to only run trusted expressions.
* Network access is allowed in some (most?) cases by the nix sandbox (where available and enabled), so again only trusted expressions are advised.

## License

MPL. See [LICENSE](LICENSE).  
Copyright (c) 2021 Jason R. McNeil <jason@mcneil.dev>