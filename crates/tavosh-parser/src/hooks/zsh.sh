if [ -z "${TAVOSH_HOOKED-}" ]; then
  TAVOSH_HOOKED=1
  autoload -Uz add-zsh-hook
  __tavo_precmd() { printf '\033]133;D;%s\007\033]133;A\007' "$?"; }
  __tavo_preexec() { printf '\033]133;C\007'; }
  add-zsh-hook precmd __tavo_precmd
  add-zsh-hook preexec __tavo_preexec
  PROMPT="${PROMPT}"$'%{\e]133;B\a%}'
fi
