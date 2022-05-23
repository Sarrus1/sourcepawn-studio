{{
  //import { readInclude } from "./readInclude";
  import { readEnum } from "./readEnum";
  import { readDefine } from "./readDefine";
  import { readMacro } from "./readMacro";
  import { readTypedef } from "./readTypedef";
  import { readTypeset } from "./readTypeset";
  import { readVariable } from "./readVariable";
  import { readFunctionAndMethod } from "./readFunctionAndMethod";
  import { readEnumStruct } from "./readEnumStruct";
  import { readMethodmap } from "./readMethodmap";
  import * as interfaces from "./interfaces";

  var TYPES_TO_PROPERTY_NAMES = {
    CallExpression:   "callee",
    MemberExpression: "object",
  };

  function filledArray(count, value) {
    return Array.apply(null, new Array(count))
      .map(function() { return value; });
  }

  function extractOptional(optional, index) {
    return optional ? optional[index] : null;
  }

  function extractList(list, index) {
    return list.map((e) => e[index]);
  }

  function buildList(head, tail, index) {
    return [head].concat(extractList(tail, index));
  }
  
  function buildListWithDoc(head, tail, index) {
    let docs = extractList(tail, index - 1);
    return [head].concat(extractList(tail, index)).map((e, i) => {
      if (docs[i]) {
        e.doc = docs[i];
      }
      return e;
    });
  }

  function buildComment(content) {
    return content
      .flat()
      .filter((e) => e !== undefined)
      .join("");
  }

  function buildNestedArray(content) {
    return content.flat(Infinity).join("")
  }

  function buildBinaryExpression(head, tail) {
    return tail.reduce(function(result, element) {
      return {
        type: "BinaryExpression",
        operator: element[1],
        left: result,
        right: element[3]
      };
    }, head);
  }

  function buildLogicalExpression(head, tail) {
    return tail.reduce(function(result, element) {
      return {
        type: "LogicalExpression",
        operator: element[1],
        left: result,
        right: element[3]
      };
    }, head);
  }

  function optionalList(value) {
    return value !== null ? value : [];
  }
}}
{
  const args = this.args;
}

Start
  = program:Program __ { return program; }

// ----- A.1 Lexical Grammar -----

SourceCharacter
  = .

WhiteSpace "whitespace"
  = "\t"
  / "\v"
  / "\f"
  / " "

LineTerminator
  = [\n\r\u2028\u2029]

LineTerminatorSequence "end of line"
  = content:(("\n"
  / "\r\n"
  / "\r"
  / "\u2028"
  / "\u2029")
  PreprocessorStatement?)
  {
    return {
      type:"LineTerminatorSequence",
      content
    }
  }

Comment "comment"
  = MultiLineComment
  / SingleLineComment

MultiLineComment
  = "/*" txt:$(!"*/" SourceCharacter)* "*/" pre:PreprocessorStatement?
  {
    return {
      type: "MultiLineComment",
      text: txt,
      pre
    };
  }

MultiLineCommentNoLineTerminator
  = "/*" txt:$(!("*/" / LineTerminator) SourceCharacter)* "*/" pre:PreprocessorStatement?
  {
    return {
      type: "MultiLineCommentNoLineTerminator",
      text: txt,
      pre
    };
  }

SingleLineComment
  = "//" txt:$(!LineTerminator SourceCharacter)* LineTerminatorSequence?
  {
    return {
      type: "SingleLineComment",
      text: txt
    };
  }

Identifier
  = !(ReservedWord !IdentifierPart) name:IdentifierName
  {
    args.fileItems.pushToken(args, name);
    return name;
  }

TypeIdentifier
  = !(TypeReservedKeyword !IdentifierPart) name:IdentifierName 
  {
    args.fileItems.pushToken(args, name);
    return name; 
  }

IdentifierName "identifier"
  = head:IdentifierStart tail:IdentifierPart* 
  {
    return {
      id: head + tail.join(""),
      loc: location()
    };
  }

IdentifierStart
  = [_A-Za-z]

IdentifierPart
  = [_A-Za-z0-9]

TypeReservedKeyword
  = Keyword
  / NullLiteral
  / BooleanLiteral
  / SizeofLiteral
  / StructReservedKeywords

StructReservedKeywords
  = ExtensionToken
  / PluginToken
  / PlversToken
  / SharedPluginToken

ReservedWord
  = Keyword
  / TypeKeyword
  / NullLiteral
  / BooleanLiteral
  / SizeofLiteral

Keyword
  = AcquireToken
  / AsToken
  / AssertToken
  / BuiltinToken
  / BreakToken
  / CaseToken
  / CastToToken
  / CatchToken
  / ContinueToken
  / ConstToken
  / DeleteToken
  / DeclToken
  / DefaultToken
  / DefinedToken
  / DoubleToken
  / DoToken
  / DotDotDotToken
  / ElseToken
  / EnumToken
  / EnumStructToken
  / ExitToken
  / ExplicitToken
  / FinallyToken
  / ForeachToken
  / ForwardToken
  / ForToken
  / FuncenumToken
  / FunctagToken
  / FunctionToken
  / GotoToken
  / IfToken
  / ImportToken
  / ImplicitToken
  / InterfaceToken
  / InToken
  / LetToken
  / MethodmapToken
  / NamespaceToken
  / NativeToken
  / NewToken
  / NullableToken
  / ObjectToken
  / OperatorToken
  / ReadonlyToken
  / ReturnToken
  / SealedToken
  / StaticToken
  / StaticAssertToken
  / StockToken
  / StructToken
  / SwitchToken
  / SizeofToken
  / ThisToken
  / ThrowToken
  / TryToken
  / TypedefToken
  / TypeSetToken
  / TypeofToken     
  / UnionToken        
  / UsingToken        
  / VarToken
  / VariantToken
  / ViewAsToken
  / VirtualToken
  / VolatileToken
  / WithToken
  / WhileToken
  / PackageToken
  / PublicToken
  / PrivateToken
  / PropertyToken
  / ProtectedToken

TypeKeyword
  = AnyToken
  / CharToken
  // / FloatToken
  / IntToken 
  / Int8Token
  / Int16Token
  / Int32Token
  / Int64Token
  / IntnToken
  / UintToken
  / Uint8Token   
  / Uint16Token
  / Uint32Token
  / Uint64Token
  / UintnToken
  / VoidToken

Literal
  = NullLiteral
  / BooleanLiteral
  / NumericLiteral
  / StringLiteral
  / DotDotDotToken
  / SizeofLiteral

SizeofLiteral
  = SizeofToken __p id:Expression __
  { 
    return { 
      type: "sizeof",
      value: id 
    }; 
  }
  /
  SizeofToken __ "(" id:Expression ")" __
  { 
    return { 
      type: "sizeof",
      value: id 
    }; 
  }

NullLiteral
  = NullToken { return { type: "Literal", value: null }; }

BooleanLiteral
  = TrueToken  { return { type: "Literal", value: true  }; }
  / FalseToken { return { type: "Literal", value: false }; }

NumericLiteral "number"
  = literal:HexIntegerLiteral !(IdentifierStart / DecimalDigit) {
      return literal;
    }
  / literal:DecimalLiteral !(IdentifierStart / DecimalDigit) {
      return literal;
    }
  / literal:BinaryLiteral !(IdentifierStart / DecimalDigit) {
    return literal;
    }

DecimalLiteral
  = DecimalIntegerLiteral "." DecimalDigit* ExponentPart? {
      return { type: "Literal", value: parseFloat(text()) };
    }
  / "." DecimalDigit+ ExponentPart? {
      return { type: "Literal", value: parseFloat(text()) };
    }
  / DecimalIntegerLiteral ExponentPart? {
      return { type: "Literal", value: parseFloat(text()) };
    }

DecimalIntegerLiteral
  = "0"
  / NonZeroDigit DecimalDigit*

DecimalDigit
  = [0-9]

NonZeroDigit
  = [1-9]

ExponentPart
  = ExponentIndicator SignedInteger

ExponentIndicator
  = "e"i

SignedInteger
  = [+-]? DecimalDigit+

HexIntegerLiteral
  = "0x"i digits:$HexDigit+ {
      return { type: "Literal", value: parseInt(digits, 16) };
     }

HexDigit
  = [0-9a-f_]i

BinaryLiteral
  = "0b"i digits:$BinaryDigit+ {
      return { type: "Literal", value: parseInt(digits, 2) };
     }

BinaryDigit
  = [0-1_]

StringLiteral "string"
  = '"' chars:DoubleStringCharacter* '"' {
      return { type: "Literal", value: chars.join("") };
    }
  / "'" chars:SingleStringCharacter* "'" {
      return { type: "Literal", value: chars.join("") };
    }

DoubleStringCharacter
  = !('"' / "\\" / LineTerminator) SourceCharacter { return text(); }
  / "\\" sequence:EscapeSequence { return sequence; }
  / LineContinuation

SingleStringCharacter
  = !("'" / "\\" / LineTerminator) SourceCharacter { return text(); }
  / "\\" sequence:EscapeSequence { return sequence; }
  / LineContinuation

LineContinuation
  = "\\" WhiteSpace* LineTerminatorSequence { return ""; }

EscapeSequence
  = CharacterEscapeSequence
  / "0" !DecimalDigit { return "\0"; }
  / HexEscapeSequence
  / UnicodeEscapeSequence

CharacterEscapeSequence
  = SingleEscapeCharacter
  / NonEscapeCharacter

SingleEscapeCharacter
  = "'"
  / '"'
  / "\\"
  / "b"  { return "\b"; }
  / "f"  { return "\f"; }
  / "n"  { return "\n"; }
  / "r"  { return "\r"; }
  / "t"  { return "\t"; }
  / "v"  { return "\v"; }

NonEscapeCharacter
  = !(EscapeCharacter / LineTerminator) SourceCharacter { return text(); }

EscapeCharacter
  = SingleEscapeCharacter
  / DecimalDigit
  / "x"
  / "u"

HexEscapeSequence
  = "x" digits:$(HexDigit HexDigit) {
      return String.fromCharCode(parseInt(digits, 16));
    }

UnicodeEscapeSequence
  = "u" digits:$(HexDigit HexDigit HexDigit HexDigit) {
      return String.fromCharCode(parseInt(digits, 16));
    }

// Tokens

AcquireToken      = "acquire"
AnyToken          = "any"
AsToken           = "as"
AssertToken       = "assert"
BuiltinToken      = "builtin"
BreakToken        = "break"
CaseToken         = "case"
CatchToken        = "catch"
CastToToken       = "cast_to"
CharToken         = "char"
ConstToken        = "const"
ContinueToken     = "continue"
DefaultToken      = "default"
DefinedToken      = "defined"
DeleteToken       = "delete"
DoToken           = "do"
DoubleToken       = "double"
DeclToken		      = "decl"
ElseToken         = "else"
EnumToken         = "enum"
EnumStructToken   = EnumToken __p StructToken
ExitToken         = "exit"
ExplicitToken     = "explicit"
ExtensionToken    = "Extension"
FalseToken        = "false"
FuncenumToken     = "funcenum"
FunctagToken      = "functag"
FloatToken        = "float"
ForeachToken      = "foreach"
DotDotDotToken    = "..." {return {id: "...", loc: location()};}
FinallyToken      = "finally"
ForToken          = "for"
ForwardToken      = "forward"
FunctionToken     = "function"
GotoToken         = "goto"
IfToken           = "if"
ImplicitToken     = "implicit"
ImportToken       = "import"
InToken           = "in"
InterfaceToken    = "interface"
IntToken          = "int"
Int8Token         = "int8"
Int16Token        = "int16"
Int32Token        = "int32"
Int64Token        = "int64"
IntnToken         = "intn"
LetToken          = "let"
MethodmapToken    = "methodmap"
NamespaceToken    = "namespace"
NativeToken       = "native"
NewToken          = "new"
NullableToken     = "__nullable__"
NullToken         = "null"
ObjectToken       = "object"
OperatorToken     = "operator" {return {id: "operator", loc: location()};}
PluginToken       = "Plugin"
PlversToken       = "PlVers"
ReturnToken       = "return"
ReadonlyToken     = "readonly"
SealedToken       = "sealed"
SharedPluginToken = "SharedPlugin"
StockToken        = "stock"
StaticToken       = "static"
StaticAssertToken = "static_assert"
SwitchToken       = "switch"
StructToken       = "struct"
SizeofToken       = "sizeof"
ThisToken         = "this"  { args.fileItems.pushToken(args, {id: "this", loc: location()}); }
ThrowToken        = "throw"
TrueToken         = "true"
TryToken          = "try"
TypedefToken      = "typedef"
TypeSetToken      = "typeset"
TypeofToken       = "typeof"
UintToken         = "uint"
Uint8Token        = "uint8"
Uint16Token       = "uint16"
Uint32Token       = "uint32"
Uint64Token       = "uint64"
UintnToken        = "uintn"
UnionToken        = "union"
UsingToken        = "using"
VarToken          = "var"
VariantToken      = "variant"
ViewAsToken       = "view_as"
VirtualToken      = "virtual"
VoidToken         = "void"
VolatileToken     = "volatile"
WithToken         = "with"
WhileToken        = "while"
PAssertToken      = "#assert"
PDefineToken      = "#define"
PElseToken        = "#else"
PElseIfToken      = "#elseif"
PEndIfToken       = "#endif"
PEndInputToken    = "#endinput"
PEndScriptToken   = "#endscript"
PErrorToken       = "#error"
PFileToken        = "#file"
PWarningToken     = "#warning"
PIfToken          = "#if"
PIncludeToken     = "#include"
PLineToken        = "#line"
PEmitToken        = "#emit"
PPragmaToken      = "#pragma"
PTryIncludeToken  = "#tryinclude"
PUndefToken       = "#undef"
PackageToken      = "package"
PublicToken       = "public"
PrivateToken      = "private"
PropertyToken     = "property"
ProtectedToken    = "protected"



// Skipped

__
  = content:(WhiteSpace / (LineContinuation / LineTerminatorSequence) / Comment)*
    {
      return content;
    }

__p "separator"
  = content:(WhiteSpace / (LineContinuation / LineTerminatorSequence) / Comment)+

_p
  = content:(WhiteSpace / MultiLineCommentNoLineTerminator / SingleLineComment)+

_
  = content:(WhiteSpace / MultiLineCommentNoLineTerminator / SingleLineComment)*
    {
      return content;
    }

__doc "trailing doc"
  = WhiteSpace* content:Comment?
    {
      return content;
    }

// Automatic Semicolon Insertion

EOS
  = __ ";"
  / _ SingleLineComment? (LineContinuation / LineTerminatorSequence) 
  / _ &"}"
  / __ EOF

EOF
  = !.

// ----- A.3 Expressions -----

PrimaryExpression
  = TypeIdentifier
  / Literal
  / ArrayLiteral
  / ObjectLiteral
  / ThisToken { return { type: "ThisExpression" }; }
  / "(" __ expression:Expression __ ")" { return expression; }

ArrayLiteral
  = "{" __ elision:(Elision __)? "}" {
      return {
        type: "ArrayExpression",
        elements: optionalList(extractOptional(elision, 0))
      };
    }
  / "{" __ elements:ElementList __ "}" {
      return {
        type: "ArrayExpression",
        elements: elements
      };
    }
  / "{" __ elements:ElementList __ "," __ elision:(Elision __)? "}" {
      return {
        type: "ArrayExpression",
        elements: elements.concat(optionalList(extractOptional(elision, 0)))
      };
    }

ElementList
  = head:(
      elision:(Elision __)? element:Expression {
        return optionalList(extractOptional(elision, 0)).concat(element);
      }
    )
    tail:(
      __ "," __ elision:(Elision __)? element:Expression {
        return optionalList(extractOptional(elision, 0)).concat(element);
      }
    )*
    { return Array.prototype.concat.apply(head, tail); }

Elision
  = "," commas:(__ ",")* { return filledArray(commas.length + 1, null); }

ObjectLiteral
  = "{" __ "}" { return { type: "ObjectExpression", properties: [] }; }
  / "{" __ properties:PropertyNameAndValueList __ "}" {
       return { type: "ObjectExpression", properties: properties };
     }
  / "{" __ properties:PropertyNameAndValueList __ "," __ "}" {
       return { type: "ObjectExpression", properties: properties };
     }
PropertyNameAndValueList
  = head:PropertyAssignment tail:(__ (","/__p)? __ PropertyAssignment)* {
      return buildList(head, tail, 3);
    }

PropertyAssignment
  = key:IdentifierName __ "=" __ value:AssignmentExpression {
      args.fileItems.pushToken(args, key);
      return { type: "Property", key: key, value: value, kind: "init" };
    }

PropertySetParameterList
  = id:Identifier { return [id]; }

MemberExpression
  = head:(
        PrimaryExpression
      / FunctionExpression
      / ViewAsExpression
      / NewToken __ callee:MemberExpression __ args:Arguments {
          return { type: "NewExpression", callee: callee, arguments: args };
        }
      / "char"
    )
    tail:(
        __ "[" __ property:Expression? __ "]" {
          return { property: property, computed: true };
        }
      / __ "." __ property:IdentifierName
      {
        args.fileItems.pushToken(args, property);
        return { property: property, computed: false };
      }
      / __ "::" __ property:IdentifierName
      {
        args.fileItems.pushToken(args, property);
        return { property: property, computed: false };
      }
    )*
    {
      return tail.reduce(function(result, element) {
        return {
          type: "MemberExpression",
          object: result,
          property: element.property,
          computed: element.computed
        };
      }, head);
    }

NewExpression
  = NewToken __ callee:MemberExpression {
      return { type: "NewExpression", callee: callee, arguments: [] };
    }

CallExpression
  = head:(
      callee:MemberExpression {
        return { type: "CallExpression", callee: callee, arguments: args };
      }
    )
    tail:(
        __ args:Arguments {
          return { type: "CallExpression", arguments: args };
        }
      / __ "[" __ property:Expression __ "]" {
          args.fileItems.pushToken(args, property);
          return {
            type: "MemberExpression",
            property: property,
            computed: true
          };
        }
      / __ "." __ property:IdentifierName {
          args.fileItems.pushToken(args, property);
          return {
            type: "MemberExpression",
            property: property,
            computed: false
          };
        }
      / __ "::" __ property:IdentifierName {
          args.fileItems.pushToken(args, property);
          return {
            type: "MemberExpression",
            property: property,
            computed: false
          };
        }
    )*
    {
      return tail.reduce(function(result, element) {
        element[TYPES_TO_PROPERTY_NAMES[element.type]] = result;

        return element;
      }, head);
    }

Arguments
  = "(" __ args:(ArgumentList __)? ")" 
  {
    return optionalList(extractOptional(args, 0));
  }

ArgumentList
  = (TypeIdentifier ":" !":")? "."? head:AssignmentExpression tail:(__ "," __ (TypeIdentifier ":" !":")? "."? AssignmentExpression)* 
  {
    return buildList(head, tail, 3);
  }

LeftHandSideExpression
  = CallExpression
  / NewExpression
  / ViewAsExpression


ViewAsExpression
  = (ViewAsToken "<" _ TypeIdentifier _ ">" "(" __ Expression __")")

PostfixExpression
  = argument:LeftHandSideExpression _ operator:PostfixOperator 
  {
    return {
        type: "UpdateExpression",
        operator: operator,
        argument: argument,
        prefix: false
    };
  }
  / LeftHandSideExpression

AliasOperator
  = MultiplicativeOperator 
  / AdditiveOperator 
  / RelationalOperator
  / EqualityOperator
  / UnaryOperator

PostfixOperator
  = "++"
  / "--"

UnaryExpression
  = PostfixExpression
  / operator:UnaryOperator __ argument:UnaryExpression 
  {
    let type = (operator === "++" || operator === "--")
      ? "UpdateExpression"
      : "UnaryExpression";

    return {
      type: type,
      operator: operator,
      argument: argument,
      prefix: true
    };
  }

UnaryOperator
  = $DeleteToken
  / $VoidToken
  / "++"
  / "--"
  / $("+" !"=")
  / $("-" !"=")
  / "~"
  / "!"

MultiplicativeExpression
  = head:UnaryExpression
  tail:(__ MultiplicativeOperator __ UnaryExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

MultiplicativeOperator
  = $("*" !"=")
  / $("/" !"=")
  / $("%" !"=")

AdditiveExpression
  = head:MultiplicativeExpression
  tail:(__ AdditiveOperator __ MultiplicativeExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

AdditiveOperator
  = $("+" ![+=])
  / $("-" ![-=])
  / DotDotDotToken

ShiftExpression
  = head:AdditiveExpression
  tail:(__ ShiftOperator __ AdditiveExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

ShiftOperator
  = $("<<"  !"=")
  / $(">>>" !"=")
  / $(">>"  !"=")

RelationalExpression
  = head:ShiftExpression
  tail:(__ RelationalOperator __ ShiftExpression)*
  {
    return buildBinaryExpression(head, tail);
    }

RelationalOperator
  = "<="
  / ">="
  / $("<" !"<")
  / $(">" !">")

RelationalExpressionNoIn
  = head:ShiftExpression
  tail:(__ RelationalOperatorNoIn __ ShiftExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

RelationalOperatorNoIn
  = "<="
  / ">="
  / $("<" !"<")
  / $(">" !">")

EqualityExpression
  = (TypeIdentifier ":" !":")? head:RelationalExpression
  tail:(__ EqualityOperator __ (TypeIdentifier ":" !":")? RelationalExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

EqualityExpressionNoIn
  = head:RelationalExpressionNoIn
  tail:(__ EqualityOperator __ RelationalExpressionNoIn)*
  { 
    return buildBinaryExpression(head, tail);
  }

EqualityOperator
  = "=="
  / "!="

BitwiseANDExpression
  = head:EqualityExpression
  tail:(__ BitwiseANDOperator __ EqualityExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

BitwiseANDExpressionNoIn
  = head:EqualityExpressionNoIn
  tail:(__ BitwiseANDOperator __ EqualityExpressionNoIn)*
  { 
    return buildBinaryExpression(head, tail);
  }

BitwiseANDOperator
  = $("&" ![&=])

BitwiseXORExpression
  = head:BitwiseANDExpression
  tail:(__ BitwiseXOROperator __ BitwiseANDExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

BitwiseXORExpressionNoIn
  = head:BitwiseANDExpressionNoIn
  tail:(__ BitwiseXOROperator __ BitwiseANDExpressionNoIn)*
  { 
    return buildBinaryExpression(head, tail);
  }

BitwiseXOROperator
  = $("^" !"=")

BitwiseORExpression
  = head:BitwiseXORExpression
  tail:(__ BitwiseOROperator __ BitwiseXORExpression)*
  { 
    return buildBinaryExpression(head, tail);
  }

BitwiseORExpressionNoIn
  = head:BitwiseXORExpressionNoIn
  tail:(__ BitwiseOROperator __ BitwiseXORExpressionNoIn)*
  { 
    return buildBinaryExpression(head, tail);
  }

BitwiseOROperator
  = $("|" ![|=])

LogicalANDExpression
  = head:BitwiseORExpression
  tail:(__ LogicalANDOperator __ BitwiseORExpression)*
  { 
    return buildLogicalExpression(head, tail);
  }

LogicalANDExpressionNoIn
  = head:BitwiseORExpressionNoIn
  tail:(__ LogicalANDOperator __ BitwiseORExpressionNoIn)*
  { 
    return buildLogicalExpression(head, tail);
  }

LogicalANDOperator
  = "&&"

LogicalORExpression
  = head:LogicalANDExpression
  tail:(__ LogicalOROperator __ LogicalANDExpression)*
  { 
    return buildLogicalExpression(head, tail);
  }

LogicalORExpressionNoIn
  = head:LogicalANDExpressionNoIn
  tail:(__ LogicalOROperator __ LogicalANDExpressionNoIn)*
  { 
    return buildLogicalExpression(head, tail);
  }

LogicalOROperator
  = "||"

ConditionalExpression
  = test:LogicalORExpression __
  "?" __ consequent:AssignmentExpressionNoIn __
  ":" !":" __ alternate:AssignmentExpressionNoIn
  {
    return {
      type: "ConditionalExpression",
      test: test,
      consequent: consequent,
      alternate: alternate
    };
  }
  / LogicalORExpression

ConditionalExpressionNoIn
  = test:LogicalORExpressionNoIn __
  "?" __ consequent:AssignmentExpressionNoIn __
  ":" !":" __ alternate:AssignmentExpressionNoIn
  {
    return {
      type: "ConditionalExpression",
      test: test,
      consequent: consequent,
      alternate: alternate
    };
  }
  / LogicalORExpressionNoIn

AssignmentExpression
  = left:LeftHandSideExpression __
  "=" !"=" __
  (TypeIdentifier ":" !":")?
  right:AssignmentExpression
  {
    return {
      type: "AssignmentExpression",
      operator: "=",
      left: left,
      right: right
    };
  }
  / left:LeftHandSideExpression __
  operator:AssignmentOperator __
  (TypeIdentifier ":" !":")?
  right:AssignmentExpression
  {
    return {
      type: "AssignmentExpression",
      operator: operator,
      left: left,
      right: right
    };
  }
  / ConditionalExpression

AssignmentExpressionNoIn
  = left:LeftHandSideExpression __
  "=" !"=" __
  right:AssignmentExpressionNoIn
  {
    return {
      type: "AssignmentExpression",
      operator: "=",
      left: left,
      right: right
    };
  }
  / left:LeftHandSideExpression __
  operator:AssignmentOperator __
  right:AssignmentExpressionNoIn
  {
    return {
      type: "AssignmentExpression",
      operator: operator,
      left: left,
      right: right
    };
  }
  / ConditionalExpressionNoIn

AssignmentOperator
  = "*="
  / "/="
  / "%="
  / "+="
  / "-="
  / "<<="
  / ">>="
  / ">>>="
  / "&="
  / "^="
  / "|="

Expression
  = (TypeIdentifier ":" !":")? head:AssignmentExpression tail:(__ "," __ (TypeIdentifier ":" !":")? AssignmentExpression)*
  {
    return tail.length > 0
      ? { type: "SequenceExpression", expressions: buildList(head, tail, 3) }
      : head;
  }

ExpressionNoIn
  = head:AssignmentExpressionNoIn tail:(__ "," __ AssignmentExpressionNoIn)*
  {
    return tail.length > 0
      ? { type: "SequenceExpression", expressions: buildList(head, tail, 3) }
      : head;
  }


// ----- A.4 Statements -----

Statement
  = Block
  / EmptyStatement
  / LocalVariableDeclaration
  / ExpressionStatement
  / IfStatement
  / IterationStatement
  / ContinueStatement
  / BreakStatement
  / ReturnStatement
  / LabelledStatement
  / SwitchStatement
  / MacroCallStatement
  / IncludeStatement
  / PropertyToken

DefineStatement
  = content:DefineStatementNoDoc doc:__doc
  {
    const res: interfaces.DefineStatement = {
      type: "DefineStatement",
      id: content.id,
      loc: content.loc,
      value: content.value,
      doc
    }
    readDefine(args, res);
    return res;
  }

DefineStatementNoDoc
  = PDefineToken _p id:IdentifierName _ value:$(AssignmentExpression)
  {
    return {
      id,
      loc: location(),
      value
    };
  }
  /
  PDefineToken _p id:IdentifierName _ LineTerminator
  {
    return {
      id,
      loc: location(),
      value: null
    };
  }

IncludeStatement
  = PIncludeToken _ pathLoc:IncludePath
  {
    const res: interfaces.IncludeStatement = {
      type: "IncludeStatement",
      path: pathLoc.path,
      loc: pathLoc.loc
    };
    //readInclude(args, res);
    return res;
  }

IncludePath 
  = "<" path:$([A-Za-z0-9\-_\/.]+) ">"
  { 
    return {
      path,
      loc: location()
    };
  }
  /
  "\"" path:$([A-Za-z0-9\-_\/.]+) "\""
  { 
    return {
      path,
      loc: location()
    };
  }

PragmaStatement
  = PPragmaToken _ value:(
      !("\\" / LineTerminator) SourceCharacter { return text(); }
      / "\\" sequence:EscapeSequence { return sequence; }
      / LineContinuation
    )+
  { 
    return {
      type:"PragmaValue",
      value: value ? value.join("") : null
    };
  }

OtherPreprocessorStatement
  = name:(
  PAssertToken
  / PElseToken
  / PElseIfToken
  / PEndIfToken
  / PEndInputToken
  / PEndScriptToken
  / PErrorToken
  / PFileToken
  / PWarningToken
  / PIfToken          
  / PLineToken
  / PTryIncludeToken
  / PUndefToken
  / PEmitToken
  ) (!LineTerminator SourceCharacter)* (LineContinuation / LineTerminatorSequence) ?
  {
    return {
      type:"PreprocessorStatement", 
      name
    };
  }

LocalVariableDeclaration
  = content:VariableDeclaration
  {
    const value = {type: "LocalVariableDeclaration", content};
    args.variableDecl.push(value);
    return value;
  }

PreprocessorStatement
  = WhiteSpace* pre:(
  PragmaStatement
  / IncludeStatement
  / DefineStatement
  / MacroStatement
  / OtherPreprocessorStatement
  )
  {
    return pre;
  }
    
Block
  = __ "{" body:(StatementList)? __ "}" 
  {
    return {
      type: "BlockStatement",
      body: body
    };
  }

StatementList
  = head:Statement tail:(Statement)* 
  { 
    return [head].concat(tail);
  }

Initialiser
  = "=" !"=" __ expression:AssignmentExpression
  { 
    return expression; 
  }

InitialiserNoIn
  = "=" !"=" __ expression:AssignmentExpressionNoIn
  { 
    return expression;
  }

EmptyStatement
  = ";" 
  {
    return { 
      type: "EmptyStatement"
    }; 
  }

ExpressionStatement
  = doc:__ !("{") expression:Expression EOS
  {
    return {
      type: "ExpressionStatement",
      expression: expression
    };
  }

IfStatement
  = doc:__ IfToken __ "(" __ test:Expression __ ")"
  consequent:Statement __
  ElseToken
  alternate:Statement
  {
    return {
      type: "IfStatement",
      test: test,
      consequent: consequent,
      alternate: alternate
    };
  }
  / doc:__ IfToken __ "(" __ test:Expression __ ")"
  consequent:Statement
  {
    return {
      type: "IfStatement",
      test: test,
      consequent: consequent,
      alternate: null
    };
  }

MacroCallStatement
  = doc:__ id:Identifier __ 
  Arguments __
  body:Statement
  { 
    return {
      type: "MacroCall",
      id,
      body
    };
  }

IterationStatement
  = doc:__ DoToken __
  body:Statement __
  WhileToken __ "(" __ test:Expression __ ")" EOS
  { 
    return {
      type: "DoWhileStatement",
      body, 
      test
    };
  }
  / doc:__ WhileToken __ "(" __ test:Expression __ ")" __
  body:Statement
  { 
    return {
      type: "WhileStatement",
      test,
      body 
    }; 
  }
  / doc:__ ForToken __
  "(" __
  variableType:(NewToken / TypeIdentifier)? __ declarations:VariableDeclarationList __ ";" __
  test:(Expression __)? ";" __
  update:(Expression __)?
  ")"
  body:Statement
  {
    const init = {
        type: "ForLoopVariableDeclaration",
        declarations,
        variableType
      };
    args.variableDecl.push(init);
    return {
      type: "ForStatement",
      init: init,
      test: extractOptional(test, 0),
      update: extractOptional(update, 0),
      body: body
    };
  }
  / doc:__ ForToken __
  "(" __
  init:(ExpressionNoIn __)? ";" __
  test:(Expression __)? ";" __
  update:(Expression __)?
  ")" __
  body:Statement
  {
    return {
      type: "ForStatement",
      init: extractOptional(init, 0),
      test: extractOptional(test, 0),
      update: extractOptional(update, 0),
      body: body
    };
  }
  / doc:__ ForToken __
  "(" __
  left:LeftHandSideExpression __
  right:Expression __
  ")" __
  body:Statement
  {
    return {
      type: "ForInStatement",
      left: left,
      right: right,
      body: body
    };
  }
  / doc:__ ForToken __
  "(" __
    __ declarations:VariableDeclarationList __
  right:Expression __
  ")" __
  body:Statement
  {
    const left = {
        type: "ForLoopVariableDeclaration",
        declarations,
      };
    args.variableDecl.push(left);
    return {
      type: "ForInStatement",
      left: left,
      right: right,
      body: body
    };
  }

ContinueStatement
  = doc:__ ContinueToken EOS 
  {
    return {
      type: "ContinueStatement",
      label: null
    };
  }
  / doc:__ ContinueToken _ label:Identifier EOS 
  {
    return {
      type: "ContinueStatement",
      label
    };
  }

BreakStatement
  = doc:__ BreakToken EOS
  {
    return {
      type: "BreakStatement",
      label: null
    };
  }
  / doc:__ BreakToken _ label:Identifier EOS
  {
    return {
      type: "BreakStatement",
      label 
    };
  }

ReturnStatement
  = doc:__ ReturnToken EOS __
  {
    return {
      type: "ReturnStatement",
      argument: null
    };
  }
  / doc:__ ReturnToken _ (TypeIdentifier ":" !":")? argument:Expression EOS __
  {
    return {
      type: "ReturnStatement",
      argument
    };
  }

SwitchStatement
  = doc:__ SwitchToken __ "(" __ discriminant:Expression __ ")" __
  cases:CaseBlock
  {
    return {
      type: "SwitchStatement",
      discriminant: discriminant,
      cases: cases
    };
  }

CaseBlock
  = "{" __ clauses:(CaseClauses __)? "}"
  {
    return optionalList(extractOptional(clauses, 0));
  }
  / "{" __
  before:(CaseClauses __)?
  default_:DefaultClause __
  after:(CaseClauses __)? "}"
  {
    return optionalList(extractOptional(before, 0))
      .concat(default_)
      .concat(optionalList(extractOptional(after, 0)));
  }

CaseClauses
  = head:CaseClause tail:(__ CaseClause)*
  { 
    return buildList(head, tail, 1);
  }

CaseClause
  = CaseToken __ test:ExpressionNoIn __ ":" !":" consequent:(__ StatementList)?
  {
    return {
      type: "SwitchCase",
      test: test,
      consequent: optionalList(extractOptional(consequent, 1))
    };
  }

DefaultClause
  = DefaultToken __ ":" !":" consequent:(__ StatementList)?
  {
    return {
      type: "SwitchCase",
      test: null,
      consequent: optionalList(extractOptional(consequent, 1))
    };
  }

LabelledStatement
  = doc:__ label:Identifier __ ":" !":" __ body:Statement
  {
    return {
      type: "LabeledStatement",
      label,
      body
    };
  }

// ----- A.5 Functions and Programs -----

OperatorDeclaration
  = doc:__ content:OperatorDeclarationNoDoc
  {
    readFunctionAndMethod(args, content.accessModifier, content.returnType, content.id, content.loc, doc, content.params, content.body, content.txt);
    return content;
  }

OperatorDeclarationNoDoc
  = accessModifier:FunctionAccessModifiers* (( NativeToken/ForwardToken ) __p )?
  returnType:FunctionReturnTypeDeclaration? id:OperatorToken operator:AliasOperator __
  "(" __ params:FormalParameterList? ")" __ body:OperatorDeclarationBody
  {
    const returnTypeId = returnType ? returnType.id : "";
    id.id = id.id + operator+returnTypeId + params.reduce((prev, curr)=>prev+curr.parameterType.name.id, "");
    return {
      type: "OperatorDeclaration",
      accessModifier,
      returnType,
      loc: location(),
      id,
      operator,
      params,
      body,
      txt:text()
    }
  }

OperatorDeclarationBody
  = body:Block
  {
    return body;
  }
  / ("=" __ Identifier)? ";"
  {
    return undefined;
  }

EnumstructDeclaration
  = doc:__ content:EnumstructDeclarationNoDoc
  {
    const res: interfaces.EnumstructDeclaration = {
      type:"EnumstructDeclaration",
      id: content.id,
      loc: content.loc,
      body: content.body,
      doc
    };
    readEnumStruct(args, res);
    return res;
  }

EnumstructDeclarationNoDoc
  = doc:__ EnumStructToken __p id:Identifier __
  "{" __ body:EnumstructBody __ "}" 
  {
    return {
      id,
      loc: location(),
      body
    };
  }

EnumstructBody
  = body:EnumstructMembers? 
  {
    return body;
  }

EnumstructMembers
  = head:(VariableDeclaration / MethodDeclaration) tail:(VariableDeclaration / MethodDeclaration)*
  {
    return [head].concat(tail);
  }

MacroStatement
  = doc:__ content:MacroStatementNoDoc
  {
    const res: interfaces.MacroStatement = {
      type: "MacroStatement",
      id: content.id,
      loc: content.loc,
      value: content.value,
      doc
    }
    readMacro(args, res);
    return res;
  }

MacroStatementNoDoc
  = PDefineToken _p id:IdentifierName value:(
      !("\\" / LineTerminator) SourceCharacter { return text(); }
      / "\\" sequence:EscapeSequence { return sequence; }
      / LineContinuation
    )+
  {
    return {
      id,
      loc: location(),
      value: buildNestedArray(value)
    };
  }

VariableAccessModifiers
  = declarationType:((PublicToken / StockToken / ConstToken / StaticToken) __p)+ 
  { 
    return declarationType.map(e=>e[0]);
  }

VariableType
  = name:TypeIdentifier modifier:$((":" !":"__)/(( __ ("[]")+)? __p))
  {
    return {
      name,
      modifier
    };
  }

GlobalVariableDeclaration
  = content:VariableDeclaration
  {
    readVariable(args, content as interfaces.VariableDeclaration);
    return content;
  }

VariableDeclaration
  = (
  __ ((DeclToken / NewToken) __p)? 
  accessModifiers:VariableAccessModifiers? 
  variableType:VariableType
  declarations:VariableDeclarationList EOS doc:__doc
  {
    return {
      type: "VariableDeclaration",
      accessModifiers,
      variableType,
      declarations: declarations,
      doc
    };
  }
  )
  /
  (
  __ accessModifiers:VariableAccessModifiers
  declarations:VariableDeclarationListOld EOS doc:__doc
  {
    return {
      type: "VariableDeclaration",
      accessModifiers,
      variableType: null,
      declarations: declarations,
      doc
    };
  }    
  )  
  /
  (
  __ ((DeclToken / NewToken) __p)
  declarations:VariableDeclarationListOld EOS doc:__doc
  {
    return {
      type: "VariableDeclaration",
      accessModifiers: null,
      variableType: null,
      declarations: declarations,
      doc
    };
  }    
  )
  /
  (
  __ ((DeclToken / NewToken) __p)
  accessModifiers:VariableAccessModifiers
  declarations:VariableDeclarationListOld EOS doc:__doc
  {
    return {
      type: "VariableDeclaration",
      accessModifiers,
      variableType: null,
      declarations: declarations,
      doc
    };
  }    
  )

VariableDeclarationList
  = head:VariableInitialisation tail:(__ "," __ VariableInitialisation)* 
  {
    return buildList(head, tail, 3);
  }

VariableDeclarationListOld
  = head:VariableInitialisationOld tail:(__ "," __ VariableInitialisationOld)* 
  {
    return buildList(head, tail, 3);
  }

ArrayInitialer
  = "[" Expression? "]"

VariableInitialisation
  = doc:__ (TypeIdentifier":" !":")? id:Identifier _ arrayInitialer:$(ArrayInitialer __)* init:Initialiser? 
  {
    return {
      type: "VariableDeclarator",
      id,
      arrayInitialer,
      init: extractOptional(init, 1)
    };
  }

VariableInitialisationOld
  = doc:__ (TypeIdentifier":" !":")? id:Identifier arrayInitialer:ArrayInitialer* init:(__ Initialiser)? 
  {
    return {
      type: "VariableDeclaratorS",
      id,
      init: extractOptional(init, 1)
    };
  }

EnumDeclaration
  = doc:__ content:EnumDeclarationNoDoc
  {
    const res: interfaces.EnumDeclaration = {
      type: "EnumDeclaration",
      id: content.id,
      loc: content.loc,
      body: content.body,
      doc: content.doc
    };
    readEnum(args, res);
    return res;
  }

EnumDeclarationNoDoc
  = EnumToken id:(__p Identifier)? (":" !":"__)?
  (__ "(" AssignmentOperator __ AssignmentExpression __ ")")? __
  "{" __ body:EnumBody? "}" EOS
  { 
    return {
      id: extractOptional(id, 1),
      loc: location(),
      body
    };
  }

EnumBody
  = head:EnumMemberDeclaration tail:(EnumMemberDeclaration)* __
  {
    return [head].concat(tail);
  }
 
EnumMemberDeclaration
  = (TypeIdentifier (":" !":"__))? name:VariableInitialisation doc:((__ ",")? __doc)
  {
    return {
      id: name.id,
      doc: doc[1]
    };
  }

FunctagDeclaration
  = doc:__ content:FunctagDeclarationNoDoc
  {
    const res: interfaces.FunctagDeclaration = {
      type: "FunctagDeclaration",
      id: content.id,
      loc: content.loc,
      body: content.body,
      doc
    };
    readTypedef(args, res);
    return res;
  }

FunctagDeclarationNoDoc
  = FunctagToken __p accessModifier:FunctionAccessModifiers* 
  returnType:FunctagType? id:Identifier 
  __ "(" __ params:FormalParameterList? ")" __ EOS
  {
    return {
      loc: location(),
      accessModifier,
      body:{
        returnType,
        params
      },
      id,
    };
  }
  /
  FunctagToken __p id:Identifier __p returnType:FunctagType? 
  accessModifier:(PublicToken __)* 
  "(" __ params:FormalParameterList? ")" __ EOS
  {
    return {
      loc: location(),
      accessModifier,
      body:{
        body: returnType,
        params
      },
      id,
    };
  }

FunctagType
  = returnType:TypeIdentifier ":"
  {
    return returnType;
  }

TypedefDeclaration
  = doc:__ content:TypedefDeclarationNoDoc
  {
    const res: interfaces.TypedefDeclaration = {
      type: "TypedefDeclaration",
      id: content.id,
      loc: content.loc,
      body: content.body,
      doc
    };
    readTypedef(args, res);
    return res;
  }

TypedefDeclarationNoDoc
  = TypedefToken __p id:TypeIdentifier __ "=" body:TypedefBody
  {
    return {
      loc: location(),
      id,
      body
    };
  }

TypedefBody
  = doc:__ FunctionToken __ returnType:TypeIdentifier 
  __ "(" __ params:FormalParameterList? ")" __ EOS
  {
  	return {
      returnType,
      params,
      doc
    };
  }

TypesetDeclaration
  = doc:__ content:TypesetDeclarationNoDoc
  {
    const res = {
      type: "TypesetDeclaration",
      id: content.id,
      loc: content.loc,
      body: content.body,
      doc
    };
    readTypeset(args, res as interfaces.TypesetDeclaration);
    return res;
  }

TypesetDeclarationNoDoc
  = TypeSetToken __p id:TypeIdentifier
  __ "{" body:( TypedefBody __ )* "}" EOS
  {
  	return {
      id,
      loc: location(),
      body: extractList(body, 0)
    };
  }

FuncenumDeclaration
  = doc:__ content:FuncenumDeclarationNoDoc
  {
    const res = {
      type: "FuncenumDeclaration",
      id: content.id,
      loc: content.loc,
      body: content.body,
      doc
    };
    readTypeset(args, res as interfaces.FuncenumDeclaration);
    return res;
  }

FuncenumDeclarationNoDoc
  = FuncenumToken __p id:TypeIdentifier
  __ "{" body:( FuncenumBody __ )? "}" EOS
  {
  	return {
      id,
      loc: location(),
      body: body !== undefined ? body[0] : []
    };
  }

FuncenumBody
  = head:FuncenumMemberDeclaration tail:(__ "," FuncenumMemberDeclaration)* ","?
  {
    return buildList(head, tail, 2);
  }

FuncenumMemberDeclaration
  = doc:__ id:TypeIdentifier (":"/_p) accessModifier:"public"
  "(" __ params:FormalParameterList? ")" __
  {
    return {
      type: "FuncenumMemberDeclaration",
      id,
      accessModifier,
      params,
      doc
    }
  }
  /
  doc:__ accessModifier:"public" __
  "("__ params:FormalParameterList? ")" __
  {
    return {
      type: "FuncenumMemberDeclaration",
      id: null,
      accessModifier,
      params,
      doc
    }
  }

StructDeclaration
  = 
  (
    doc:__ accessModifier:FunctionAccessModifiers* 
    StructReservedKeywords ( __ ":" __ / __p ) name:IdentifierName __ "=" __
    infos:ObjectLiteral EOS
    {
      return {
        type: "PluginInfosDeclaration",
        infos,
        name
      }
    }
  )
  /
  (
    doc:__ StructToken __p id:StructReservedKeywords __
    "{" __ (VariableAccessModifiers? (IdentifierName (( __ ("[]")+)? __p )) IdentifierName (__ ";")? __)* "}" __ EOS
  )

MethodmapDeclaration
  = doc:__ content:MethodmapDeclarationNoDoc
  {
    const res: interfaces.MethodmapDeclaration = {
      type:"MethodmapDeclaration",
      id: content.id,
      loc: content.loc,
      inherit: content.inherit,
      body: content.body,
      doc
    }
    readMethodmap(args, res);
    return res;
  }

MethodmapDeclarationNoDoc
  = MethodmapToken __p id:Identifier inherit:MethodmapInherit? __
  "{" body:MethodmapBody __ "}"  (__ ";")?
  {
    return {
      id,
      loc: location(),
      inherit,
      body
    };
  }

MethodmapInherit
  = __ "<" __ id:Identifier
  {
    return id;
  }
  / __p NullableToken

MethodmapBody
  = body:(PropertyDeclaration / MethodDeclaration / MethodmapNativeForwardDeclaration)*
  {
    return body;
  }

PropertyDeclaration
  = doc:__ content:PropertyDeclarationNoDoc
  {
    return {
      type: "PropertyDeclaration",
      propertyType: content.propertyType,
      id: content.id,
      doc,
      loc: content.loc,
      body: content.body,
      txt: content.txt
    };
  }

PropertyDeclarationNoDoc
  = header:PropertyDeclarationHeader __
  "{" __ body:(MethodDeclaration / MethodmapNativeForwardDeclaration)* __ "}"
  {
    return {
      propertyType: header.propertyType,
      id: header.id,
      loc: location(),
      body,
      txt: header.txt
    };
  }

PropertyDeclarationHeader
  = PropertyToken __p propertyType:TypeIdentifier __p id:Identifier
  {
    return {
      propertyType,
      id,
      txt: text()
    };
  }

MethodDeclaration
  = doc:__ content:FunctionDeclarationNoDoc
  {
    return {
      type: "MethodDeclaration",
      accessModifier: content.accessModifier,
      returnType: content.returnType,
      id: content.id,
      doc,
      loc: content.loc,
      params: content.params,
      body: content.body,
      txt: content.txt
    };
  }

MethodmapNativeForwardDeclaration
  = doc:__ content:MethodmapNativeForwardDeclarationNoDoc EOS
  {
    return {
      type: "MethodmapNativeForwardDeclaration",
      accessModifier: content.accessModifier,
      returnType: content.returnType,
      loc: content.loc,
      id: content.id,
      params: content.params,
      doc,
      body: undefined,
      txt: content.txt
    }
  }

MethodmapNativeForwardDeclarationNoDoc
  = accessModifier:FunctionAccessModifiers* token:(NativeToken / ForwardToken) __p
  returnType:FunctionReturnTypeDeclaration? id:Identifier __
  "(" __ params:FormalParameterList? ")"
  {
    accessModifier.push(token)
    return {
      type: "MethodmapNativeForwardDeclaration",
      accessModifier,
      returnType,
      loc: location(),
      id,
      params,
      txt: text()
    }
  }

FunctionAccessModifiers
  = name:(PublicToken / StockToken / StaticToken) __p
  {
    return name;
  }

FunctionReturnTypeDeclaration
  = name:TypeIdentifier ((":" !":"__)/(__("[]")__)/__p)
  {
    return name;
  }

FunctionDeclaration
  = doc:__ content:FunctionDeclarationNoDoc
  {
    readFunctionAndMethod(args, content.accessModifier, content.returnType, content.id, content.loc, doc, content.params, content.body, content.txt);
    return content;
  }

FunctionDeclarationNoDoc
  = header:FunctionDeclarationHeader __
  body:Block
  {
    return {
      type: "FunctionDeclaration",
      accessModifier: header.accessModifier,
      returnType: header.returnType,
      loc: location(),
      id: header.id,
      params: header.params,
      body,
      txt: header.txt
    };
  }

FunctionDeclarationHeader
  = accessModifier:FunctionAccessModifiers* returnType:FunctionReturnTypeDeclaration? id:Identifier __
  "(" __ params:FormalParameterList? ")"
  {
    return {
      accessModifier,
      returnType,
      id,
      params,
      txt: text()
    };
  }

FunctionExpression
  = __ id:(Identifier __)?
  "(" __ params:FormalParameterList? ")"
  "{" __ body:FunctionBody __ "}"
  {
    return {
      type: "FunctionExpression",
      id: extractOptional(id, 0),
      params,
      body: body
    };
  }

ParameterTypeDeclaration
  = name:TypeIdentifier? modifier:$((":" !":"__)/(__(("["Expression?"]")+/"&")__)/__p )
  { 
    return {
      name, 
      modifier
    };
  }

ParameterDeclarationType
  = declarationType:ConstToken __p 
  { 
    return declarationType;
  }

ParameterDeclaration
 = (
  declarationType:ParameterDeclarationType?
  "&"parameterType:ParameterTypeDeclaration? 
  id:(Identifier/DotDotDotToken)
  (__"[" property:Expression? "]"__ / DotDotDotToken)*
  init:(__ Initialiser)?
  {
    return {
      type: "ParameterDeclaration",
      declarationType,
      parameterType: null,
      init,
      id
    };
  }
  )
  /
  (
  declarationType:ParameterDeclarationType? 
  parameterType:ParameterTypeDeclaration? 
  id:(Identifier/DotDotDotToken)
  (__"[" property:Expression? "]"__ / DotDotDotToken)*
  init:(__ Initialiser)?
  {
    return {
      type: "ParameterDeclaration",
      declarationType,
      parameterType,
      init,
      id
    };
  }
  )
  /
  (
  declarationType:ParameterDeclarationType? 
  id:(Identifier/DotDotDotToken)
  (__"[" property:Expression? "]"__ / DotDotDotToken)*
  init:(__ Initialiser)?
  {
    return {
      type: "ParameterDeclaration",
      declarationType,
      parameterType: null,
      init,
      id
    };
  }
  )
  /
  (
  declarationType:ParameterDeclarationType? 
  parameterType:TypeIdentifier id:(DotDotDotToken{ return {id:"...", loc: location()}; })
  {
    return {
      type: "ParameterDeclaration",
      declarationType,
      parameterType,
      init: null,
      id
    };
  }
  )

FormalParameterList
  = head:ParameterDeclaration tail:(__ "," __ ParameterDeclaration)* __
  {
    return buildList(head, tail, 3);
  }

FunctionBody
  = body:(StatementList)? 
  {
    return {
      type: "BlockStatement",
      body: optionalList(body)
    };
  }

NativeForwardDeclaration
  = doc:__ content:NativeForwardDeclarationNoDoc
  {
    readFunctionAndMethod(args, content.accessModifier, content.returnType, content.id, content.loc, doc, content.params, null, content.txt);
    return content;
  }

NativeForwardDeclarationNoDoc
  = accessModifier:FunctionAccessModifiers* token:(NativeToken / ForwardToken) __p
  returnType:FunctionReturnTypeDeclaration? id:Identifier __
  "(" __ params:FormalParameterList? ")" ( __ "=" __ Identifier)? EOS
  {
    if (accessModifier) {
      accessModifier.push(token);
    }
    return {
      type: "NativeForwardDeclaration",
      accessModifier,
      returnType,
      loc: location(),
      id,
      params,
      txt: text()
    };
  }

// Take care of weird declaration in handles.inc
UsingDeclaration
 = doc:__ "using" [^\n;]+ ";"

Program
  = PreprocessorStatement? body:SourceElements? 
  {
    return {
      type: "Program",
      body: optionalList(body)
    };
  }

SourceElements
  = head:SourceElement tail:(SourceElement)* 
  {
    return [head].concat(tail);
  }

SourceElement 
  = 
  FunctionDeclaration
  / OperatorDeclaration
  / EnumDeclaration
  / EnumstructDeclaration
  / FuncenumDeclaration
  / FunctagDeclaration
  / UsingDeclaration
  / NativeForwardDeclaration
  / MethodmapDeclaration
  / TypedefDeclaration
  / TypesetDeclaration
  / StructDeclaration
  / GlobalVariableDeclaration