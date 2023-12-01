import pathlib
import re
from typing import Dict, List


def main(path: pathlib.Path):
    lines = path.read_text().splitlines()
    for i, line in enumerate(lines):
        if line.startswith("enum {"):
            enum = collect_enum(i, lines)
            break
    for i, line in enumerate(lines):
        if line.startswith("static const char * const ts_symbol_names[] = {"):
            ts_symbol_names = collect_kinds(i, lines)
            break
    res = []
    for k, v in enum.items():
        res.append(f"{k} = {v},")
    with open("../crates/syntax/src/generated.rs", "w") as f:
        f.write(
            """#![allow(bad_style, missing_docs, unreachable_pub, unused)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum TSKind {
    """
        )
        f.write("\n    ".join(res))
        f.write(
            """
}

impl From<tree_sitter::Node<'_>> for TSKind {
    fn from(v: tree_sitter::Node<'_>) -> Self {
        unsafe { ::std::mem::transmute(v.kind_id()) }
    }
}
"""
        )


def collect_kinds(idx: int, lines: List[str]) -> Dict[str, int]:
    res = dict()
    for line in lines[idx + 1 :]:
        if line.startswith("};"):
            break
        match_ = re.search(r"\[(\w+)\] = \"(.+)\"", line)
        res[match_.group(1)] = match_.group(2)
    return res


def collect_enum(idx: int, lines: List[str]) -> Dict[str, int]:
    res = dict()
    for line in lines[idx + 1 :]:
        if line.startswith("};"):
            break
        match_ = re.search(r"(\w+) = (\d+)", line)
        res[match_.group(1)] = int(match_.group(2))
    return res


if __name__ == "__main__":
    import sys

    path = pathlib.Path(sys.argv[1])
    if not path.exists():
        sys.exit(1)
    main(path)
