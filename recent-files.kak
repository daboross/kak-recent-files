# Kak Recent Files WIP plugin

declare-option -docstring "whether or not recent files recordings should be saved to disk" \
    bool krf_use_temp_storage false

declare-option -hidden -docstring "temp storage for krf if saving to disk is not enabled" \
    str krf_temp_storage

declare-option -docstring "command to use when opening menu" \
    str krf_menu_command "rofi -dmenu -i -matching fuzzy"

define-command -hidden krf-add-file %{
    evaluate-commands %sh{
        kak-recent-files --session "$kak_session" "$kak_buffile"
    }
}

hook global KakBegin .* -group krf-global-hooks %{
    evaluate-commands %sh{
        if echo "$kak_session" | grep -qE '^[0-9]+$'; then
            echo "set-option global krf_use_temp_storage true"
        else
            echo "set-option global krf_use_temp_storage false"
            kak-recent-files --session "$kak_session" --use-temp false open-initial-file
        fi
    }
}

hook global WinDisplay .* -group krf-global-hooks %{
    evaluate-commands %sh{
        kak-recent-files --session "$kak_session" \
            --use-temp "$kak_opt_krf_use_temp_storage" \
            --temp-storage "$kak_opt_krf_temp_storage" \
            opened-file "$kak_buffile"
    }
}

define-command krf-open-menu -docstring "open menu for opening recent files" %{
    evaluate-commands %sh{
        kak-recent-files --session "$kak_session" \
            --use-temp "$kak_opt_krf_use_temp_storage" \
            --temp-storage "$kak_opt_krf_temp_storage" \
            open-menu --from "$kak_buffile" \
                --cmd "$kak_opt_krf_menu_command"
    }
}

define-command krf-delete-file -docstring "remove a file from recent files for this session" \
        -buffer-completion -params 1 %{
    evaluate-commands %sh{
        kak-recent-files --session "$kak_session" \
            --use-temp "$kak_opt_krf_use_temp_storage" \
            --temp-storage "$kak_opt_krf_temp_storage" \
            remove-file "$0"
    }
}

define-command krf-reset -docstring "resets the recent files list for this session" %{
    evaluate-commands %sh{
        kak-recent-files --session "$kak_session" \
            --use-temp "$kak_opt_krf_use_temp_storage" \
            --temp-storage "$kak_opt_krf_temp_storage" \
            reset
    }
}
