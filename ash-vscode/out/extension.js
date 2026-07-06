"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const child_process = __importStar(require("child_process"));
const os = __importStar(require("os"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
let outputChannel;
let runningProcess = null;
let binaryPath = null;
const INSTALL_INSTRUCTIONS = "Install via npm: npm i -g @ash-lang/cli  |  GitHub Releases: https://github.com/kenny1125nz/ash-lang/releases";
function resolveBinaryPath(extensionPath) {
    const whichCmd = os.platform() === "win32" ? "where" : "which";
    try {
        const result = child_process
            .execSync(`${whichCmd} ash`, { encoding: "utf8", timeout: 3000 })
            .trim();
        if (result) {
            return "ash";
        }
    }
    catch {
        // not on PATH
    }
    const platform = os.platform();
    const arch = os.arch();
    let platformDir;
    if (platform === "linux") {
        platformDir = "linux-x64";
    }
    else if (platform === "darwin" && arch === "arm64") {
        platformDir = "darwin-arm64";
    }
    else if (platform === "darwin") {
        platformDir = "darwin-x64";
    }
    else if (platform === "win32") {
        platformDir = "win-x64";
    }
    else {
        return null;
    }
    const binaryName = platform === "win32" ? "ash.exe" : "ash";
    const bundledPath = path.join(extensionPath, "bin", platformDir, binaryName);
    if (fs.existsSync(bundledPath)) {
        if (platform !== "win32") {
            try {
                fs.chmodSync(bundledPath, 0o755);
            }
            catch {
                // chmod failed, try running anyway
            }
        }
        return bundledPath;
    }
    return null;
}
function executeAsh(args, cwd, prefix) {
    if (runningProcess) {
        vscode.window.showErrorMessage("A script is already running. Use 'Ash: Stop Script' to stop it first.");
        return;
    }
    if (!binaryPath) {
        vscode.window.showErrorMessage(`ash not found. ${INSTALL_INSTRUCTIONS}`);
        return;
    }
    const startTime = Date.now();
    outputChannel.clear();
    outputChannel.show(true);
    const proc = child_process.spawn(binaryPath, args, { cwd });
    runningProcess = proc;
    proc.stdout?.on("data", (data) => {
        for (const line of data.toString().split("\n")) {
            if (line !== "") {
                outputChannel.appendLine((prefix ? `[${prefix}] ` : "") + line);
            }
        }
    });
    proc.stderr?.on("data", (data) => {
        for (const line of data.toString().split("\n")) {
            if (line !== "") {
                outputChannel.appendLine((prefix ? `[${prefix}] ` : "") + line);
            }
        }
    });
    proc.on("close", (code) => {
        runningProcess = null;
        const elapsedSec = ((Date.now() - startTime) / 1000).toFixed(2);
        outputChannel.appendLine(`\u2500\u2500 Finished in ${elapsedSec}s with exit code ${code} \u2500\u2500`);
    });
    proc.on("error", (err) => {
        runningProcess = null;
        outputChannel.appendLine(`\u2500\u2500 Error: ${err.message} \u2500\u2500`);
    });
}
function stopRunningProcess() {
    if (!runningProcess) {
        vscode.window.showInformationMessage("No script is running.");
        return;
    }
    if (os.platform() === "win32") {
        try {
            child_process.exec(`taskkill /pid ${runningProcess.pid} /T /F`);
        }
        catch {
            // taskkill may fail if process already exited
        }
    }
    else {
        runningProcess.kill("SIGTERM");
    }
    outputChannel.appendLine("\u2500\u2500 Script stopped by user \u2500\u2500");
    runningProcess = null;
}
function activate(context) {
    binaryPath = resolveBinaryPath(context.extensionPath);
    outputChannel = vscode.window.createOutputChannel("Ash");
    if (binaryPath) {
        outputChannel.appendLine(`Using ash: ${binaryPath}`);
    }
    else {
        outputChannel.appendLine(`ash not found on PATH and no bundled binary available.\n${INSTALL_INSTRUCTIONS}`);
    }
    const runScript = vscode.commands.registerCommand("ash.runScript", () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage("No active editor.");
            return;
        }
        if (editor.document.languageId !== "ash") {
            vscode.window.showErrorMessage("Active file is not an .ash script.");
            return;
        }
        const filePath = editor.document.uri.fsPath;
        const cwd = path.dirname(filePath);
        executeAsh([filePath], cwd);
    });
    const checkScript = vscode.commands.registerCommand("ash.checkScript", () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage("No active editor.");
            return;
        }
        if (editor.document.languageId !== "ash") {
            vscode.window.showErrorMessage("Active file is not an .ash script.");
            return;
        }
        const filePath = editor.document.uri.fsPath;
        const cwd = path.dirname(filePath);
        executeAsh(["--dry-run", filePath], cwd, "dry-run");
    });
    const stopScript = vscode.commands.registerCommand("ash.stopScript", () => {
        stopRunningProcess();
    });
    context.subscriptions.push(runScript, checkScript, stopScript);
}
function deactivate() {
    if (runningProcess) {
        stopRunningProcess();
    }
    if (outputChannel) {
        outputChannel.dispose();
    }
}
//# sourceMappingURL=extension.js.map