function es
    BIN set -sfish $argv | source
    return $pipestatus[1]
end

complete -c es -e
complete -c es -s f -l file -d "Config file" -r -F
complete -c es -s l -l list -d "List available environments"

function __es_complete_positional
    BIN complete (commandline -opc)[2..] 2>/dev/null
end

complete -c es -f -a '(__es_complete_positional)'
