// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

import * as vscode from 'vscode';
import { runFile } from './commands/run';
import { jitFile } from './commands/jit';
import { evalSelection } from './commands/eval';
import { checkSynoemaCliExists, getSynoemaVersion, getConfig } from './config';
import * as output from './output';

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const config = getConfig();

  // Check if synoema CLI is available
  if (!checkSynoemaCliExists(config.path)) {
    vscode.window.showWarningMessage(
      `Synoema CLI not found at "${config.path}". Please install it or configure the correct path in settings.`,
      'View Documentation',
      'Open Settings'
    ).then((selection) => {
      if (selection === 'View Documentation') {
        vscode.env.openExternal(vscode.Uri.parse('https://github.com/Delimitter/synoema'));
      } else if (selection === 'Open Settings') {
        vscode.commands.executeCommand('workbench.action.openSettings', 'synoema');
      }
    });
  } else {
    const version = getSynoemaVersion(config.path);
    output.appendLine(`Synoema ${version} ready`);
  }

  // Register commands
  const runCommand = vscode.commands.registerCommand('synoema.run', runFile);
  const jitCommand = vscode.commands.registerCommand('synoema.jit', jitFile);
  const evalCommand = vscode.commands.registerCommand('synoema.eval', evalSelection);

  context.subscriptions.push(runCommand, jitCommand, evalCommand);
}

export function deactivate(): void {
  output.dispose();
}
