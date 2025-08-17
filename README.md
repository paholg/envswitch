# envswitch

A simple tool to manage multiple sets of environment variables.

## Compatibility

Currently, envswitch is compatible with bash, zsh, and fish shells.

If you'd like to use envswitch with another shell, it should be quite easy to
add; please open a ticket or a PR. See [src/shell.rs](src/shell.rs).

## Installation

Thanks to [dist](https://github.com/axodotdev/cargo-dist), installation on
various platforms is easy.

| Platform | Instructions |
|-------|-----------------|
| MacOs | Run `brew install envswitch` |
| Nix   | Add this repo as a flake input |
| Other | See [releases](https://github.com/paholg/envswitch/releases/) |

## Configuration

It is strongly recommended that you run `envswitch` through a shell alias, as
otherwise it just outputs shell commands that need to be sourced.

The following aliases will look for the file `envswitch.toml` in the directory
that you call them from. If you'd prefer alterntive behavior, such as a set
location, pass in the `--file` flag.

### Bash and Zsh
```bash
es() { source <(envswitch -sbash "$@"); }
```

### Fish
```fish
function es; envswitch -sfish $argv | source; end
```

## Usage

The `envswitch.toml` should be thought of as a tree; `envswitch` will walk it
to your given path, overriding any more general settings.

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

**`envswitch`**

Finally, running just `envswitch` (and not one of our shell functions) will
print the name of the environment we are currently in, such as `staging.def`.

## Integrations

### Starship
