import * as peggy from "peggy";
import { readFileSync } from "fs";

const text = readFileSync("./cfg.pegjs").toString("utf-8");
const parser = peggy.generate(text);
