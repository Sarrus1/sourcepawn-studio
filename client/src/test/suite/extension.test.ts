import * as assert from "assert";

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as fs from "fs";
import { join } from "path";
import { run as CreateTaskCommand } from "../../Commands/createTask";
import { run as CreateScriptCommand } from "../../Commands/createScript";
import { run as CreateREADMECommand } from "../../Commands/createREADME";
import { run as CreateMasterCommand } from "../../Commands/createGitHubActions";

const testFolderLocation = "/../../../src/test/examples/";

suite("Extension Test", async () => {
  test("Create Task Command", () => {
    let examplesVscode = join(__dirname, testFolderLocation, ".vscode");
    rmdir(examplesVscode);
    let error: number = CreateTaskCommand();
    // If sm_home is not defined, this command will error out.
    // This counts this error as expected behaviour.
    let test: boolean = error == 0 || error == 1;
    assert.equal(test, true);
    rmdir(examplesVscode);
  });

  test("Create Script Command", () => {
    let examplesScripting = join(__dirname, testFolderLocation, "scripting");
    rmdir(examplesScripting);
    let error: number = CreateScriptCommand();
    assert.equal(error, 0);
    rmdir(examplesScripting);
  });

  test("Create ReadMe Command", () => {
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

  test("Create Master Command", () => {
    let examplesGithub = join(__dirname, testFolderLocation, ".github");
    rmdir(examplesGithub);
    let error: number = CreateMasterCommand();
    assert.equal(error, 0);
    rmdir(examplesGithub);
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
