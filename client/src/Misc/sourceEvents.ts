import { CompletionItem } from "vscode";

import { sourceGenericEvents } from "./sourceEvents/sourceGeneric";
import { csgoEvents } from "./sourceEvents/csgoEvents";
import { bmEvents } from "./sourceEvents/bmEvents";

export const events: CompletionItem[] = [].concat(
	sourceGenericEvents,
	csgoEvents,
	bmEvents
)