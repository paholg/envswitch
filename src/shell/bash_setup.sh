es() {
    local env
    env=$(BIN set -sbash "$@") || return $?
    eval "$env"
}

_es_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Handle flag arguments
    case "${prev}" in
        -f|--file)
            # File completion for -f/--file
            mapfile -t COMPREPLY < <(compgen -f -- "${cur}")
            return 0
            ;;
    esac

    # Handle flags
    if [[ ${cur} == -* ]]; then
        opts="-f --file -l --list"
        mapfile -t COMPREPLY < <(compgen -W "${opts}" -- "${cur}")
        return 0
    fi

    # Handle dynamic env argument
    local args=("${COMP_WORDS[@]:1}")
    local completions
    mapfile -t completions < <(BIN complete "${args[@]}" 2>/dev/null)
    mapfile -t COMPREPLY < <(compgen -W "${completions[*]}" -- "${cur}")
}
