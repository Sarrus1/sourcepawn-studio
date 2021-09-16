import * as path from "path";
import * as Mocha from "mocha";
import * as glob from "glob";

export function run(): Promise<void> {
  const nyc = setupCoverage();

  const mochaOpts: Mocha.MochaOptions = {
    timeout: 10 * 1000,
    ui: "tdd",
    color: true,
  };

  if (process.env.ONLY_MINSPEC === "true") {
    mochaOpts.grep = "node runtime"; // may eventually want a more dynamic system
  }

  const grep = mochaOpts.grep || (mochaOpts as Record<string, unknown>).g;
  if (grep) {
    mochaOpts.grep = new RegExp(String(grep), "i");
  }

  // Create the mocha test
  const mocha = new Mocha(mochaOpts);

  const testsRoot = path.resolve(__dirname, "..");

  return new Promise((c, e) => {
    glob("**/**.test.js", { cwd: testsRoot }, async (err, files) => {
      if (err) {
        return e(err);
      }

      // Add files to the test suite
      files.forEach((f) => mocha.addFile(path.resolve(testsRoot, f)));
      try {
        // Run the mocha test
        mocha.run((failures) => {
          if (failures > 0) {
            e(new Error(`${failures} tests failed.`));
          } else {
            c();
          }
        });
      } catch (err) {
        console.error(err);
        e(err);
      } finally {
        if (nyc) {
          nyc.writeCoverageFile();
          await nyc.report();
        }
      }
    });
  });
}

function setupCoverage() {
  const NYC = require("nyc");
  const nyc = new NYC({
    cwd: path.join(__dirname, "..", "..", "..", ".."),
    exclude: ["**/test/**", ".vscode-test/**"],
    reporter: ["html"],
    all: true,
    instrument: true,
    hookRequire: true,
    hookRunInContext: true,
    hookRunInThisContext: true,
  });

  nyc.reset();
  nyc.wrap();

  return nyc;
}
