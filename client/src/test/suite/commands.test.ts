import * as assert from "assert";
import * as vscode from "vscode";
import { URI } from "vscode-uri";
const { suite, test, suiteSetup, suiteTeardown } = require("mocha");

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
const examplesVscode = join(__dirname, testFolderLocation, ".vscode");
const examplesReadme = join(__dirname, testFolderLocation, "README.md");
const examplesScript = join(
  __dirname,
  testFolderLocation,
  "scripting/testSuite.sp"
);
const examplesGithub = join(__dirname, testFolderLocation, ".github");

suite("Run tests", () => {
  suiteSetup(async () => {
    let uri: URI = URI.file(join(__dirname, testFolderLocation));
    vscode.commands.executeCommand("vscode.openFolder", uri);
    rmdir(examplesVscode);
    if (fs.existsSync(examplesReadme)) {
      fs.unlinkSync(examplesReadme);
    }
    if (fs.existsSync(examplesScript)) {
      fs.unlinkSync(examplesScript);
    }
    rmdir(examplesGithub);
    vscode.commands.executeCommand("vscode.open", mainUri);

    // Give some time to parse everything
    await sleep(3000);
  });

  suiteTeardown("Remove files after the tests", () => {
    rmdir(examplesVscode);
    if (fs.existsSync(examplesReadme)) {
      fs.unlinkSync(examplesReadme);
    }
    if (fs.existsSync(examplesScript)) {
      fs.unlinkSync(examplesScript);
    }
    rmdir(examplesGithub);
  });

  suite("Test commands", () => {
    test("Create Task Command", () => {
      rmdir(examplesVscode);
      let error: number = CreateTaskCommand();
      // If sm_home is not defined, this command will error out.
      // This counts this error as expected behaviour.
      assert.ok(error == 0 || error == 1);
    });

    test("Create Script Command", () => {
      assert.equal(CreateScriptCommand(), 0);
    });

    test("Create ReadMe Command", () => {
      assert.equal(CreateREADMECommand(), 0);
    });

    test("Create Master Command", () => {
      assert.equal(CreateMasterCommand(), 0);
    });
  });

  suite("Test providers", () => {
    suite("Test Position provider", () => {
      test("Test ConVar g_cvWebhook", () => {
        let position: vscode.Position = new vscode.Position(16, 8);
        vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(
              location[0].range,
              new vscode.Range(16, 7, 16, 18)
            );
            assert.equal(location[0].uri.fsPath, mainUri.fsPath);
          });
      });

      test("Test OnPluginStart;", () => {
        let position: vscode.Position = new vscode.Position(19, 19);
        vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(
              location[0].range,
              new vscode.Range(125, 13, 125, 26)
            );
            assert.ok(
              location[0].uri.fsPath.includes(
                "\\scripting\\include\\sourcemod.inc"
              )
            );
          });
      });

      test("Test CreateConVar", () => {
        let position: vscode.Position = new vscode.Position(22, 22);
        vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(
              location[0].range,
              new vscode.Range(80, 14, 80, 26)
            );
            assert.ok(
              location[0].uri.fsPath.includes(
                "\\scripting\\include\\convars.inc"
              )
            );
          });
      });

      test("Test FooEnum test", () => {
        let position: vscode.Position = new vscode.Position(17, 10);
        vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(
              location[0].range,
              new vscode.Range(17, 8, 17, 12)
            );
            assert.equal(location[0].uri.fsPath, mainUri.fsPath);
          });
      });
    });

    test("Test Doc Completion provider", () => {
      let position = new vscode.Position(31, 0);
      vscode.commands
        .executeCommand(
          "vscode.executeCompletionItemProvider",
          mainUri,
          position,
          "/*"
        )
        .then((docCompletion: vscode.CompletionList) => {
          assert.ok(docCompletion.items.length > 0);
        });
    });

    test("Test Signature Help provider", () => {
      let position = new vscode.Position(24, 16);
      vscode.commands
        .executeCommand(
          "vscode.executeSignatureHelpProvider",
          mainUri,
          position,
          "("
        )
        .then((signature: vscode.SignatureHelp) => {
          assert.deepEqual(
            signature.signatures[0].label,
            'native void RegConsoleCmd(const char[] cmd, ConCmd callback, const char[] description="", int flags=0)'
          );
          assert.equal(signature.signatures[0].parameters.length, 4);
        });
    });

    test("Test Formater provider", () => {
      vscode.commands
        .executeCommand("vscode.executeFormatDocumentProvider", mainUri)
        .then((edits: vscode.TextEdit[]) => {
          assert.ok(edits !== undefined);
        });
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
