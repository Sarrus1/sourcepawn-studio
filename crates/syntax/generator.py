import json
from pprint import pprint
from typing import Any, Dict, List, Tuple

with open("/Users/charles/Developer/tree-sitter-sourcepawn/src/grammar.json", "r") as f:
    grammar: Dict[str, Any] = json.load(f)


def get_all_keys(json_obj, key_search, values=None):
    if values is None:
        values = set()
    if isinstance(json_obj, dict):
        for key, value in json_obj.items():
            if key_search == key and not isinstance(value, (dict, list)):
                values.add(value)
            elif isinstance(value, (dict, list)):
                get_all_keys(value, key_search, values)
    elif isinstance(json_obj, list):
        for item in json_obj:
            get_all_keys(item, key_search, values)
    return values


def snake_to_pascal(input: str) -> str:
    return "".join(s.lower().capitalize() for s in input.split("_"))


def get_rules(json_obj):
    rules = set()
    for k, v in json_obj["rules"].items():
        rules.add(snake_to_pascal(k))
    return rules


def escape_kw(input: str) -> str:
    if input.startswith("_"):
        return input[1:]
    return input


def generate_kinds():
    output = []
    output.append("#![allow(bad_style, missing_docs, unreachable_pub)]")
    output.append("")
    output.append("#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]")
    output.append("#[repr(u16)]")
    output.append("pub enum SyntaxKind {")
    keys_list = []
    for k, v in grammar["rules"].items():
        value: str
        key = escape_kw(k)
        key_upper = key.upper()
        if v.get("type", None) == "STRING":
            if (value := v.get("value", None)) is not None:
                output.append(f"    /// {value}")
                keys_list.append((value, key_upper))
        output.append(f"    {key.upper()},")
        output.append("")
    output.append("    #[doc(hidden)]")
    output.append("    __LAST,")
    output.append("}")

    make_macro_export(keys_list, output)

    with open("src/ast/generated/syntax_kind.rs", "w") as f:
        f.write("\n".join(output))


def escape_token(input_: str) -> str:
    static_set = {"{", "}", "[", "]", "(", ")"}
    if input_ in static_set:
        return f"'{input_}'"
    return input_


def make_macro_export(keys_list: List[Tuple[str, str]], output: List[str]):
    output.append("#[macro_export]")
    buffer = []
    for value, key_upper in keys_list:
        buffer.append(
            f"[{escape_token(value)}] => {{ $ crate::SyntaxKind::{key_upper} }}"
        )
    output.append("macro_rules ! T {" + " ; ".join(buffer) + "}")
    output.append("pub use T;")


def generate_nodes():
    # rules = get_rules(grammar)

    output = []
    output.append("use crate::syntax_node::SyntaxNode;")
    for k, v in grammar["rules"].items():
        if v["type"] == "STRING":
            continue

        output.append("#[derive(Debug, Clone, PartialEq, Eq, Hash)]")
        output.append(f"pub struct {snake_to_pascal(k)} {{")
        output.append(f"  pub(crate) syntax: SyntaxNode,")
        output.append(f"}}")
        output.append("")

    with open("src/ast/generated/nodes.rs", "w") as f:
        f.write("\n".join(output))


if __name__ == "__main__":
    generate_kinds()
    generate_nodes()
