CFG_text
  = members:(@member) {
  	let res = {};
    res[members.name] = members.value;
    res["loc"] = members["loc"];
    return res;
  }

begin_object    = ws "{" ws
end_object      = ws "}" ws
name_separator  = ws
value_separator = ws

ws "whitespace" = [ \t\n\r]*


Comment "comment"
  = MultiLineComment
  / SingleLineComment

// ----- 3. Values -----

value
  = object
  / string

// ----- 4. Objects -----

member
  = name:key Comment? name_separator Comment? value:value Comment?{
      return { name: name, value: value, loc: location() };
    }

object
  = begin_object
    (Comment ws)?
    members:(
      head:member
      tail:(value_separator @member)*
      {

        var result = [head].concat(tail).map(function(element) {
          let res = {};
          res["name"] = element.name;
          res["value"] = element.value;
          res["location"] = location();
          return res;
        });

        return result;
      }
    )
    Comment?
    end_object
    { 
      members["loc"] = location();
      return members !== null ? members: {}; }


// ----- 7. Strings -----

string "string"
  = quotation_mark chars:char* quotation_mark { return chars.join(""); }

key "key"
  = quotation_mark chars:char+ quotation_mark { return chars.join(""); }

char
  = unescaped
  / escape
    sequence:(
        '"'
      / "\\"
      / "/"
      / "b" { return "\b"; }
      / "f" { return "\f"; }
      / "n" { return "\n"; }
      / "r" { return "\r"; }
      / "t" { return "\t"; }
      / "u" digits:$(HEXDIG HEXDIG HEXDIG HEXDIG) {
          return String.fromCharCode(parseInt(digits, 16));
        }
    )
    { return sequence; }

LineTerminator
  = [\n\r\u2028\u2029]

SourceCharacter
  = .

MultiLineComment
  = "/*" (!"*/" SourceCharacter)* "*/"

MultiLineCommentNoLineTerminator
  = "/*" (!("*/" / LineTerminator) SourceCharacter)* "*/"

SingleLineComment
  = "//" (!LineTerminator SourceCharacter)*

escape
  = "\\"

quotation_mark
  = '"'

unescaped
  = [^\0-\x1F\x22\x5C]

// ----- Core ABNF Rules -----

// See RFC 4234, Appendix B (http://tools.ietf.org/html/rfc4234).
DIGIT  = [0-9]
HEXDIG = [0-9a-f]i