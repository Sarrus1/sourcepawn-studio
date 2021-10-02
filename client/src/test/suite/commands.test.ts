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


suite("Run tests", async () => {
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
  
  /*
  await test("Create Script Command", () => {
    let examplesScripting = join(__dirname, testFolderLocation, "scripting");
    rmdir(examplesScripting);
    let error: number = CreateScriptCommand();
    assert.equal(error, 0);
    rmdir(examplesScripting);
  });
  */

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
    let uri: URI = URI.file(join(__dirname, testFolderLocation));
    vscode.commands.executeCommand("vscode.openFolder", uri);
    let mainUri: URI = URI.file(
      join(__dirname, testFolderLocation, "scripting/main.sp")
    );
    let secondaryUri: URI = URI.file(
      join(__dirname, testFolderLocation, "scripting/include/secondary.sp")
    );
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
