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
const testKvLocation = "test.phrases.txt";
const kvUri: URI = URI.file(
  join(__dirname, testFolderLocation, testKvLocation)
);
const examplesVscode = join(__dirname, testFolderLocation, ".vscode");
const examplesReadme = join(__dirname, testFolderLocation, "README.md");
const examplesGithub = join(__dirname, testFolderLocation, ".github");
const examplesChangelog = join(__dirname, testFolderLocation, "CHANGELOG.md");

suite("Run tests", () => {
  suiteSetup(async () => {
    const uri: URI = URI.file(join(__dirname, testFolderLocation));
    vscode.commands.executeCommand("vscode.openFolder", uri);
    rmdir(examplesVscode);
    if (fs.existsSync(examplesReadme)) {
      fs.rmSync(examplesReadme);
    }
    if (fs.existsSync(examplesChangelog)) {
      fs.rmSync(examplesChangelog);
    }
    rmdir(examplesGithub);
    vscode.commands.executeCommand("vscode.open", kvUri);

    // Give some time to parse everything
    await sleep(2000);
  });

  suiteTeardown("Remove files after the tests", () => {
    rmdir(examplesVscode);
    if (fs.existsSync(examplesReadme)) {
      fs.unlinkSync(examplesReadme);
    }
    if (fs.existsSync(examplesChangelog)) {
      fs.unlinkSync(examplesChangelog);
    }
    rmdir(examplesGithub);
  });

  suite("Test commands", () => {
    test("Create Task Command", () => {
      rmdir(examplesVscode);
      const error: number = CreateTaskCommand();
      // If sm_home is not defined, this command will error out.
      // This counts this error as expected behaviour.
      assert.ok(error == 0 || error == 1);
    });

    test("Create Script Command", () => {
      assert.strictEqual(CreateScriptCommand(), 0);
    });

    test("Create Changelog Command", () => {
      assert.strictEqual(CreateChangelogCommand(), 0);
    });

    test("Create Readme Command", () => {
      assert.strictEqual(CreateREADMECommand(), 0);
    });

    test("Create Master Command", () => {
      assert.strictEqual(CreateMasterCommand(), 0);
    });
  });

  suite("Test providers", () => {
    test("Test KV Formater provider", () => {
      return vscode.commands
        .executeCommand("vscode.executeFormatDocumentProvider", kvUri)
        .then((edits: vscode.TextEdit[]) => {
          assert.ok(edits !== undefined);
        });
    });
  });
});

function rmdir(dir: string): void {
  if (!fs.existsSync(dir)) {
    return;
  }
  return fs.rmSync(dir);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
