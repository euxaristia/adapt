# bash completion for adapt                               -*- shell-script -*-

_adapt_installable_pkgs() {
    apt-cache --no-generate pkgnames "$1" 2>/dev/null
}

_adapt_complete_installed_pkgs() {
    if declare -F _xfunc >/dev/null 2>&1; then
        _xfunc dpkg _comp_dpkg_installed_packages "$1"
    else
        COMPREPLY=( $(dpkg-query -W -f='${Package}\n' "$1*" 2>/dev/null) )
    fi
}

_adapt() {
    local cur prev words cword
    if declare -F _init_completion >/dev/null 2>&1; then
        _init_completion || return
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    fi

    local commands="update upgrade full-upgrade install remove purge search show autoremove clean list edit-sources"
    local global_opts="--noconfirm -h --help -V --version -S --sync -R --remove -Q --query -s --search -i --info -y --refresh -u --upgrade -c --clean -Rs --recursive"

    local op="" search_modifier="" subcommand="" i w
    for (( i=1; i < cword; i++ )); do
        w="${words[i]}"
        case "$w" in
            --sync) op="${op:-sync}" ;;
            --remove) op="${op:-remove}" ;;
            --query) op="${op:-query}" ;;
            --search) search_modifier=1 ;;
            --) break ;;
            --*) ;;
            -*)
                [[ $w == *S* ]] && op="${op:-sync}"
                [[ $w == *R* ]] && op="${op:-remove}"
                [[ $w == *Q* ]] && op="${op:-query}"
                [[ $w == *s* && $w != *R* ]] && search_modifier=1
                ;;
            *) subcommand="$w"; break ;;
        esac
    done

    if [[ -n $subcommand ]]; then
        case "$subcommand" in
            install|show|search)
                if [[ $cur == -* ]]; then
                    COMPREPLY=( $(compgen -W "--noconfirm -h --help" -- "$cur") )
                else
                    COMPREPLY=( $(_adapt_installable_pkgs "$cur") )
                fi
                ;;
            remove|purge|list)
                if [[ $cur == -* ]]; then
                    COMPREPLY=( $(compgen -W "--noconfirm -h --help" -- "$cur") )
                else
                    _adapt_complete_installed_pkgs "$cur"
                fi
                ;;
            upgrade)
                COMPREPLY=( $(compgen -W "-f --full --noconfirm -h --help" -- "$cur") )
                ;;
            *)
                if [[ $cur == -* ]]; then
                    COMPREPLY=( $(compgen -W "--noconfirm -h --help" -- "$cur") )
                fi
                ;;
        esac
        return
    fi

    if [[ $cur == -* ]]; then
        COMPREPLY=( $(compgen -W "$commands $global_opts" -- "$cur") )
        return
    fi

    if [[ -n $op && -z $search_modifier ]]; then
        case "$op" in
            sync) COMPREPLY=( $(_adapt_installable_pkgs "$cur") ) ;;
            remove|query) _adapt_complete_installed_pkgs "$cur" ;;
        esac
        return
    fi

    COMPREPLY=( $(compgen -W "$commands $global_opts" -- "$cur") )
}

complete -F _adapt adapt
