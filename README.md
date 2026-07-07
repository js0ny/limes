# `limes` Linux Input MEthod Switcher

`limes` is a D-Bus wrapper for switching Linux input methods.

It provides a small `get|set|list|toggle` CLI over multiple input method
backends.

Supported backends: `fcitx5`, `fcitx5-rime`, `ibus`.

> [!WARNING]
> The code of this project is mainly LLM Generated, with basic manual audit, use it with caution.

## Usage

```bash
limes get # if no verb given, the default action is get
limes list
limes set rime
limes toggle
```

The default backend is currently `fcitx5`.

Select a backend explicitly with `--backend`:

```bash
limes --backend fcitx5 get
limes --backend ibus list
limes --backend ibus set libpinyin
limes --backend ibus set xkb:us::eng
```

Generate shell completions:

```bash
limes completion <SHELL>
```

## Backends

### `fcitx5`

Switches Fcitx5 input methods through the Fcitx5 controller D-Bus API.

```bash
limes --backend fcitx5 get
limes --backend fcitx5 list
limes --backend fcitx5 set rime
limes --backend fcitx5 toggle
```

### `fcitx5-rime`

Talks to Fcitx5's Rime D-Bus API.

By default, `fcitx5-rime` operates on Rime ascii mode:

```bash
limes --backend fcitx5-rime get        # prints true or false
limes --backend fcitx5-rime set --ascii true # the flag --ascii is optional
limes --backend fcitx5-rime set false
limes --backend fcitx5-rime toggle
```

Use `--mode schema` to operate on Rime schemas:

```bash
limes --backend fcitx5-rime --mode schema get
limes --backend fcitx5-rime --mode schema list
limes --backend fcitx5-rime --mode schema set 
```

Schema mode does not support `toggle`.

### `ibus`

Switches IBus global engines through IBus' private D-Bus connection.

```bash
limes --backend ibus get
limes --backend ibus list
limes --backend ibus set libpinyin
limes --backend ibus set xkb:us::eng
```

IBus does not support `toggle`. `--mode` is only supported by
`fcitx5-rime`.

## Build

```bash
cargo build --release
```

With Nix:

```bash
nix build
```

## TODOs

- [ ] Config File, set default backend
- [ ] `--init-config`, auto discover backend on first run
- [ ] Defining `toggle` ime groups in config file
