# SyntaxNodes generation rules

**This is very inspired from the awesome list of tutorials by Matklad**

Questions:

1. What tokens should be in the SyntaxKind enum?
   1. Should semicolons and autosemicolons be in it?

Notes:

# Rules For building the nodes:

- Have some known fields, for instance "Attributes", "Name", "Visibility", etc.
- For each field of the grammar, anonymous or not, impl a trait.
  - If it's a name field, make it `impl ast::HasName for Enum {}` **NOT SURE ABOUT THAT**
  - if it's a token field, i.e `enum`, make it a `pub fn enum_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![enum]) }`
  - If it's an another node, make it a `pub fn variant_list(&self) -> Option<VariantList> { support::child(&self.syntax) }`
  - If it's multiple, make it a `children.`
