import json
import pprint as pp
import re
from enum import Enum
from typing import Any, Dict, List, Optional, Set, Tuple


class NodeType(Enum):
    SEQ = "SEQ"
    TOKEN = "TOKEN"
    SYMBOL = "SYMBOL"
    PATTERN = "PATTERN"
    IMMEDIATE_TOKEN = "IMMEDIATE_TOKEN"
    REPEAT1 = "REPEAT1"
    REPEAT = "REPEAT"
    PREC_RIGHT = "PREC_RIGHT"
    STRING = "STRING"
    BLANK = "BLANK"
    CHOICE = "CHOICE"
    OPTIONAL = "OPTIONAL"
    FIELD = "FIELD"
    PREC = "PREC"
    PREC_LEFT = "PREC_LEFT"


class Node:
    name: Optional[str]
    type_: NodeType
    members: List["Node"]
    value: Optional[str]
    content: Optional["Node"]

    def __init__(self, json_obj: Dict[str, Any]) -> None:
        self.type_ = NodeType[json_obj["type"]]
        self.members = []
        if json_obj.get("members", None) is not None:
            for member in json_obj["members"]:
                self.members.append(Node(member))
        self.value = json_obj.get("value", None)
        self.name = json_obj.get("name", None)
        self.content = json_obj.get("content", None)

    def __repr__(self) -> str:
        res = {"type_": self.type_}
        if len(self.members) > 0:
            res["members"] = self.members
        if self.value is not None:
            res["value"] = self.value
        if self.name is not None:
            res["name"] = self.name
        if self.content is not None:
            res["content"] = self.content

        return pp.pformat(res, indent=4)


class SyntaxKinds:
    hardcoded = {"using __intrinsics__.Handle"}
    punct_mapping = {
        ";": "SEMICOLON",
        ",": "COMMA",
        "(": "L_PAREN",
        ")": "R_PAREN",
        "{": "L_CURLY",
        "}": "R_CURLY",
        "[": "L_BRACK",
        "]": "R_BRACK",
        "<": "L_ANGLE",
        ">": "R_ANGLE",
        "@": "AT",
        "#": "POUND",
        "~": "TILDE",
        "?": "QUESTION",
        "$": "DOLLAR",
        "&": "AMP",
        "|": "PIPE",
        "+": "PLUS",
        "++": "INCREMENT",
        "--": "DECREMENT",
        "*": "STAR",
        "/": "SLASH",
        "^": "CARET",
        "%": "PERCENT",
        "_": "UNDERSCORE",
        ".": "DOT",
        "..": "DOT2",
        "...": "DOT3",
        "..=": "DOT2EQ",
        ":": "COLON",
        "::": "COLON2",
        "=": "EQ",
        "==": "EQ2",
        "=>": "FAT_ARROW",
        "!": "BANG",
        "!=": "NEQ",
        "-": "MINUS",
        "->": "THIN_ARROW",
        "<=": "LTEQ",
        ">=": "GTEQ",
        "+=": "PLUSEQ",
        "-=": "MINUSEQ",
        "|=": "PIPEEQ",
        "&=": "AMPEQ",
        "^=": "CARETEQ",
        "/=": "SLASHEQ",
        "*=": "STAREQ",
        "%=": "PERCENTEQ",
        "~=": "TILDEEQ",
        "&&": "AMP2",
        "||": "PIPE2",
        "<<": "SHL",
        ">>": "SHR",
        "<<=": "SHLEQ",
        ">>=": "SHREQ",
        ">>>": "USHR",
    }
    tokens = {"IDENT"}
    punct: Set[str]
    keywords: Set[Tuple[str, str]]
    preproc_stmts: Set[str]
    nodes: List[Tuple[str, Node]]
    literals: Set[str]
    ignored: Set[str]

    def __init__(self) -> None:
        self.punct = set()
        self.keywords = set()
        self.preproc_stmts = set()
        self.nodes = []
        self.literals = set()
        self.ignored = set()

    @staticmethod
    def parse(
        rules: Dict[str, Any], ast_src: Optional["SyntaxKinds"] = None
    ) -> "SyntaxKinds":
        ast_src = SyntaxKinds()
        for k, v in rules.items():
            if k.endswith("_literal") or k == "concatenated_string" or k == "null":
                ast_src.literals.add(k)
            else:
                ast_src.nodes.append((k, Node(v)))
        stack = [rules]
        values = []
        while len(stack) > 0:
            rules = stack.pop()
            if isinstance(rules, dict):
                if rules.get("type", None) == "STRING":
                    values.append(rules.get("value", None))
                for value in rules.values():
                    if isinstance(value, (dict, list)):
                        stack.append(value)
            elif isinstance(rules, list):
                for value in rules:
                    stack.append(value)
        for value in values:
            ast_src.insert(value)
        return ast_src

    @property
    def escaped_punct(self) -> List[Tuple[str, str]]:
        res = []
        for punct in self.punct:
            res.append((punct, self.punct_mapping[punct]))
        return res

    @staticmethod
    def is_keyword(value: str) -> bool:
        return re.match(r"^[_A-Za-z][_A-Za-z0-9]*$", value) is not None

    @staticmethod
    def is_punct(value: str) -> bool:
        return value in SyntaxKinds.punct_mapping

    @staticmethod
    def is_preproc(value: str) -> bool:
        return value.startswith("#")

    def insert(self, value: str) -> None:
        if value in {"'", '"', "\\", "\\>", "//", "/*"} or value.startswith("0"):
            self.ignored.add(value)
        elif self.is_keyword(value):
            if value == "String" or value == "Float":
                escaped = f"old_{value.lower()}"
            else:
                escaped = value
            self.keywords.add((escaped, value))
        elif self.is_punct(value):
            self.punct.add(value)
        elif self.is_preproc(value):
            self.preproc_stmts.add(value)
        elif value in self.hardcoded:
            self.ignored.add(value)
        else:
            raise KeyError(value)

    def make_macro_export(self, output: List[str]):
        output.append("#[macro_export]")
        buffer = []
        for punct, escaped_punct in self.escaped_punct:
            buffer.append(
                f"[{escape_token(punct)}] => {{ $ crate::SyntaxKind::{escaped_punct} }}"
            )
        for escaped, keyword in self.keywords:
            buffer.append(
                f"[{keyword}] => {{ $ crate::SyntaxKind::{escaped.upper()}_KW }}"
            )
        for preproc in self.preproc_stmts:
            preproc = preproc.replace("#", "p")
            buffer.append(
                f"[{preproc}] => {{ $ crate::SyntaxKind::{preproc.upper()} }}"
            )
        buffer.append(f"[hardcoded] => {{ $ crate::SyntaxKind::HARDCODED }}")
        buffer.append(f"[IDENT] => {{ $ crate::SyntaxKind::IDENT }}")
        output.append("macro_rules ! T {" + " ; ".join(buffer) + "}")
        output.append("pub use T;")

    def generate_tokens(self):
        output = [
            """//! Generated by `sourcegen_ast`, do not edit by hand.

use crate::{
    ast::AstToken,
    SyntaxKind::{self, *},
    SyntaxToken,
};"""
        ]
        for token in list(ast_src.tokens.union(ast_src.literals)):
            token_upper = token.upper()
            token = snake_to_pascal(token.lower())
            output.append(
                f"""#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct {token} {{
    pub(crate) syntax: SyntaxToken,
}}

impl std::fmt::Display for {token} {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        std::fmt::Display::fmt(&self.syntax, f)
    }}
}}

impl AstToken for {token} {{
    fn can_cast(kind: SyntaxKind) -> bool {{ 
        kind == {token_upper} 
    }}
    fn cast(syntax: SyntaxToken) -> Option<Self> {{
        if Self::can_cast(syntax.kind()) {{
            Some(Self {{ syntax }})
        }} else {{
            None
        }}
    }}
    fn syntax(&self) -> &SyntaxToken {{
        &self.syntax
    }}
}}
    """
            )

        with open("src/ast/tokens/generated.rs", "w") as f:
            f.write("\n".join(output))

    def generate_kinds(self):
        KIND_HEADER = [
            "#![allow(bad_style, missing_docs, unreachable_pub)]",
            "#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]",
            "#[repr(u16)]",
            "pub enum SyntaxKind {",
        ]
        output = []
        output.extend(KIND_HEADER)
        for punct, escaped_punct in self.escaped_punct:
            output.append(f"    /// {punct}")
            output.append(f"    {escaped_punct.upper()},")
        for escaped, keyword in self.keywords:
            output.append(f"    /// {keyword},")
            output.append(f"    {escaped.upper()}_KW,")
        for preproc in self.preproc_stmts:
            output.append(f"    /// {preproc},")
            output.append(f"    {preproc.upper().replace('#', 'P')},")
        for literal in self.literals:
            output.append(f"    {literal.upper()},")
        for token in self.tokens:
            output.append(f"    {token.upper()},")
        for node in self.nodes:
            output.append(f"    {node[0].upper()},")
        output.append(f"    HARDCODED,")
        output.append("// Technical kind so that we can cast from u16 safely")
        output.append("    #[doc(hidden)]")
        output.append("    __LAST,")
        output.append("}")

        output.append("")
        self.make_macro_export(output)

        with open("src/ast/syntax_kind/generated.rs", "w") as f:
            f.write("\n".join(output))

    def generate_nodes(self):
        output = [
            """//! Generated by `sourcegen_ast`, do not edit by hand.

#![allow(non_snake_case)]
use crate::{
    ast::{self, support, AstChildren, AstNode},
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T,
};"""
        ]

        for k, node in self.nodes:
            output.append("#[derive(Debug, Clone, PartialEq, Eq, Hash)]")
            output.append(f"pub struct {snake_to_pascal(k)} {{")
            output.append(f"  pub(crate) syntax: SyntaxNode,")
            output.append(f"}}")
            output.append(f"impl {snake_to_pascal(k)} {{")
            for sub_node in node.members:
                if sub_node.type_ == NodeType.STRING:
                    value = sub_node.value
                    escaped = sub_node.value
                    if sub_node.value in SyntaxKinds.hardcoded:
                        value = "hardcoded"
                        escaped = "hardcoded"
                    elif value.startswith("#"):
                        value = value.replace("#", "p")
                        escaped = escaped.replace("#", "p")
                    elif value in SyntaxKinds.punct_mapping:
                        value = SyntaxKinds.punct_mapping[value].lower()
                        if escaped in {"{", "}", "(", ")", "[", "]"}:
                            escaped = f"'{escaped}'"
                    output.append(
                        f"pub fn {value}_token(&self) -> Option<SyntaxToken> {{ support::token(&self.syntax, T![{escaped}]) }}"
                    )
                elif sub_node.type_ == NodeType.SYMBOL:
                    f"pub fn {sub_node.name}(&self) -> Option<{sub_node.name.upper()}> {{ support::child(&self.syntax) }}"
            output.append("}")
            output.append("")

        with open("src/ast/nodes/generated.rs", "w") as f:
            f.write("\n".join(output))

    def __repr__(self) -> str:
        values = {
            "hardcoded": self.hardcoded,
            "punct": self.punct,
            "keywords": self.keywords,
            "preproc_stmts": self.preproc_stmts,
            "nodes": [node[0] for node in self.nodes],
            "literals": self.literals,
            "tokens": self.tokens,
            "ignored": self.ignored,
        }
        return pp.pformat(values, indent=4)


def snake_to_pascal(input: str) -> str:
    return "".join(s.lower().capitalize() for s in input.split("_"))


def escape_kw(input: str) -> str:
    if input.startswith("_"):
        return input[1:]
    return input


def escape_token(input_: str) -> str:
    static_set = {"{", "}", "[", "]", "(", ")"}
    if input_ in static_set:
        return f"'{input_}'"
    return input_


if __name__ == "__main__":
    with open(
        "/Users/charles/Developer/tree-sitter-sourcepawn/src/grammar.json", "r"
    ) as f:
        grammar: Dict[str, Any] = json.load(f)
    ast_src = SyntaxKinds.parse(grammar["rules"])
    ast_src.generate_kinds()
    ast_src.generate_nodes()
    ast_src.generate_tokens()
    # print(ast_src.nodes)
