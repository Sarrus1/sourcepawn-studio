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

const testFolderLocation = "/../../../src/test/examples/";
const testFolderLocationBis = "/../../../src/test/testSuite/";

suite("Extension Test", async () => {
  await test("Create Task Command", () => {
    let examplesVscode = join(__dirname, testFolderLocation, ".vscode");
    rmdir(examplesVscode);
    let error: number = CreateTaskCommand();
    // If sm_home is not defined, this command will error out.
    // This counts this error as expected behaviour.
    let test: boolean = error == 0 || error == 1;
    assert.equal(test, true);
    rmdir(examplesVscode);
  });

  await test("Create Script Command", () => {
    let examplesScripting = join(__dirname, testFolderLocation, "scripting");
    rmdir(examplesScripting);
    let error: number = CreateScriptCommand();
    assert.equal(error, 0);
    rmdir(examplesScripting);
  });

  await test("Create ReadMe Command", () => {
    let examplesReadme = join(__dirname, testFolderLocation, "README.md");
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
    let examplesGithub = join(__dirname, testFolderLocation, ".github");
    rmdir(examplesGithub);
    let error: number = CreateMasterCommand();
    assert.equal(error, 0);
    rmdir(examplesGithub);
  });

  await test("Open and parse files", async () => {
    let uri: URI = URI.file(join(__dirname, testFolderLocationBis));
    vscode.commands.executeCommand("vscode.openFolder", uri);
    // Give time to parse the file
    await sleep(2000);
    let fileUri: URI = URI.file(
      join(__dirname, testFolderLocationBis, "scripting/main.sp")
    );
    let position: vscode.Position = new vscode.Position(4, 3);
    let location = await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      fileUri,
      position
    );
    assert.deepEqual(location, [
      {
        targetRange: new vscode.Range(0, 3, 0, 8),
        targetUri: fileUri,
      },
    ]);
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
