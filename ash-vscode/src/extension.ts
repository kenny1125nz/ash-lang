import * as vscode from "vscode";
import * as child_process from "child_process";
import * as os from "os";
import * as path from "path";
import * as fs from "fs";

let outputChannel: vscode.OutputChannel;
let runningProcess: child_process.ChildProcess | null = null;
let binaryPath: string | null = null;

const INSTALL_INSTRUCTIONS =
  "Install via npm: npm i -g @ash-lang/cli  |  GitHub Releases: https://github.com/kenny1125nz/ash-lang/releases";

function resolveBinaryPath(extensionPath: string): string | null {
  const whichCmd = os.platform() === "win32" ? "where" : "which";
  try {
    const result = child_process
      .execSync(`${whichCmd} ash`, { encoding: "utf8", timeout: 3000 })
      .trim();
    if (result) {
      return "ash";
    }
  } catch {
    // not on PATH
  }

  const platform = os.platform();
  const arch = os.arch();
  let platformDir: string;

  if (platform === "linux") {
    platformDir = "linux-x64";
  } else if (platform === "darwin" && arch === "arm64") {
    platformDir = "darwin-arm64";
  } else if (platform === "darwin") {
    platformDir = "darwin-x64";
  } else if (platform === "win32") {
    platformDir = "win-x64";
  } else {
    return null;
  }

  const binaryName = platform === "win32" ? "ash.exe" : "ash";
  const bundledPath = path.join(extensionPath, "bin", platformDir, binaryName);

  if (fs.existsSync(bundledPath)) {
    if (platform !== "win32") {
      try {
        fs.chmodSync(bundledPath, 0o755);
      } catch {
        // chmod failed, try running anyway
      }
    }
    return bundledPath;
  }

  return null;
}

function executeAsh(
  args: string[],
  cwd: string,
  prefix?: string,
): void {
  if (runningProcess) {
    vscode.window.showErrorMessage(
      "A script is already running. Use 'Ash: Stop Script' to stop it first.",
    );
    return;
  }

  if (!binaryPath) {
    vscode.window.showErrorMessage(
      `ash not found. ${INSTALL_INSTRUCTIONS}`,
    );
    return;
  }

  const startTime = Date.now();
  outputChannel.clear();
  outputChannel.show(true);

  const proc = child_process.spawn(binaryPath, args, { cwd });
  runningProcess = proc;

  proc.stdout?.on("data", (data: Buffer) => {
    for (const line of data.toString().split("\n")) {
      if (line !== "") {
        outputChannel.appendLine(
          (prefix ? `[${prefix}] ` : "") + line,
        );
      }
    }
  });

  proc.stderr?.on("data", (data: Buffer) => {
    for (const line of data.toString().split("\n")) {
      if (line !== "") {
        outputChannel.appendLine(
          (prefix ? `[${prefix}] ` : "") + line,
        );
      }
    }
  });

  proc.on("close", (code) => {
    runningProcess = null;
    const elapsedSec = ((Date.now() - startTime) / 1000).toFixed(2);
    outputChannel.appendLine(
      `\u2500\u2500 Finished in ${elapsedSec}s with exit code ${code} \u2500\u2500`,
    );
  });

  proc.on("error", (err) => {
    runningProcess = null;
    outputChannel.appendLine(
      `\u2500\u2500 Error: ${err.message} \u2500\u2500`,
    );
  });
}

function stopRunningProcess(): void {
  if (!runningProcess) {
    vscode.window.showInformationMessage("No script is running.");
    return;
  }

  if (os.platform() === "win32") {
    try {
      child_process.exec(`taskkill /pid ${runningProcess.pid} /T /F`);
    } catch {
      // taskkill may fail if process already exited
    }
  } else {
    runningProcess.kill("SIGTERM");
  }

  outputChannel.appendLine(
    "\u2500\u2500 Script stopped by user \u2500\u2500",
  );
  runningProcess = null;
}

export function activate(context: vscode.ExtensionContext) {
  binaryPath = resolveBinaryPath(context.extensionPath);
  outputChannel = vscode.window.createOutputChannel("Ash");

  if (binaryPath) {
    outputChannel.appendLine(`Using ash: ${binaryPath}`);
  } else {
    outputChannel.appendLine(
      `ash not found on PATH and no bundled binary available.\n${INSTALL_INSTRUCTIONS}`,
    );
  }

  const runScript = vscode.commands.registerCommand("ash.runScript", () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage("No active editor.");
      return;
    }
    if (editor.document.languageId !== "ash") {
      vscode.window.showErrorMessage(
        "Active file is not an .ash script.",
      );
      return;
    }
    const filePath = editor.document.uri.fsPath;
    const cwd = path.dirname(filePath);
    executeAsh([filePath], cwd);
  });

  const checkScript = vscode.commands.registerCommand(
    "ash.checkScript",
    () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor.");
        return;
      }
      if (editor.document.languageId !== "ash") {
        vscode.window.showErrorMessage(
          "Active file is not an .ash script.",
        );
        return;
      }
      const filePath = editor.document.uri.fsPath;
      const cwd = path.dirname(filePath);
      executeAsh(["--dry-run", filePath], cwd, "dry-run");
    },
  );

  const stopScript = vscode.commands.registerCommand(
    "ash.stopScript",
    () => {
      stopRunningProcess();
    },
  );

  context.subscriptions.push(runScript, checkScript, stopScript);
}

export function deactivate() {
  if (runningProcess) {
    stopRunningProcess();
  }
  if (outputChannel) {
    outputChannel.dispose();
  }
}
