import sys

with open("src/core/ascii_export.rs", "r") as f:
    content = f.read()

content = content.replace(
    "let mut options = ExportOptions::default();\n        options.max_width = 5;",
    "let options = ExportOptions { max_width: 5, ..Default::default() };"
)
content = content.replace(
    "let mut options = ExportOptions::default();\n        options.max_width = 3;",
    "let options = ExportOptions { max_width: 3, ..Default::default() };"
)

with open("src/core/ascii_export.rs", "w") as f:
    f.write(content)
