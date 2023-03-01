/*!
A Rust crate providing an implementation of an RFC-compliant `EmailAddress` newtype.

Primarily for validation, the `EmailAddress` type is constructed with `FromStr::from_str` which will raise any
parsing errors. Prior to constructions the functions `is_valid`, `is_valid_local_part`, and `is_valid_domain` may
also be used to test for validity without constructing an instance. This supports all of the RFC ASCII and UTF-8
character set rules, quoted and unquoted local parts but does not yet support all of the productions required for SMTP
headers; folding whitespace, comments, etc.

# Example

The following shoes the basic `is_valid` and `from_str` functions.

```rust
use email_address::*;
use std::str::FromStr;
assert!(EmailAddress::is_valid("user.name+tag+sorting@example.com"));

assert_eq!(
    EmailAddress::from_str("Abc.example.com"),
    Error::MissingSeparator.into()
);
```

The following shows the three format functions used to output an email address.

```rust
use email_address::*;
use std::str::FromStr;

let email = EmailAddress::from_str("johnstonsk@gmail.com").unwrap();

assert_eq!(
    email.to_string(),
    "johnstonsk@gmail.com".to_string()
);

assert_eq!(
    String::from(email.clone()),
    "johnstonsk@gmail.com".to_string()
);

assert_eq!(
    email.as_ref(),
    "johnstonsk@gmail.com"
);

assert_eq!(
    email.to_uri(),
    "mailto:johnstonsk@gmail.com".to_string()
);

assert_eq!(
    email.to_display("Simon Johnston"),
    "Simon Johnston <johnstonsk@gmail.com>".to_string()
);
```


# Specifications

1. RFC 1123: [_Requirements for Internet Hosts -- Application and Support_](https://tools.ietf.org/html/rfc1123),
   IETF,Oct 1989.
1. RFC 3629: [_UTF-8, a transformation format of ISO 10646_](https://tools.ietf.org/html/rfc3629),
   IETF, Nov 2003.
1. RFC 3696: [_Application Techniques for Checking and Transformation of
   Names_](https://tools.ietf.org/html/rfc3696), IETF, Feb 2004.
1. RFC 4291 [_IP Version 6 Addressing Architecture_](https://tools.ietf.org/html/rfc4291),
   IETF, Feb 2006.
1. RFC 5234: [_Augmented BNF for Syntax Specifications: ABNF_](https://tools.ietf.org/html/rfc5234),
   IETF, Jan 2008.
1. RFC 5321: [_Simple Mail Transfer Protocol_](https://tools.ietf.org/html/rfc5321),
   IETF, Oct 2008.
1. RFC 5322: [_Internet Message Format_](https://tools.ietf.org/html/rfc5322), I
   ETF, Oct 2008.
1. RFC 5890: [_Internationalized Domain Names for Applications (IDNA): Definitions and Document
   Framework_](https://tools.ietf.org/html/rfc5890), IETF, Aug 2010.
1. RFC 6531: [_SMTP Extension for Internationalized Email_](https://tools.ietf.org/html/rfc6531),
   IETF, Feb 2012
1. RFC 6532: [_Internationalized Email Headers_](https://tools.ietf.org/html/rfc6532),
   IETF, Feb 2012.

From RFC 5322: §3.2.1. [Quoted characters](https://tools.ietf.org/html/rfc5322#section-3.2.1):

```ebnf
quoted-pair     =   ("\" (VCHAR / WSP)) / obs-qp
```

From RFC 5322: §3.2.2. [Folding White Space and Comments](https://tools.ietf.org/html/rfc5322#section-3.2.2):

```ebnf
FWS             =   ([*WSP CRLF] 1*WSP) /  obs-FWS
                                       ; Folding white space

ctext           =   %d33-39 /          ; Printable US-ASCII
                    %d42-91 /          ;  characters not including
                    %d93-126 /         ;  "(", ")", or "\"
                    obs-ctext

ccontent        =   ctext / quoted-pair / comment

comment         =   "(" *([FWS] ccontent) [FWS] ")"

CFWS            =   (1*([FWS] comment) [FWS]) / FWS
```

From RFC 5322: §3.2.3. [Atom](https://tools.ietf.org/html/rfc5322#section-3.2.3):

```ebnf
atext           =   ALPHA / DIGIT /    ; Printable US-ASCII
                    "!" / "#" /        ;  characters not including
                    "$" / "%" /        ;  specials.  Used for atoms.
                    "&" / "'" /
                    "*" / "+" /
                    "-" / "/" /
                    "=" / "?" /
                    "^" / "_" /
                    "`" / "{" /
                    "|" / "}" /
                    "~"

atom            =   [CFWS] 1*atext [CFWS]

dot-atom-text   =   1*atext *("." 1*atext)

dot-atom        =   [CFWS] dot-atom-text [CFWS]

specials        =   "(" / ")" /        ; Special characters that do
                    "<" / ">" /        ;  not appear in atext
                    "[" / "]" /
                    ":" / ";" /
                    "@" / "\" /
                    "," / "." /
                    DQUOTE
```

From RFC 5322: §3.2.4. [Quoted Strings](https://tools.ietf.org/html/rfc5322#section-3.2.4):

```ebnf
qtext           =   %d33 /             ; Printable US-ASCII
                    %d35-91 /          ;  characters not including
                    %d93-126 /         ;  "\" or the quote character
                    obs-qtext

qcontent        =   qtext / quoted-pair

quoted-string   =   [CFWS]
                    DQUOTE *([FWS] qcontent) [FWS] DQUOTE
                    [CFWS]
```

From RFC 5322, §3.4.1. [Addr-Spec Specification](https://tools.ietf.org/html/rfc5322#section-3.4.1):

```ebnf
addr-spec       =   local-part "@" domain

local-part      =   dot-atom / quoted-string / obs-local-part

domain          =   dot-atom / domain-literal / obs-domain

domain-literal  =   [CFWS] "[" *([FWS] dtext) [FWS] "]" [CFWS]

dtext           =   %d33-90 /          ; Printable US-ASCII
                    %d94-126 /         ;  characters not including
                    obs-dtext          ;  "[", "]", or "\"
```

RFC 3696, §3. [Restrictions on email addresses](https://tools.ietf.org/html/rfc3696#section-3)
describes in detail the quoting of characters in an address.

## Unicode

RFC 6531, §3.3. [Extended Mailbox Address Syntax](https://tools.ietf.org/html/rfc6531#section-3.3)
extends the rules above for non-ASCII character sets.

```ebnf
sub-domain   =/  U-label
    ; extend the definition of sub-domain in RFC 5321, Section 4.1.2

atext   =/  UTF8-non-ascii
    ; extend the implicit definition of atext in
    ; RFC 5321, Section 4.1.2, which ultimately points to
    ; the actual definition in RFC 5322, Section 3.2.3

qtextSMTP  =/ UTF8-non-ascii
    ; extend the definition of qtextSMTP in RFC 5321, Section 4.1.2

esmtp-value  =/ UTF8-non-ascii
    ; extend the definition of esmtp-value in RFC 5321, Section 4.1.2
```

RFC 6532: §3.1 [UTF-8 Syntax and Normalization](https://tools.ietf.org/html/rfc6532#section-3.1),
and §3.2 [Syntax Extensions to RFC 5322](https://tools.ietf.org/html/rfc6532#section-3.2) extend
the syntax above with:

```ebnf
UTF8-non-ascii  =   UTF8-2 / UTF8-3 / UTF8-4

...

VCHAR   =/  UTF8-non-ascii

ctext   =/  UTF8-non-ascii

atext   =/  UTF8-non-ascii

qtext   =/  UTF8-non-ascii

text    =/  UTF8-non-ascii
              ; note that this upgrades the body to UTF-8

dtext   =/  UTF8-non-ascii
```

These in turn refer to RFC 6529 §4. [Syntax of UTF-8 Byte Sequences](https://tools.ietf.org/html/rfc3629#section-4):

> A UTF-8 string is a sequence of octets representing a sequence of UCS
> characters.  An octet sequence is valid UTF-8 only if it matches the
> following syntax, which is derived from the rules for encoding UTF-8
> and is expressed in the ABNF of \[RFC2234\].

```ebnf
   UTF8-octets = *( UTF8-char )
   UTF8-char   = UTF8-1 / UTF8-2 / UTF8-3 / UTF8-4
   UTF8-1      = %x00-7F
   UTF8-2      = %xC2-DF UTF8-tail
   UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
                 %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
   UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
                 %xF4 %x80-8F 2( UTF8-tail )
   UTF8-tail   = %x80-BF
```

Comments in addresses are discussed in RFC 5322 Appendix A.5. [White Space, Comments, and Other
Oddities](https://tools.ietf.org/html/rfc5322#appendix-A.5).

An informal description can be found on [Wikipedia](https://en.wikipedia.org/wiki/Email_address).

*/

use idna::domain_to_unicode;
use regex::Regex;

#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// Error type used when parsing an address.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// An invalid character was found in some component of the address.
    InvalidCharacter,
    /// The separator character between `local-part` and `domain` (character: '@') was missing.
    MissingSeparator,
    /// The `local-part` is an empty string.
    LocalPartEmpty,
    /// The `local-part` is is too long.
    LocalPartTooLong,
    /// The `domain` is an empty string.
    DomainEmpty,
    /// The `domain` is is too long.
    DomainTooLong,
    /// A `sub-domain` within the `domain` is is too long.
    SubDomainTooLong,
    /// Too few `sub-domain`s in `domain`.
    DomainTooFew,
    /// Invalid placement of the domain separator (character: '.').
    DomainInvalidSeparator,
    /// The overall address is too long
    AddressTooLong,
    /// The quotes (character: '"') around `local-part` are unbalanced.
    UnbalancedQuotes,
    /// A Comment within the either the `local-part`, or `domain`, was malformed.
    InvalidComment,
    /// An IP address in a `domain-literal` was malformed.
    InvalidIPAddress,
    InvalidLabel,
}

///
/// Type representing a single email address. This is basically a wrapper around a String, the
/// email address is parsed for correctness with `FromStr::from_str`, which is the only want to
/// create an instance. The various components of the email _are not_ parsed out to be accessible
/// independently.
///
#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "serde_support", derive(Serialize))]
pub struct EmailAddress(String);

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

const LOCAL_PART_MAX_LENGTH: usize = 64;
const DOMAIN_MAX_LENGTH: usize = 254; // see: https://www.rfc-editor.org/errata_search.php?rfc=3696&eid=1690
const SUB_DOMAIN_MAX_LENGTH: usize = 63;
const ADDRESS_MAX_LENGTH: usize = 254;

#[allow(dead_code)]
const CR: char = '\r';
#[allow(dead_code)]
const LF: char = '\n';
const SP: char = ' ';
const HTAB: char = '\t';
const ESC: char = '\\';

const AT: char = '@';
const DOT: char = '.';
const DQUOTE: char = '"';
const LBRACKET: char = '[';
const RBRACKET: char = ']';
#[allow(dead_code)]
const LPAREN: char = '(';
#[allow(dead_code)]
const RPAREN: char = ')';
const HYPHEN: char = '-';

const UTF8_START: char = '\u{0080}';

const MAILTO_URI_PREFIX: &str = "mailto:";

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCharacter => write!(f, "Invalid character."),
            Error::LocalPartEmpty => write!(f, "Local part is empty."),
            Error::LocalPartTooLong => write!(f, "Local part is too long. Length limit: {}", LOCAL_PART_MAX_LENGTH),
            Error::DomainEmpty => write!(f, "Domain is empty."),
            Error::DomainTooLong => {
                write!(f, "Domain is too long. Length limit: {}", DOMAIN_MAX_LENGTH)
            }
            Error::SubDomainTooLong => write!(f, "A sub-domain is too long. Length limit: {}", SUB_DOMAIN_MAX_LENGTH),
            Error::MissingSeparator => write!(f, "Missing separator character '{}'.", AT),
            Error::DomainTooFew => write!(f, "Too few parts in the domain"),
            Error::DomainInvalidSeparator => {
                write!(f, "Invalid placement of the domain separator '{:?}", DOT)
            }
            Error::AddressTooLong => {
                write!(f, "The Address is too long. Length limit: {}", ADDRESS_MAX_LENGTH)
            }
            Error::InvalidIPAddress => write!(f, "Invalid IP Address specified for domain."),
            Error::UnbalancedQuotes => write!(f, "Quotes around the local-part are unbalanced."),
            Error::InvalidComment => write!(f, "A comment was badly formed."),
            Error::InvalidLabel => write!(f, "Invalid label."),
        }
    }
}

impl std::error::Error for Error {}

impl<T> From<Error> for std::result::Result<T, Error> {
    fn from(err: Error) -> Self {
        Err(err)
    }
}

// ------------------------------------------------------------------------------------------------

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// From RFC 5321, section 2.4:
//
// The local-part of a mailbox MUST BE treated as case sensitive. Therefore,
// SMTP implementations MUST take care to preserve the case of mailbox
// local-parts. In particular, for some hosts, the user "smith" is different
// from the user "Smith". However, exploiting the case sensitivity of mailbox
// local-parts impedes interoperability and is discouraged. Mailbox domains
// follow normal DNS rules and are hence not case sensitive.
//

impl PartialEq for EmailAddress {
    fn eq(&self, other: &Self) -> bool {
        let (left, right, _) = split_parts(&self.0).unwrap();
        let (other_left, other_right, _) = split_parts(&other.0).unwrap();
        left.eq(other_left) && right.eq_ignore_ascii_case(other_right)
    }
}

impl Hash for EmailAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl FromStr for EmailAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_address(s)
    }
}

impl From<EmailAddress> for String {
    fn from(email: EmailAddress) -> Self {
        email.0
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "serde_support")]
impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};

        struct EmailAddressVisitor;

        impl Visitor<'_> for EmailAddressVisitor {
            type Value = EmailAddress;

            fn expecting(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
                fmt.write_str("data")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                EmailAddress::from_str(s).map_err(|err| {
                    let exp = format!("{}", err);
                    Error::invalid_value(Unexpected::Str(s), &exp.as_ref())
                })
            }
        }

        deserializer.deserialize_str(EmailAddressVisitor)
    }
}

impl EmailAddress {
    ///
    /// Creates an `EmailAddress` without checking if the email is valid. Only
    /// call this method if the address is known to be valid.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use email_address::EmailAddress;
    ///
    /// let unchecked = "john.doe@example.com";
    /// let email = EmailAddress::from_str(unchecked).expect("email is not valid");
    /// let valid_email = String::from(email);
    /// let email = EmailAddress::new_unchecked(valid_email);
    ///
    /// assert_eq!("John Doe <john.doe@example.com>", email.to_display("John Doe"));
    /// ```
    pub fn new_unchecked<S>(address: S) -> Self
    where
        S: Into<String>,
    {
        Self(address.into())
    }

    ///
    /// Determine whether the `address` string is a valid email address. Note this is equivalent to
    /// the following:
    ///
    /// ```rust
    /// use email_address::*;
    /// use std::str::FromStr;
    ///
    /// let is_valid = EmailAddress::from_str("johnstonskj@gmail.com").is_ok();
    /// ```
    ///
    pub fn is_valid(address: &str) -> bool {
        Self::from_str(address).is_ok()
    }

    ///
    /// Determine whether the `part` string would be a valid `local-part` if it were in an
    /// email address.
    ///
    pub fn is_valid_local_part(part: &str) -> bool {
        parse_local_part(part).is_ok()
    }

    ///
    /// Determine whether the `part` string would be a valid `domain` if it were in an
    /// email address.
    ///
    pub fn is_valid_domain(part: &str) -> bool {
        parse_domain(part).is_ok()
    }

    ///
    /// Return this email address formatted as a URI. This will also URI-encode the email
    /// address itself. So, `name@example.org` becomes `mailto:name@example.org`.
    ///
    /// ```rust
    /// use email_address::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(
    ///     EmailAddress::from_str("name@example.org").unwrap().to_uri(),
    ///     String::from("mailto:name@example.org")
    /// );
    /// ```
    ///
    pub fn to_uri(&self) -> String {
        let encoded = encode(&self.0);
        format!("{}{}", MAILTO_URI_PREFIX, encoded)
    }

    ///
    /// Return a string formatted as a display email with the user name. This is commonly used
    /// in email headers and other locations where a display name is associated with the
    /// address.
    ///
    /// ```rust
    /// use email_address::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(
    ///     EmailAddress::from_str("name@example.org").unwrap().to_display("My Name"),
    ///     String::from("My Name <name@example.org>")
    /// );
    /// ```
    ///
    pub fn to_display(&self, display_name: &str) -> String {
        format!("{} <{}>", display_name, self)
    }

    ///
    /// Returns the local part of the email address. This is borrowed so that no additional
    /// allocation is required.
    ///
    /// ```rust
    /// use email_address::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(
    ///     EmailAddress::from_str("name@example.org").unwrap().local_part(),
    ///     String::from("name")
    /// );
    /// ```
    ///
    pub fn local_part(&self) -> &str {
        let (left, _, _) = split_parts(&self.0).unwrap();
        left
    }

    ///
    /// Returns the display part of the email address. This is borrowed so that no additional
    /// allocation is required.
    ///
    /// ```rust
    /// use email_address::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(
    ///     EmailAddress::from_str("Name <name@example.org>").unwrap().display_part(),
    ///     String::from("Name")
    /// );
    /// ```
    ///
    pub fn display_part(&self) -> &str {
        let (_, _, display) = split_parts(&self.0).unwrap();
        display
    }

    ///
    /// Returns the email part of the email address. This is borrowed so that no additional
    /// allocation is required.
    ///
    /// ```rust
    /// use email_address::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(
    ///     EmailAddress::from_str("Name <name@example.org>").unwrap().email(),
    ///     String::from("name@example.org")
    /// );
    /// ```
    ///
    pub fn email(&self) -> String {
        let (left, right, _) = split_parts(&self.0).unwrap();
        format!("{}@{}", left, right)
    }

    ///
    /// Returns the domain of the email address. This is borrowed so that no additional
    /// allocation is required.
    ///
    /// ```rust
    /// use email_address::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(
    ///     EmailAddress::from_str("name@example.org").unwrap().domain(),
    ///     String::from("example.org")
    /// );
    /// ```
    ///
    pub fn domain(&self) -> &str {
        let (_, right, _) = split_parts(&self.0).unwrap();
        right
    }

    ///
    /// Returns the email address as a string reference.
    ///
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

fn encode(address: &str) -> String {
    let mut result = String::new();
    for c in address.chars() {
        if is_uri_reserved(c) {
            result.push_str(&format!("%{:02X}", c as u8))
        } else {
            result.push(c);
        }
    }
    result
}

fn is_uri_reserved(c: char) -> bool {
    // No need to encode '@' as this is allowed in the email scheme.
    c == '!'
        || c == '#'
        || c == '$'
        || c == '%'
        || c == '&'
        || c == '\''
        || c == '('
        || c == ')'
        || c == '*'
        || c == '+'
        || c == ','
        || c == '/'
        || c == ':'
        || c == ';'
        || c == '='
        || c == '?'
        || c == '['
        || c == ']'
}

fn parse_address(address: &str) -> Result<EmailAddress, Error> {
    //
    // Deals with cases of '@' in `local-part`, if it is quoted they are legal, if
    // not then they'll return an `InvalidCharacter` error later.
    //
    let idna_address = &domain_to_unicode(address).0;
    let (local_part, domain, _) = split_parts(idna_address)?;

    parse_local_part(local_part)?;
    parse_domain(domain)?;
    validate_email_length(idna_address)?;
    Ok(EmailAddress(address.to_owned()))
}

fn validate_email_length(address: &str) -> Result<(), Error> {
    if address.len() <= ADDRESS_MAX_LENGTH {
        return Ok(());
    }
    Error::AddressTooLong.into()
}

fn split_parts(address: &str) -> Result<(&str, &str, &str), Error> {
    let (display, email) = split_display_email(address)?;
    let (local_part, domain) = split_at(email)?;
    Ok((local_part, domain, display))
}

fn split_display_email(text: &str) -> Result<(&str, &str), Error> {
    match text.rsplit_once(" <") {
        None => Ok(("", text)),
        Some(left_right) => {
            let email = &left_right.1[0..left_right.1.len() - 1];
            let display_name = left_right.0;

            Ok((display_name, email))
        }
    }
}

fn split_at(address: &str) -> Result<(&str, &str), Error> {
    match address.rsplit_once(AT) {
        None => Error::MissingSeparator.into(),
        Some(left_right) => Ok(left_right),
    }
}

fn parse_local_part(part: &str) -> Result<(), Error> {
    if part.is_empty() {
        Error::LocalPartEmpty.into()
    } else if part.len() > LOCAL_PART_MAX_LENGTH {
        Error::LocalPartTooLong.into()
    } else if part.starts_with(DQUOTE) && part.ends_with(DQUOTE) {
        if part.len() == 2 {
            Error::LocalPartEmpty.into()
        } else {
            parse_quoted_local_part(&part[1..part.len() - 1])
        }
    } else {
        parse_unquoted_local_part(part)
    }
}

fn parse_quoted_local_part(part: &str) -> Result<(), Error> {
    if is_qcontent(part) {
        return Ok(());
    }
    Error::InvalidCharacter.into()
}

fn parse_unquoted_local_part(part: &str) -> Result<(), Error> {
    if is_dot_atom_text(part) {
        return Ok(());
    }
    Error::InvalidCharacter.into()
}

fn parse_domain(part: &str) -> Result<(), Error> {
    if part.is_empty() {
        Error::DomainEmpty.into()
    } else if part.len() > DOMAIN_MAX_LENGTH {
        Error::DomainTooLong.into()
    } else if part.starts_with(HYPHEN) || part.ends_with(HYPHEN) || part.contains("-.") || part.contains(".-") {
        // An email address cannot have a period and a hyphen next to each other
        // An email address cannot have a {} immediately after the @-sign.', 'An email address cannot end with a {}.'
        Error::InvalidCharacter.into()
    } else if part.starts_with(LBRACKET) && part.ends_with(RBRACKET) {
        parse_literal_domain(&part[1..part.len() - 1])
    } else {
        parse_text_domain(part)
    }
}

fn parse_text_domain(part: &str) -> Result<(), Error> {
    if !part.ends_with(char::is_alphabetic) {
        return Error::InvalidCharacter.into(); // TODO
    }
    if is_dot_atom_text(part) {
        let sub_parts: Vec<&str> = part.split(DOT).collect();
        if sub_parts.len() < 2 {
            return Error::DomainTooFew.into();
        }
        for sub_part in sub_parts {
            if sub_part.len() > SUB_DOMAIN_MAX_LENGTH {
                return Error::SubDomainTooLong.into();
            }
            is_valid_label(sub_part)?;
        }
        return Ok(());
    }
    Error::InvalidCharacter.into()
}

fn is_valid_label(label: &str) -> Result<(), Error> {
    let re = Regex::new(r"..--").unwrap();
    for capture in re.find_iter(label) {
        if capture.as_str() != "xn--" {
            return Error::InvalidLabel.into(); // TODO: Better error
        }
    }
    Ok(())
}

fn parse_literal_domain(part: &str) -> Result<(), Error> {
    if part.chars().all(is_dtext_char) {
        return is_valid_label(part);
    }
    Error::InvalidCharacter.into()
}

// ------------------------------------------------------------------------------------------------

fn is_atext(c: char) -> bool {
    c.is_alphanumeric()
        || c == '!'
        || c == '#'
        || c == '$'
        || c == '%'
        || c == '&'
        || c == '\''
        || c == '*'
        || c == '+'
        || c == '-'
        || c == '/'
        || c == '='
        || c == '?'
        || c == '^'
        || c == '_'
        || c == '`'
        || c == '{'
        || c == '|'
        || c == '}'
        || c == '~'
        || is_uchar(c)
}

#[allow(dead_code)]
fn is_special(c: char) -> bool {
    c == '('
        || c == ')'
        || c == '<'
        || c == '>'
        || c == '['
        || c == ']'
        || c == ':'
        || c == ';'
        || c == '@'
        || c == '\\'
        || c == ','
        || c == '.'
        || c == DQUOTE
}

fn is_uchar(c: char) -> bool {
    c >= UTF8_START
}

fn is_atom(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_atext)
}

fn is_dot_atom_text(s: &str) -> bool {
    s.split(DOT).all(is_atom)
}

fn is_vchar(c: char) -> bool {
    ('\x21'..='\x7E').contains(&c)
}

fn is_wsp(c: char) -> bool {
    c == SP || c == HTAB
}

fn is_qtext_char(c: char) -> bool {
    c == '\x21' || ('\x23'..='\x5B').contains(&c) || ('\x5D'..='\x7E').contains(&c) || is_uchar(c)
}

fn is_qcontent(s: &str) -> bool {
    let mut char_iter = s.chars();
    while let Some(c) = &char_iter.next() {
        if c == &ESC {
            // quoted-pair
            match char_iter.next() {
                Some(c2) if is_vchar(c2) => (),
                _ => return false,
            }
        } else if !(is_wsp(*c) || is_qtext_char(*c)) {
            // qtext
            return false;
        }
    }
    true
}

fn is_dtext_char(c: char) -> bool {
    ('\x21'..='\x5A').contains(&c) || ('\x5E'..='\x7E').contains(&c)
}

#[allow(dead_code)]
fn is_ctext_char(c: char) -> bool {
    (c >= '\x21' && c == '\x27') || ('\x2A'..='\x5B').contains(&c) || ('\x5D'..='\x7E').contains(&c)
}

#[allow(dead_code)]
fn is_ctext(s: &str) -> bool {
    s.chars().all(is_ctext_char)
}

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid(address: &str, test_case: Option<&str>) {
        if let Some(test_case) = test_case {
            println!(">> test case: {}", test_case);
            println!("     <{}>", address);
        } else {
            println!(">> <{}>", address);
        }
        assert!(EmailAddress::is_valid(address));
    }

    #[test]
    fn test_good_examples_from_wikipedia_01() {
        is_valid("simple@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_02() {
        is_valid("very.common@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_03() {
        is_valid("disposable.style.email.with+symbol@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_04() {
        is_valid("other.email-with-hyphen@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_05() {
        is_valid("fully-qualified-domain@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_06() {
        is_valid(
            "user.name+tag+sorting@example.com",
            Some(" may go to user.name@example.com inbox depending on mail server"),
        );
    }

    #[test]
    fn test_good_examples_from_wikipedia_07() {
        is_valid("x@example.com", Some("one-letter local-part"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_08() {
        is_valid("example-indeed@strange-example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_09() {
        is_valid(
            "admin@mailserver1",
            Some("local domain name with no TLD, although ICANN highly discourages dotless email addresses"),
        );
    }

    #[test]
    fn test_good_examples_from_wikipedia_10() {
        is_valid("example@s.example", Some("see the List of Internet top-level domains"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_11() {
        is_valid("\" \"@example.org", Some("space between the quotes"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_12() {
        is_valid("\"john..doe\"@example.org", Some("quoted double dot"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_13() {
        is_valid(
            "mailhost!username@example.org",
            Some("bangified host route used for uucp mailers"),
        );
    }

    #[test]
    fn test_good_examples_from_wikipedia_14() {
        is_valid(
            "user%example.com@example.org",
            Some("% escaped mail route to user@example.com via example.org"),
        );
    }

    #[test]
    fn test_good_examples_from_wikipedia_15() {
        is_valid("jsmith@[192.168.2.1]", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_16() {
        is_valid("jsmith@[IPv6:2001:db8::1]", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_17() {
        is_valid("user+mailbox/department=shipping@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_18() {
        is_valid("!#$%&'*+-/=?^_`.{|}~@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_19() {
        // '@' is allowed in a quoted local part. Sorry.
        is_valid("\"Abc@def\"@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_20() {
        is_valid("\"Joe.\\\\Blow\"@example.com", None);
    }

    #[test]
    fn test_good_examples_from_wikipedia_21() {
        is_valid("用户@例子.广告", Some("Chinese"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_22() {
        is_valid("अजय@डाटा.भारत", Some("Hindi"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_23() {
        is_valid("квіточка@пошта.укр", Some("Ukranian"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_24() {
        is_valid("θσερ@εχαμπλε.ψομ", Some("Greek"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_25() {
        is_valid("Dörte@Sörensen.example.com", Some("German"));
    }

    #[test]
    fn test_good_examples_from_wikipedia_26() {
        is_valid("коля@пример.рф", Some("Russian"));
    }

    // ------------------------------------------------------------------------------------------------

    #[test]
    fn test_to_strings() {
        let email = EmailAddress::from_str("коля@пример.рф").unwrap();

        assert_eq!(String::from(email.clone()), String::from("коля@пример.рф"));

        assert_eq!(email.to_string(), String::from("коля@пример.рф"));

        assert_eq!(email.as_ref(), "коля@пример.рф");
    }

    #[test]
    fn test_to_display() {
        let email = EmailAddress::from_str("коля@пример.рф").unwrap();

        assert_eq!(email.to_display("коля"), String::from("коля <коля@пример.рф>"));
    }

    #[test]
    fn test_touri() {
        let email = EmailAddress::from_str("коля@пример.рф").unwrap();

        assert_eq!(email.to_uri(), String::from("mailto:коля@пример.рф"));
    }

    // ------------------------------------------------------------------------------------------------

    fn expect(address: &str, error: Error, test_case: Option<&str>) {
        if let Some(test_case) = test_case {
            println!(">> test case: {}", test_case);
            println!("     <{}>, expecting {:?}", address, error);
        } else {
            println!(">> <{}>, expecting {:?}", address, error);
        }
        assert_eq!(EmailAddress::from_str(address), error.into());
    }

    #[test]
    fn test_bad_examples_from_wikipedia_00() {
        expect("Abc.example.com", Error::MissingSeparator, Some("no @ character"));
    }

    #[test]
    fn test_bad_examples_from_wikipedia_01() {
        expect(
            "A@b@c@example.com",
            Error::InvalidCharacter,
            Some("only one @ is allowed outside quotation marks"),
        );
    }

    #[test]
    fn test_bad_examples_from_wikipedia_02() {
        expect(
            "a\"b(c)d,e:f;g<h>i[j\\k]l@example.com",
            Error::InvalidCharacter,
            Some("none of the special characters in this local-part are allowed outside quotation marks"),
        );
    }

    #[test]
    fn test_bad_examples_from_wikipedia_03() {
        expect(
            "just\"not\"right@example.com",
            Error::InvalidCharacter,
            Some("quoted strings must be dot separated or the only element making up the local-part"),
        );
    }

    #[test]
    fn test_bad_examples_from_wikipedia_04() {
        expect(
            "this is\"not\\allowed@example.com",
            Error::InvalidCharacter,
            Some(
                "spaces, quotes, and backslashes may only exist when within quoted strings and preceded by a backslash",
            ),
        );
    }

    #[test]
    fn test_bad_examples_from_wikipedia_05() {
        // ()
        expect("this\\ still\"not\\allowed@example.com",
            Error::InvalidCharacter,
        Some("even if escaped (preceded by a backslash), spaces, quotes, and backslashes must still be contained by quotes")
        );
    }

    #[test]
    fn test_bad_examples_from_wikipedia_06() {
        expect(
            "1234567890123456789012345678901234567890123456789012345678901234+x@example.com",
            Error::LocalPartTooLong,
            Some("local part is longer than 64 characters"),
        );
    }

    #[test]
    fn test_bad_example_01() {
        expect(
            "foo@example.v1234567890123456789012345678901234567890123456789012345678901234v.com",
            Error::SubDomainTooLong,
            Some("domain part is longer than 64 characters"),
        );
    }

    #[test]
    fn test_bad_example_02() {
        expect("@example.com", Error::LocalPartEmpty, Some("local-part is empty"));
    }

    #[test]
    fn test_bad_example_03() {
        expect("\"\"@example.com", Error::LocalPartEmpty, Some("local-part is empty"));
    }

    #[test]
    fn test_bad_example_04() {
        expect("simon@", Error::DomainEmpty, Some("domain is empty"));
    }

    // make sure Error impl Send + Sync
    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    #[test]
    fn test_error_traits() {
        is_send::<Error>();
        is_sync::<Error>();
    }

    #[test]
    // BUG: https://github.com/johnstonskj/rust-email_address/issues/11
    fn test_eq_name_case_sensitive_local() {
        let email = EmailAddress::new_unchecked("simon@example.com");

        assert_eq!(email, EmailAddress::new_unchecked("simon@example.com"));
        assert_ne!(email, EmailAddress::new_unchecked("Simon@example.com"));
        assert_ne!(email, EmailAddress::new_unchecked("simoN@example.com"));
    }

    #[test]
    // BUG: https://github.com/johnstonskj/rust-email_address/issues/11
    fn test_eq_name_case_insensitive_domain() {
        let email = EmailAddress::new_unchecked("simon@example.com");

        assert_eq!(email, EmailAddress::new_unchecked("simon@Example.com"));
        assert_eq!(email, EmailAddress::new_unchecked("simon@example.COM"));
    }
}
