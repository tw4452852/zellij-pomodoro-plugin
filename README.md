# Pomodoro Clock

This is a Pomodoro Clock implemented as a [Zellij][zellij] plugin.

It shows a Pomodoro time as well as current date time.

[zellij]: https://github.com/zellij-org/zellij

## Usage

Get the `pomodoro-clock.wasm` either building it from source (`cargo build --release --locked`) or downloading [the released binary](https://github.com/tw4452852/zellij-pomodoro-plugin/releases/latest/download/pomodoro-clock.wasm).
Place it in your layout, e.g:

```yaml
- direction: Horizontal
  borderless: true
  split_size:
  Percent: 30
  run:
    plugin:
      path: "file:/path/to/pomodoro-clock.wasm"
      _allow_exec_host_cmd: true # Optional
```

**Note:** If you want a notification when the timer expired, you must specify `_allow_exec_host_cmd: true` and have `notify-send` installed.

### Shortcuts

- `<space>` or `mouse left-click`: Suspend/Resume the timer.
- `r`: Reset the timer.
