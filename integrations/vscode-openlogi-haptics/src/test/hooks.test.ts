import assert from 'node:assert/strict';
import test from 'node:test';

import { createHookConfig } from '../hooks.js';

test('creates attention and session-end Copilot hooks', () => {
  const config = createHookConfig({
    command: 'openlogi',
    attentionWaveform: 'happy-alert',
    sessionEndWaveform: 'completed',
  });

  assert.equal(config.version, 1);
  assert.match(config.hooks.agentStop[0].bash, /'openlogi' 'haptic' 'happy-alert'/);
  assert.match(config.hooks.sessionEnd[0].bash, /'completed'/);
  assert.equal(config.hooks.agentStop[0].timeoutSec, 5);
});

test('quotes executable paths and device filters for both shells', () => {
  const config = createHookConfig({
    command: "/Applications/Open'Logi/openlogi",
    device: "Desk's MX Master 4",
    attentionWaveform: 'knock',
    sessionEndWaveform: 'completed',
  });
  const hook = config.hooks.agentStop[0];

  assert.equal(
    hook.bash,
    `'/Applications/Open'"'"'Logi/openlogi' 'haptic' 'knock' '--device' 'Desk'"'"'s MX Master 4' >/dev/null`,
  );
  assert.equal(
    hook.powershell,
    `& '/Applications/Open''Logi/openlogi' 'haptic' 'knock' '--device' 'Desk''s MX Master 4' | Out-Null`,
  );
});
