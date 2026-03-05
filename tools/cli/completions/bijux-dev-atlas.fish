complete -c bijux-dev-atlas -f
complete -c bijux-dev-atlas -n '__fish_use_subcommand' -a "api observe load ops audit invariants drift reproduce governance system security docs configs"
complete -c bijux-dev-atlas -n '__fish_seen_subcommand_from api' -a "list explain diff verify validate contract"
complete -c bijux-dev-atlas -n '__fish_seen_subcommand_from observe' -a "metrics dashboards logs traces"
