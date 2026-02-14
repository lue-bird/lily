import * as vscode from "vscode";
import {
  LanguageClientOptions,
} from "vscode-languageclient";
import {
  LanguageClient,
  ServerOptions,
} from "vscode-languageclient/node";
import * as child_process from "node:child_process";

let client: LanguageClient | null = null;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const languageServerExecutableName: string =
    // switch to your locally built debug executable path when developing
    "still";
  context.subscriptions.push(vscode.commands.registerCommand("still.commands.restart", async () => {
    if (client !== null) {
      await client.stop();
      await client.start();
    }
  }));

  const serverOptions: ServerOptions = async () => {
    return child_process.spawn(languageServerExecutableName)
  };
  const clientOptions: LanguageClientOptions = {
    diagnosticCollectionName: "still",
    documentSelector: [
      {
        scheme: "file",
        language: "still",
      }
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.still")
    },
  };
  client = new LanguageClient(
    "still",
    "still",
    serverOptions,
    clientOptions,
  );
  await client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (client !== null) {
    return client.stop()
  }
  return undefined;
}
