// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

import * as vscode from 'vscode';
import { execSync } from 'child_process';

export interface SynoemaConfig {
  path: string;
  showOutput: boolean;
  timeout: number;
}

export function getConfig(): SynoemaConfig {
  const config = vscode.workspace.getConfiguration('synoema');
  const path = config.get<string>('path') || 'synoema';
  const showOutput = config.get<boolean>('showOutput') ?? true;
  const timeout = config.get<number>('timeout') ?? 30000;

  return { path, showOutput, timeout };
}

export function checkSynoemaCliExists(path: string): boolean {
  try {
    execSync(`${path} --version`, { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

export function getSynoemaVersion(path: string): string | null {
  try {
    const output = execSync(`${path} --version`, { encoding: 'utf-8' });
    return output.trim();
  } catch {
    return null;
  }
}
