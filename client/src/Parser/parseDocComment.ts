import { Parser } from "./spParser";
import { FunctionParam } from "../Providers/spItems";

export function parseDocComment(
  parser: Parser
): {
  description: string;
  params: FunctionParam[];
} {
  if (parser.scratch === undefined) {
    let description = "";
    let params = [];
    return { description, params };
  }
  let description = (() => {
    let lines = [];
    for (let line of parser.scratch) {
      if (/^\s*\/\*\*\s*/.test(line)) {
        //Check if @return or @error
        continue;
      }

      lines.push(line.replace(/^\s*\*\s+/, "\n").replace(/^\s*\/\/\s+/, "\n"));
    }
    return lines.join(" ");
  })();

  const paramRegex = /@param\s+([\w\.]+)\s+(.*)/;
  let params = (() => {
    let params = [];
    let currentParam;
    for (let line of parser.scratch) {
      let match = line.match(paramRegex);
      if (match) {
        // Check if we already have a param description in the buffer.
        // If yes, save it.
        if (currentParam) {
          currentParam.documentation = currentParam.documentation.join(" ");
          params.push(currentParam);
        }
        currentParam = { label: match[1], documentation: [match[2]] };
      } else {
        // Check if it's a return or error description.
        if (/@(?:return|error)/.test(line)) {
          // Check if we already have a param description in the buffer.
          // If yes, save it.
          if (currentParam != undefined) {
            currentParam.documentation = currentParam.documentation.join(" ");
            params.push(currentParam);
            currentParam = undefined;
          }
        } else {
          // Check if we already have a param description in the buffer.
          // If yes, append the new line to it.
          let match = line.match(/\s*(?:\*|\/\/)\s*(.*)/);
          if (match && currentParam) {
            currentParam.documentation.push(match[1]);
          }
        }
      }
    }
    // Add the last param
    if (currentParam != undefined) {
      currentParam.documentation = currentParam.documentation.join(" ");
      params.push(currentParam);
    }

    return params;
  })();

  // Reset the comments buffer
  parser.scratch = [];
  return { description, params };
}
