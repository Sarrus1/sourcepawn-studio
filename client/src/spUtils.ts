import {MarkdownString} from "vscode";

export function description_to_md(description: string): MarkdownString {
	description = description.replace(/([^.])(\.) *[\n]+(?:\s*([^@\s.]))/gm, '$1. $3').replace(/\s+\*\s*/gm, "\n\n");
	// Make all @ nicer
	description = description.replace(
		/\s*(@[A-z]+)\s+/gm,
		"\n\n_$1_ "
	);
	// Make the @params nicer
	description = description.replace(
		/(\_@param\_) ([A-z0-9_.]+)\s*/gm,
		"$1 `$2` — "
	);

	// Format other functions which are referenced in the description
	description = description.replace(/([A-z0-9_]+\([A-z0-9_ \:]*\))/gm, "`$1`");
	return new MarkdownString(description);
}