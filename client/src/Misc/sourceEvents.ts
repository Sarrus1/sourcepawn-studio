import { CompletionItem } from "vscode";

import { sourceGenericEvents } from "./sourceEvents/sourceGeneric";
import { csgoEvents } from "./sourceEvents/csgoEvents";

export const events: CompletionItem[] = [].concat(
	sourceGenericEvents,
	csgoEvents
)