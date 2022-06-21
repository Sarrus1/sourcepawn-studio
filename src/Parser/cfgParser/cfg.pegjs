Start
  = __ keyvalues:KeyValue* {
      return keyvalues;
    }

KeyValue
  = key:Key __ value:(Value / Section) __ {
      return {
        key,
        value
      };
    }

Key "key"
  = key:QuotedString
  {
    return {
      loc: location(),
      key
    };
  }

Value "value"
  = value:QuotedString
  {
    return {
      loc: location(),
      value
    };
  }

Section "section"
  = "{" __ keyvalues:KeyValue* "}" {
      return keyvalues;
    }

QuotedString "string"
  = '"' chars:DoubleStringCharacter* '"' {
      return chars.join("");
    }

DoubleStringCharacter
  = !('"' / "\\") SourceCharacter { return text(); }
  / "\\" sequence:CharacterEscapeSequence { return sequence; }

CharacterEscapeSequence
  = SingleEscapeCharacter
  / char:NonEscapeCharacter { return "\\" + char; }

SingleEscapeCharacter
  = '"'
  / "\\"
  / "n"  { return "\n"; }
  / "r"  { return "\r"; }
  / "t"  { return "\t"; }

NonEscapeCharacter
  = !SingleEscapeCharacter SourceCharacter { return text(); }

SourceCharacter
  = .

WhiteSpace "whitespace"
  = "\t"
  / "\v"
  / "\f"
  / " "

LineTerminator
  = [\n\r]

LineTerminatorSequence "end of line"
  = "\n"
  / "\r\n"
  / "\r"

Comment "comment"
  = comment:(MultiLineComment
  / SingleLineComment)
  {
    return comment;
  }

MultiLineComment
  = value:("/*" (!"*/" SourceCharacter)* "*/")
  {
    return {
      type: "MultiLineComment",
      loc: location(),
      value
    };
  }

MultiLineCommentNoLineTerminator
  = value:("/*" (!("*/" / LineTerminator) SourceCharacter)* "*/")
  {
    return {
      type: "MultiLineCommentNoLineTerminator",
      loc: location(),
      value
    };
  }

SingleLineComment
  = value:("//" (!LineTerminator SourceCharacter)*)
  {
    return {
      type: "SingleLineComment",
      loc: location(),
      value
    };
  }

__
  = (WhiteSpace / LineTerminatorSequence / Comment)*