import * as assert from "assert";
import * as vscode from "vscode";
import { URI } from "vscode-uri";

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as fs from "fs";
import { join } from "path";
import { run as CreateTaskCommand } from "../../Commands/createTask";
import { run as CreateScriptCommand } from "../../Commands/createScript";
import { run as CreateREADMECommand } from "../../Commands/createREADME";
import { run as CreateMasterCommand } from "../../Commands/createGitHubActions";

const testFolderLocation = "/../../../client/src/test/testSuite/";
const testMainLocation = "scripting/main.sp";
const testSecondaryLocation = "scripting/include/secondary.sp";
const mainUri: URI = URI.file(
  join(__dirname, testFolderLocation, testMainLocation)
);
const secondaryUri: URI = URI.file(
  join(__dirname, testFolderLocation, testSecondaryLocation)
);

suite("Run tests", async () => {
  suiteSetup(async () => {
    let uri: URI = URI.file(join(__dirname, testFolderLocation));
    await vscode.commands.executeCommand("vscode.openFolder", uri);
  });

  suite("Test commands", async () => {
    const examplesVscode = join(__dirname, testFolderLocation, ".vscode");
    const examplesReadme = join(__dirname, testFolderLocation, "README.md");
    const examplesScript = join(
      __dirname,
      testFolderLocation,
      "scripting/testSuite.sp"
    );
    const examplesGithub = join(__dirname, testFolderLocation, ".github");

    await suiteSetup("Remove files before", async () => {
      rmdir(examplesVscode);
      if (fs.existsSync(examplesReadme)) {
        fs.unlinkSync(examplesReadme);
      }
      if (fs.existsSync(examplesScript)) {
        fs.unlinkSync(examplesScript);
      }
      rmdir(examplesGithub);
    });

    await test("Create Task Command", () => {
      rmdir(examplesVscode);
      let error: number = CreateTaskCommand();
      // If sm_home is not defined, this command will error out.
      // This counts this error as expected behaviour.
      let test: boolean = error == 0 || error == 1;
      assert.equal(test, true);
    });

    await test("Create Script Command", () => {
      let error: number = CreateScriptCommand();
      assert.equal(error, 0);
    });

    await test("Create ReadMe Command", () => {
      if (fs.existsSync(examplesReadme)) {
        fs.unlinkSync(examplesReadme);
      }
      let error: number = CreateREADMECommand();
      assert.equal(error, 0);
      if (fs.existsSync(examplesReadme)) {
        fs.unlinkSync(examplesReadme);
      }
    });

    await test("Create Master Command", () => {
      rmdir(examplesGithub);
      let error: number = CreateMasterCommand();
      assert.equal(error, 0);
      rmdir(examplesGithub);
    });

    await suiteTeardown("Remove files after the tests", async () => {
      rmdir(examplesVscode);
      if (fs.existsSync(examplesReadme)) {
        fs.unlinkSync(examplesReadme);
      }
      if (fs.existsSync(examplesScript)) {
        fs.unlinkSync(examplesScript);
      }
      rmdir(examplesGithub);
    });
  });

  await suite("Test providers", async () => {
    await test("Test Position Provider", async () => {
      vscode.commands.executeCommand("vscode.open", mainUri);

      // Give some time to parse everything
      await sleep(3000);

      // Test ConVar g_cvWebhook;
      let position: vscode.Position = new vscode.Position(16, 8);
      let location: vscode.Location[] = await vscode.commands.executeCommand(
        "vscode.executeDefinitionProvider",
        mainUri,
        position
      );
      assert.ok(location.length > 0);
      assert.deepEqual(location[0].range, new vscode.Range(16, 7, 16, 18));
      assert.equal(location[0].uri.fsPath, mainUri.fsPath);

      // Test FooEnum test;
      position = new vscode.Position(17, 10);
      location = await vscode.commands.executeCommand(
        "vscode.executeDefinitionProvider",
        mainUri,
        position
      );
      assert.ok(location.length > 0);
      assert.deepEqual(location[0].range, new vscode.Range(17, 8, 17, 12));
      assert.equal(location[0].uri.fsPath, mainUri.fsPath);
    });

    await test("Test Doc Completion provider", async () => {
      let position = new vscode.Position(31, 0);
      let docCompletion: vscode.CompletionList = await vscode.commands.executeCommand(
        "vscode.executeCompletionItemProvider",
        mainUri,
        position,
        "/*"
      );

      assert.ok(docCompletion.items.length > 0);
    });

    await test("Test Signature Help Provider", async () => {
      let position = new vscode.Position(24, 16);
      let signature: vscode.SignatureHelp = await vscode.commands.executeCommand(
        "vscode.executeSignatureHelpProvider",
        mainUri,
        position,
        "("
      );

      assert.ok(signature.signatures.length > 0);
    });
  });

  // Test the formater separatly to avoid interferences with the other tests
  await suite("Test Formater provider", async () => {
    await test("Test Formater Provider", async () => {
      let edits: vscode.TextEdit[] = await vscode.commands.executeCommand(
        "vscode.executeFormatDocumentProvider",
        mainUri
      );
      assert.ok(edits !== undefined);
    });
  });
});

function rmdir(dir) {
  if (!fs.existsSync(dir)) {
    return null;
  }
  fs.readdirSync(dir).forEach((f) => {
    let pathname = join(dir, f);
    if (!fs.existsSync(pathname)) {
      return fs.unlinkSync(pathname);
    }
    if (fs.statSync(pathname).isDirectory()) {
      return rmdir(pathname);
    } else {
      return fs.unlinkSync(pathname);
    }
  });
  return fs.rmdirSync(dir);
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
