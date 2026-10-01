export const HAPTIC_WAVEFORMS = [
  'sharp-state-change',
  'damp-state-change',
  'sharp-collision',
  'damp-collision',
  'subtle-collision',
  'happy-alert',
  'angry-alert',
  'completed',
  'square',
  'wave',
  'firework',
  'mad',
  'knock',
  'jingle',
  'ringing',
  'whisper-collision',
] as const;

export type HapticWaveform = (typeof HAPTIC_WAVEFORMS)[number];

export interface HookSettings {
  readonly command: string;
  readonly device?: string;
  readonly attentionWaveform: HapticWaveform;
  readonly sessionEndWaveform: HapticWaveform;
}

interface CommandHook {
  readonly type: 'command';
  readonly bash: string;
  readonly powershell: string;
  readonly timeoutSec: number;
}

export interface CopilotHookConfig {
  readonly version: 1;
  readonly hooks: {
    readonly agentStop: readonly CommandHook[];
    readonly sessionEnd: readonly CommandHook[];
  };
}

function quoteBash(value: string): string {
  return `'${value.replaceAll("'", "'\"'\"'")}'`;
}

function quotePowerShell(value: string): string {
  return `'${value.replaceAll("'", "''")}'`;
}

function hookCommand(settings: HookSettings, waveform: HapticWaveform): CommandHook {
  const args = ['haptic', waveform];
  const device = settings.device?.trim();
  if (device) {
    args.push('--device', device);
  }

  return {
    type: 'command',
    bash: `${[settings.command, ...args].map(quoteBash).join(' ')} >/dev/null`,
    powershell: `& ${[settings.command, ...args].map(quotePowerShell).join(' ')} | Out-Null`,
    timeoutSec: 5,
  };
}

export function createHookConfig(settings: HookSettings): CopilotHookConfig {
  return {
    version: 1,
    hooks: {
      agentStop: [hookCommand(settings, settings.attentionWaveform)],
      sessionEnd: [hookCommand(settings, settings.sessionEndWaveform)],
    },
  };
}
