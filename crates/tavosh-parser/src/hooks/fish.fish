if not set -q TAVOSH_HOOKED
  set -g TAVOSH_HOOKED 1
  function __tavo_preexec --on-event fish_preexec
    printf '\033]133;C\007'
  end
  function __tavo_postexec --on-event fish_postexec
    printf '\033]133;D;%s\007' $status
  end
  functions -c fish_prompt __tavo_original_prompt
  function fish_prompt
    printf '\033]133;A\007'
    __tavo_original_prompt
    printf '\033]133;B\007'
  end
end
