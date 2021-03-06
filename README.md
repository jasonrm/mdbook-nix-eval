# mdbook-nix-eval

This is a [mdbook](https://crates.io/crates/mdbook) preprocessor designed to evaluate code blocks containing [nix](https://nixos.org/) expressions.

Code blocks with the nix language hint are evaluated and the original expression, and results (or stderr output), are returned to be included in the output document.

    ```nix
    builtins.langVersion
    ```

Conde blocks with filename-like language hint will be evaluated as above, but also written to a per-chapter temp directory where the file can be referenced later.

    ```test-file.nix
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
#eval_args = "--timeout 5"
```

Finally, build your book as normal:

```sh
mdbook path/to/book
```

## Warnings

* If the nix-builder has sandboxing enabled, there *should* be limited access to sensitive info, but... it's probably best to only run trusted expressions.
* Network access is allowed in some (most?) cases by the nix sandbox (where available and enabled), so again only trusted expressions are advised.
* nix-instantiate is run with --strict, which the man page say "Warning: This option can cause non-termination, because lazy data structures can be infinitely large." Without the flag, error messages like `error: cannot convert a thunk to JSON` are much more common, and more importantly, the output is much less helpful.
* Aside from what nix does internally with deterministic outputs, there isn't anything on top of that, so shorter chapters are better if using `mdbook serve` as each block does call out to nix.

## License

MPL. See [LICENSE](LICENSE).
Copyright (c) 2021 Jason R. McNeil <jason@mcneil.dev>
