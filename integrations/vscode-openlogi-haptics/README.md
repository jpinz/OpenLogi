# OpenLogi Haptics for VS Code

Plays an MX Master 4 haptic waveform when a GitHub Copilot agent turn stops or
its session ends.

The extension installs a user-level Copilot hook at
`~/.copilot/hooks/openlogi-haptics.json` (or
`$COPILOT_HOME/hooks/openlogi-haptics.json`). The hook calls OpenLogi's
agent-backed CLI, so it does not compete with OpenLogi for the mouse's HID
channel.

## Requirements

- OpenLogi with the `openlogi haptic` command installed and on `PATH`
- A connected device that exposes HID++ haptic feedback
- VS Code/Copilot hooks enabled for the selected session target

## Build and install

```sh
npm install
npm test
npm run package
code --install-extension openlogi-haptics-0.1.0.vsix
```

Run **OpenLogi Haptics: Enable Copilot Hooks** from the Command Palette, then
use either test command to confirm playback. Settings control the CLI path,
optional device-name filter, attention waveform, and session-end waveform.

## Event behavior

- `agentStop` plays the attention waveform whenever Copilot finishes a turn.
  This is the public event that also covers a turn where the agent stops to ask
  for more input.
- `sessionEnd` plays the completion waveform when the Copilot harness reports
  that the full session terminated.

VS Code does not currently expose a stable extension event that distinguishes
"finished successfully" from "waiting for user input." Mid-turn permission
dialogs are also not observable through the public VS Code extension API, so
the extension deliberately does not scrape Copilot UI state.

The hook executes wherever the selected agent harness runs. Local desktop
sessions can reach the local OpenLogi agent; remote and cloud sessions usually
cannot reach the mouse attached to your desktop.
