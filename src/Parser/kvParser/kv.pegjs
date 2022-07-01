{{
  function buildComment(content) {
    return content
      .flat()
      .filter((e) => e !== undefined)
      .join("");
  }
}}

Start
  = doc:__ keyvalues:KeyValue* {
      return {
        doc,
        keyvalues
        };
    }

KeyValue
  = key:Key doc:__ value:(Value / Section) trailDoc:__ {
      return {
      	doc,
        trailDoc,
        key,
        value
      };
    }

Key "key"
  = txt:QuotedString
  {
    return {
      type: "key",
      loc: location(),
      txt
    };
  }

Value "value"
  = txt:QuotedString
  {
    return {
      type: "value",
      loc: location(),
      txt
    };
  }

Section "section"
  = "{" doc:__ keyvalues:KeyValue* "}" {
      return {
        type: "section",
      	doc,
      	keyvalues
      };
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
  = '"'  { return "\\\""; }
  / "\\" { return "\\"; }
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
  = "/*" txt:(!"*/" SourceCharacter)* "*/"
  {
    return {
      type: "MultiLineComment",
      loc: location(),
      value: buildComment(txt)
    };
  }

MultiLineCommentNoLineTerminator
  = "/*" txt:(!("*/" / LineTerminator) SourceCharacter)* "*/"
  {
    return {
      type: "MultiLineCommentNoLineTerminator",
      loc: location(),
      value: buildComment(txt)
    };
  }

SingleLineComment
  = "//" txt:(!LineTerminator SourceCharacter)*
  {
    return {
      type: "SingleLineComment",
      loc: location(),
      value: buildComment(txt)
    };
  }

__
  = txt: (WhiteSpace / LineTerminatorSequence / Comment)*
  {
  	return txt.filter(e=>typeof(e)!=="string");
  }