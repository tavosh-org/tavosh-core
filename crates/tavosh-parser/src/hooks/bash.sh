# Sourced only in interactive bash. Preserve existing prompt hooks and traps.
if [ -z "${TAVOSH_HOOKED-}" ]; then
  TAVOSH_HOOKED=1
  __tavo_prompt() { local s=$?; printf '\033]133;D;%s\007\033]133;A\007' "$s"; }
  __tavo_debug() { case "$BASH_COMMAND" in __tavo*|"${PROMPT_COMMAND-}") return;; esac; if [ "${__tavo_ready-}" = 1 ]; then printf '\033]133;C\007'; __tavo_ready=0; fi; }
  PROMPT_COMMAND="__tavo_prompt${PROMPT_COMMAND:+; $PROMPT_COMMAND}; __tavo_ready=1"
  PS1="${PS1}"'\[\e]133;B\a\]'
  # Do not replace a user DEBUG trap. Heuristics cover shells with one installed.
  [ -n "$(trap -p DEBUG)" ] || trap '__tavo_debug' DEBUG
fi
