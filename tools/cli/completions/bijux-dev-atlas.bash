_bijux_dev_atlas_complete() {
  local cur prev
  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  local commands="api observe load audit invariants drift reproduce governance system security docs configs check checks contract registry suites tests ops"

  if [[ ${COMP_CWORD} -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
    return 0
  fi

  if [[ "$prev" == "api" ]]; then
    COMPREPLY=( $(compgen -W "list explain diff verify validate contract" -- "$cur") )
    return 0
  fi
}
complete -F _bijux_dev_atlas_complete bijux-dev-atlas
