import re
import subprocess

from tqdm import tqdm

with open("/Users/charles/Developer/tree-sitter-sourcepawn/src/parser.c", "r") as f:
    enum = f.read()

output = []
ops = []
matches = re.findall(r"\[anon_sym_(\w+)\] = \"(.*)\",", enum)
for match in matches:
    output.append(f'{match[0].lower()}: ($) => "{match[1]}",\n')
    ops.append((match[0].lower(), match[1]))


# for n, op in tqdm(enumerate(ops)):
#     with open("grammar.js", "r") as f:
#         grammar = f.readlines()
#     for i, line in enumerate(grammar):
#         pattern = f'(?<!token\\.immediate\\()\\"{re.escape(op[1])}\\"(?!\\))'
#         grammar[i] = re.sub(pattern, f"$.{op[0]}", grammar[i])

#     new_grammar = grammar[:120] + output + grammar[120:]

#     with open("/Users/charles/Developer/tree-sitter-sourcepawn/grammar.js", "w") as f:
#         f.writelines(new_grammar)

#     res = subprocess.run(
#         "npx tree-sitter generate",
#         cwd="/Users/charles/Developer/tree-sitter-sourcepawn/",
#         shell=True,
#         capture_output=True,
#         text=True,
#     )
#     if res.returncode != 0:
#         print(n, op)

with open("grammar.js", "r") as f:
    grammar = f.readlines()

for i, line in enumerate(grammar):
    for op in ops:
        pattern = (
            f'(?<!token\\.immediate\\()(?<!field\\()\\"{re.escape(op[1])}\\"(?!\\))'
        )
        grammar[i] = re.sub(pattern, f"$.{op[0]}", grammar[i])

new_grammar = grammar[:120] + output + grammar[120:]

# with open("/Users/charles/Developer/tree-sitter-sourcepawn/grammar.js", "w") as f:
#     f.writelines(new_grammar)
