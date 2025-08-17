# envswitch

A simple tool to manage multiple sets of environment variables.

Quick example:

Given:

```toml
[local]
URL = "http://localhost:3000"
KEY = "test_key_123"

[prod]
URL = "https://example.com"

[prod.abc]
KEY = "super_secret"
```

Then running `es local` will set `URL=http://localhost:3000` and
`KEY=test_key_123`, whereas running `es prod.abc` will set
`URL=https://example.com` and `KEY=super_secret`.

See [Usage](#Usage) for more detailed examples.

## Compatibility

Currently, envswitch is compatible with bash, zsh, and fish shells.

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

The following functions will look for the file `envswitch.toml` in the directory
that you call them from. If you'd prefer alterntive behavior, such as a set
location, pass in the `--file` flag. You can see all options with
`envswitch --help`.

Please place the appropriate line in your shell config:

### Bash and Zsh
```bash
es() { source <(envswitch -sbash "$@"); }
```

### Fish
```fish
function es; envswitch -sfish $argv | source; end
```

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

## Integrations

Running just `envswitch` (and not one of our shell functions) will
print the name of the environment we are currently in, such as `staging.def`.

This can be used to show the current environment in your prompt.

### Starship

If you use [starship](https://starship.rs/), the `envswitch` environment can be
easily added as a [custom command](https://starship.rs/config/#custom-commands).
Here is an example that can be added to `starship.toml`:

```toml
[custom.envswitch]
description = "Show which envswitch environment is currently active."
command = "envswitch"
style = "yellow"
when = true
format = "[($symbol $output )]($style)"
symbol = ""
```
