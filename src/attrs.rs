use derive_syn_parse::Parse;
use syn::{Token, punctuated::Punctuated};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Parse)]
#[non_exhaustive]
pub enum SisAttr {
    #[peek(Token![const], name = "Const")]
    Const (Token![const])
}

impl SisAttr {
    #[inline]
    pub fn as_const (&self) -> Option<&Token![const]> {
        match self {
            Self::Const(x) => Some(x),
            _ => None
        }
    }
}

#[derive(Parse)]
pub struct SisAttrs {
    #[call(Punctuated::parse_terminated)]
    pub attrs: Punctuated<SisAttr, Token![,]>
}