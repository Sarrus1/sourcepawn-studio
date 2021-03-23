import {
	Location,
	Range,
} from "vscode-languageserver/node";


import { URI } from "vscode-uri";

export interface Definition {
  name: string;
	file : URI;
	range: Range

  to_definition_item(file: string): Location;
}

export class FunctionDefinition implements Definition {
  name: string;
	file : URI;
	range : Range;

  constructor(
    name: string,
		file : URI,
		range : Range
  ) {
		this.name = name;
		this.file = file;
		this.range = range;
  }

  to_definition_item(file: string): Location {
		return Location.create("this.file.toString()", {
			start: { line: 2, character: 5 },
			end: { line: 2, character: 6 }
		});
  }
}