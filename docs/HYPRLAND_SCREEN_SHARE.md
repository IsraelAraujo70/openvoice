# Hyprland Screen Share

This branch makes OpenVoice windows expose stable Linux application IDs so Hyprland rules can
match them reliably.

OpenVoice now also tries to call `hyprctl keyword windowrule 'match:class ..., no_screen_share on'`
automatically when each window opens.

## Window classes

By default, the app now opens windows with these IDs:

- `openvoice-main`
- `openvoice-subtitle`
- `openvoice-copilot-overlay`
- `openvoice-copilot-response`

If you set `OPENVOICE_APPLICATION_ID_PREFIX`, the prefix changes and the suffixes stay the same.

Examples:

- `OPENVOICE_APPLICATION_ID_PREFIX=openvoice-dev` -> `openvoice-dev-main`
- `OPENVOICE_APPLICATION_ID_PREFIX=openvoice-dev` -> `openvoice-dev-subtitle`

## How to inspect in Hyprland

Use:

```bash
hyprctl clients -j | jq '.[] | {class, initialClass, title, initialTitle}'
```

OpenVoice should show the IDs above in `class` / `initialClass`.

## Example rules

If automatic rule injection fails on your setup, keep one of these rules as a fallback.

To hide every OpenVoice window from screen sharing:

```ini
windowrule {
  name = openvoice-hide-all
  match:class = ^(openvoice-(main|subtitle|copilot-overlay|copilot-response))$

  no_screen_share = on
}
```

To hide only the lightweight HUD / subtitle surfaces:

```ini
windowrule {
  name = openvoice-hide-hud
  match:class = ^(openvoice-main|openvoice-subtitle)$

  no_screen_share = on
}
```

To hide only the copilot overlays:

```ini
windowrule {
  name = openvoice-hide-copilot
  match:class = ^(openvoice-(copilot-overlay|copilot-response))$

  no_screen_share = on
}
```

## Notes

- Hyprland window rules are case-sensitive.
- On Hyprland, `no_screen_share` is compositor behavior, not an `Iced` feature.
