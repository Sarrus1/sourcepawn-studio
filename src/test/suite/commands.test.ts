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
import { run as CreateChangelogCommand } from "../../Commands/createCHANGELOG";

const testFolderLocation = "/../../../src/test/testSuite/";
const testMainLocation = "scripting/main.sp";
const testSecondaryLocation = "scripting/include/second.sp";
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
    await sleep(2000);
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

    test("Create Changelog Command", () => {
      assert.equal(CreateChangelogCommand(), 0);
    });

    test("Create Readme Command", () => {
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
        return vscode.commands
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

      test("Test FooEnum test", () => {
        let position: vscode.Position = new vscode.Position(17, 10);
        return vscode.commands
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

      test("Test OnPluginStart", () => {
        let position: vscode.Position = new vscode.Position(19, 19);
        return vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(
              location[0].range,
              new vscode.Range(125, 13, 125, 26)
            );
            assert.ok(location[0].uri.fsPath.endsWith("sourcemod.inc"));
          });
      });

      test("Test CreateConVar", () => {
        let position: vscode.Position = new vscode.Position(21, 22);
        return vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(
              location[0].range,
              new vscode.Range(80, 14, 80, 26)
            );
            assert.ok(location[0].uri.fsPath.endsWith("convars.inc"));
          });
      });

      test("Test test line 28", () => {
        let position: vscode.Position = new vscode.Position(28, 4);
        return vscode.commands
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

      test("Test test.fullAccountID line 28", () => {
        let position: vscode.Position = new vscode.Position(28, 13);
        return vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(location[0].range, new vscode.Range(4, 6, 4, 19));
            assert.equal(location[0].uri.fsPath, secondaryUri.fsPath);
          });
      });

      test("Test test line 29", () => {
        let position: vscode.Position = new vscode.Position(29, 4);
        return vscode.commands
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

      test("Test test.Init(1) line 29", () => {
        let position: vscode.Position = new vscode.Position(29, 9);
        return vscode.commands
          .executeCommand("vscode.executeDefinitionProvider", mainUri, position)
          .then((location: vscode.Location[]) => {
            assert.ok(location.length > 0);
            assert.deepEqual(location[0].range, new vscode.Range(6, 6, 6, 10));
            assert.equal(location[0].uri.fsPath, secondaryUri.fsPath);
          });
      });
    });

    suite("Test Hover provider", () => {
      test("Test ConVar g_cvWebhook", () => {
        let position: vscode.Position = new vscode.Position(16, 3);
        return vscode.commands
          .executeCommand("vscode.executeHoverProvider", mainUri, position)
          .then((hover: vscode.Hover[]) => {
            assert.ok(hover.length > 0);
            assert.deepEqual(hover[0].range, new vscode.Range(16, 0, 16, 6));
          });
      });

      test("Test OnPluginStart", () => {
        let position: vscode.Position = new vscode.Position(19, 19);
        return vscode.commands
          .executeCommand("vscode.executeHoverProvider", mainUri, position)
          .then((hover: vscode.Hover[]) => {
            assert.ok(hover.length > 0);
            assert.deepEqual(hover[0].range, new vscode.Range(19, 12, 19, 25));
          });
      });

      test("Test CreateConVar", () => {
        let position: vscode.Position = new vscode.Position(21, 22);
        return vscode.commands
          .executeCommand("vscode.executeHoverProvider", mainUri, position)
          .then((hover: vscode.Hover[]) => {
            assert.ok(hover.length > 0);
            assert.deepEqual(hover[0].range, new vscode.Range(21, 15, 21, 27));
          });
      });

      test("Test test.Init(1) line 29", () => {
        let position: vscode.Position = new vscode.Position(29, 9);
        return vscode.commands
          .executeCommand("vscode.executeHoverProvider", mainUri, position)
          .then((hover: vscode.Hover[]) => {
            assert.ok(hover.length > 0);
            assert.deepEqual(hover[0].range, new vscode.Range(29, 6, 29, 10));
          });
      });
    });

    test("Test Doc Completion provider", () => {
      let position = new vscode.Position(31, 0);
      return vscode.commands
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
      return vscode.commands
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

    /*test("Test Formater provider", () => {
      return vscode.commands
        .executeCommand("vscode.executeFormatDocumentProvider", mainUri)
        .then((edits: vscode.TextEdit[]) => {
          assert.ok(edits !== undefined);
        });
    });
    */

    /*
    test("Test Semantic Token Highlighting provider", () => {
      return vscode.commands
        .executeCommand("vscode.provideDocumentSemanticTokens", mainUri)
        .then((tokens: vscode.SemanticTokens) => {
          // For now we test that it's not null
          assert.ok(tokens !== undefined && tokens.data.length === 1);
        });
    });
    */
  });
});

function rmdir(dir: string): void {
  if (!fs.existsSync(dir)) {
    return;
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

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
