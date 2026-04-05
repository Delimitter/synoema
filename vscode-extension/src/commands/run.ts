// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { getConfig } from '../config';
import * as output from '../output';

export async function runFile(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('No active editor');
    return;
  }

  const filePath = editor.document.fileName;
  if (!filePath.endsWith('.sno')) {
    vscode.window.showErrorMessage('Current file is not a Synoema program (.sno)');
    return;
  }

  const config = getConfig();

  // Save the file first
  if (editor.document.isDirty) {
    await editor.document.save();
  }

  output.clear();
  output.appendLine(`$ ${config.path} run ${filePath}`);

  if (config.showOutput) {
    output.show();
  }

  const startTime = Date.now();

  return new Promise((resolve) => {
    const child = spawn(config.path, ['run', filePath], {
      timeout: config.timeout,
      shell: true
    });

    let stdout = '';
    let stderr = '';

    child.stdout?.on('data', (data: Buffer) => {
      const text = data.toString();
      stdout += text;
      output.append(text);
    });

    child.stderr?.on('data', (data: Buffer) => {
      const text = data.toString();
      stderr += text;
      output.append(text);
    });

    child.on('close', (code: number | null) => {
      const duration = Date.now() - startTime;

      if (code === 0) {
        output.appendLine(`✓ Duration: ${duration}ms`);
      } else {
        output.appendLine(`✗ Exit code: ${code} (${duration}ms)`);
        vscode.window.showErrorMessage(`Synoema run failed with exit code ${code}`);
      }

      resolve();
    });

    child.on('error', (err: Error) => {
      output.appendLine(`Error: ${err.message}`);
      vscode.window.showErrorMessage(`Failed to run synoema: ${err.message}`);
      resolve();
    });
  });
}
