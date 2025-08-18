[![version](https://img.shields.io/github/v/release/paholg/envswitch)](https://github.com/paholg/envswitch/releases/)

# envswitch

A simple tool to manage multiple sets of environment variables.

## Quick example

Given this config file:

```toml
[local]
URL = "http://localhost:3000"
KEY = "test_key_123"

[prod]
URL = "https://example.com"

[prod.abc]
KEY = "super_secret"
```

Running `es local` will set:

```bash
URL=http://localhost:3000
KEY=test_key_123
```

Running `es prod.abc` will set:

```bash
URL=https://example.com
KEY=super_secret
```

Running `es -l` will show the options:
```
Available environments:
  local
  prod
  prod.abc
```

See [Usage](#Usage) for more detailed examples.

## Compatibility

Currently, envswitch is compatible with these shells:
* bash
* fish
* zsh

If you'd like to use envswitch with another shell, it should be quite easy to
add; please open a ticket or a PR. See [src/shell.rs](src/shell.rs).

## Installation

Thanks to [dist](https://github.com/axodotdev/cargo-dist), installation on
various platforms is easy.

| Platform | Instructions |
|-------|-----------------|
| MacOs | Run `brew install paholg/tap/envswitch` |
| Nix   | Add this repo as a flake input |
| Other | See [releases](https://github.com/paholg/envswitch/releases/) |

## Configuration

It is strongly recommended that you run `envswitch` through a shell function, as
otherwise it just outputs shell commands that need to be sourced.

Please place the appropriate snippet in your shell config. It will register the
function `es`; if you'd prefer another name, you can run the `setup` command
yourself and copy the output to your config with any desired changed.

### Bash

```bash
source <(envswitch setup bash)
```

### Fish
```fish
envswitch setup fish | source
```

### Zsh

```zsh
source <(envswitch setup zsh)
```

---

The prior functions will look for the file `envswitch.toml` in the directoy you
call them in. If you'd prefer a different file, or perhaps to set an alias with
a fixed file location, you can do so with the `--file/-f` flag.

For example, you might want to set an alias like this:

```bash
alias esh="es -f ~/.envswitch.toml"
```

You can see all options with `envswitch --help`.

## Usage

The `envswitch.toml` file should be thought of as a tree; `envswitch` will walk
it to your given path, overriding any more general settings.

Given this example file:

```toml
GLOBAL = "some global variable"

[staging]
URL = "staging.com"

[staging.abc]
KEY = "secret_ABC"

[staging.def]
KEY = "secret_DEF"
URL = "def.staging.com"

[prod]
GLOBAL = "override for production"
URL = "prod.com"

[prod.abc]
KEY = "prod_secret_ABC"
```

Here are some commands, and what variables they will cause to be set:

**`es staging`**
```bash
GLOBAL="some global variable"
URL="staging.com"
```

**`es staging.abc`**
```bash
GLOBAL="some global variable"
URL="staging.com"
KEY="secret_ABC"
```

**`es staging.def`**
```bash
GLOBAL="some global variable"
# There is a more specific URL here, so we use that.
URL="def.staging.com"
KEY="secret_DEF"
```

**`es prod.abc`**
```bash
GLOBAL="override for production"
URL="prod.com"
KEY="prod_secret_ABC"
```

**`es prod`**
```bash
GLOBAL="override for production"
URL="prod.com"
# NOTE: `KEY` will be unset here if it was previously set by envswitch.
```

**`es`**
```bash
GLOBAL="some global variable"
# NOTE: Running with no arguments will cause any non-global variables that were
# set by envswitch to be unset.
```

You can also run `es -l` to see available environments.

## Integrations

Running `envswitch get` will print the name of the environment we are currently
in, such as `staging.def`.

This can be used to show the current environment in your prompt.

### Starship

If you use [starship](https://starship.rs/), the `envswitch` environment can be
easily added as a [custom command](https://starship.rs/config/#custom-commands).
Here is an example that can be added to `starship.toml`:

```toml
[custom.envswitch]
description = "Show which envswitch environment is currently active."
command = "envswitch get"
style = "yellow"
when = true
format = "[($symbol $output )]($style)"
symbol = ""
```

### How it Works

When you run an `envswitch set` command, it outputs commands to set or unset
variables, which are then sourced by your shell in the function setup in
[Configuration](#configuration).

It also sets a special variable to let it track what it has set before. This
enables `envswitch` to unset variables it has previously set even if you edit
the config file or pass it a different one, such as by moving to another
directory.

Perhaps this is most clear with some examples. Using the config file from
[Usage](#usage),

```bash
$ envswitch set -sbash staging
export ENVSWITCH_ENV="staging:GLOBAL,URL"
export GLOBAL="some global variable"
export URL="staging.com"
Environment set: staging GLOBAL URL
```

The 3 export lines are piped to stdout, whereas the last line is sent to stderr
so that it is not captured by the pipe to `source`.

The `ENVSWITCH_ENV` variable tells us the name of the environment we're in
(which is used by `envswitch get`) and which variables we have set.

So, when we run another `set` command, it can unset them:

```bash
$ envswitch get
staging

$ envswitch set -sbash prod.abc
unset GLOBAL
unset URL
export ENVSWITCH_ENV="prod.abc:GLOBAL,URL,KEY"
export GLOBAL="override for production"
export URL="prod.com"
export KEY="prod_secret_ABC"
Environment set: prod.abc GLOBAL URL KEY
```
