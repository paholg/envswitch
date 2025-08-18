es() {
    local env
    env=$(BIN set -szsh "$@") || return $?
    eval "$env"
}

autoload -U compinit && compinit

_es() {
    local context state line
    typeset -A opt_args

    _arguments \
        '(-f --file)'{-f,--file}'[Config File]:file:_files' \
        '(-l --list)'{-l,--list}'[List available environments]' \
        '*::positional:_es_positional'
}

_es_positional() {
    local -a completions

    local -a full_line=(${(z)BUFFER})
    local -a args=("${full_line[@]:1}")

    completions=(${(f)"$(BIN complete ${args[@]} 2>/dev/null)"})

    _describe 'environments' completions
}

compdef '_es' 'es'
