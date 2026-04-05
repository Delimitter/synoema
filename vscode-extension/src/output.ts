// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

import * as vscode from 'vscode';

let outputChannel: vscode.OutputChannel | null = null;

export function getOutputChannel(): vscode.OutputChannel {
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel('Synoema');
  }
  return outputChannel;
}

export function appendLine(message: string): void {
  getOutputChannel().appendLine(message);
}

export function append(message: string): void {
  getOutputChannel().append(message);
}

export function clear(): void {
  getOutputChannel().clear();
}

export function show(): void {
  getOutputChannel().show(true);
}

export function hide(): void {
  getOutputChannel().hide();
}

export function dispose(): void {
  if (outputChannel) {
    outputChannel.dispose();
    outputChannel = null;
  }
}
