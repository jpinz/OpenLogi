import { execFile } from 'node:child_process';
import { promises as fs } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { promisify } from 'node:util';

import * as vscode from 'vscode';

import {
  HAPTIC_WAVEFORMS,
  type HapticWaveform,
  type HookSettings,
  createHookConfig,
} from './hooks.js';

const execFileAsync = promisify(execFile);
const ENABLED_KEY = 'openlogiHaptics.enabled';
const HOOK_FILENAME = 'openlogi-haptics.json';

function configuredSettings(): HookSettings {
  const config = vscode.workspace.getConfiguration('openlogiHaptics');
  const command = config.get<string>('command', 'openlogi').trim();
  if (!command) {
    throw new Error('openlogiHaptics.command must not be empty');
  }

  return {
    command,
    device: config.get<string>('device', '').trim() || undefined,
    attentionWaveform: waveformSetting(config, 'attentionWaveform', 'happy-alert'),
    sessionEndWaveform: waveformSetting(config, 'sessionEndWaveform', 'completed'),
  };
}

function waveformSetting(
  config: vscode.WorkspaceConfiguration,
  key: string,
  fallback: HapticWaveform,
): HapticWaveform {
  const value = config.get<string>(key, fallback);
  if (HAPTIC_WAVEFORMS.includes(value as HapticWaveform)) {
    return value as HapticWaveform;
  }
  throw new Error(`openlogiHaptics.${key} has an unknown waveform: ${value}`);
}

function hookPath(): string {
  const configuredHome = process.env.COPILOT_HOME?.trim();
  const copilotHome = configuredHome || path.join(os.homedir(), '.copilot');
  return path.join(copilotHome, 'hooks', HOOK_FILENAME);
}

async function writeHookFile(): Promise<string> {
  const target = hookPath();
  await fs.mkdir(path.dirname(target), { recursive: true });
  const content = `${JSON.stringify(createHookConfig(configuredSettings()), null, 2)}\n`;
  await fs.writeFile(target, content, { encoding: 'utf8', mode: 0o600 });
  return target;
}

async function removeHookFile(): Promise<string> {
  const target = hookPath();
  await fs.rm(target, { force: true });
  return target;
}

async function play(kind: 'attention' | 'sessionEnd'): Promise<void> {
  try {
    const settings = configuredSettings();
    const waveform =
      kind === 'attention' ? settings.attentionWaveform : settings.sessionEndWaveform;
    const args = ['haptic', waveform];
    if (settings.device) {
      args.push('--device', settings.device);
    }
    await execFileAsync(settings.command, args, {
      timeout: 5_000,
      windowsHide: true,
    });
    void vscode.window.showInformationMessage(`OpenLogi played ${waveform}.`);
  } catch (error) {
    const detail =
      error instanceof Error ? error.message : 'The OpenLogi CLI command failed.';
    void vscode.window.showErrorMessage(`OpenLogi haptic failed: ${detail}`);
  }
}

async function enable(context: vscode.ExtensionContext): Promise<void> {
  try {
    const target = await writeHookFile();
    await context.globalState.update(ENABLED_KEY, true);
    void vscode.window.showInformationMessage(
      `OpenLogi Copilot haptics enabled in ${target}.`,
    );
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(`Could not enable OpenLogi haptics: ${detail}`);
  }
}

async function disable(context: vscode.ExtensionContext): Promise<void> {
  try {
    const target = await removeHookFile();
    await context.globalState.update(ENABLED_KEY, false);
    void vscode.window.showInformationMessage(
      `OpenLogi Copilot haptics disabled; removed ${target}.`,
    );
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(`Could not disable OpenLogi haptics: ${detail}`);
  }
}

async function refreshEnabledHook(context: vscode.ExtensionContext): Promise<void> {
  if (!context.globalState.get<boolean>(ENABLED_KEY, false)) {
    return;
  }
  try {
    await writeHookFile();
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(`Could not update OpenLogi hooks: ${detail}`);
  }
}

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('openlogiHaptics.enable', () => enable(context)),
    vscode.commands.registerCommand('openlogiHaptics.disable', () => disable(context)),
    vscode.commands.registerCommand('openlogiHaptics.testAttention', () => play('attention')),
    vscode.commands.registerCommand('openlogiHaptics.testSessionEnd', () => play('sessionEnd')),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('openlogiHaptics')) {
        void refreshEnabledHook(context);
      }
    }),
  );

  void refreshEnabledHook(context);
}

export function deactivate(): void {}
