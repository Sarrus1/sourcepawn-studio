import {
  CompletionItemKind,
  Range,
  CompletionItem,
  SignatureInformation,
  Hover,
  LocationLink,
  Location,
} from "vscode";
import { URI } from "vscode-uri";

import { descriptionToMD } from "../../spUtils";
import { EnumStructItem } from "./spEnumStructItem";
import { SPItem } from "./spItems";

export class EnumStructMemberItem implements SPItem {
  name: string;
  enumStruct: EnumStructItem;
  filePath: string;
  description: string;
  type: string;
  kind = CompletionItemKind.Property;
  parent: string;
  references: Location[];
  range: Range;

  constructor(
    name: string,
    file: string,
    description: string,
    EnumStruct: EnumStructItem,
    range: Range,
    type: string
  ) {
    this.name = name;
    this.filePath = file;
    this.description = description;
    this.enumStruct = EnumStruct;
    this.range = range;
    this.parent = EnumStruct.name;
    this.type = type;
    this.references = [];
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.enumStruct.name,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.filePath),
    };
  }

  toSignature(): SignatureInformation | undefined {
    return undefined;
  }

  toHover(): Hover {
    let enumName = this.enumStruct.name;
    if (enumName == "") {
      return new Hover([
        { language: "sourcepawn", value: this.name },
        descriptionToMD(this.description),
      ]);
    } else {
      return new Hover([
        {
          language: "sourcepawn",
          value: this.enumStruct.name + " " + this.name,
        },
        descriptionToMD(this.description),
      ]);
    }
  }
}
