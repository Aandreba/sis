use derive_syn_parse::Parse;
use syn::{Token, punctuated::Punctuated};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Parse)]
#[non_exhaustive]
pub enum SisAttr {
    #[peek(Token![const], name = "Const")]
    Const (Token![const]),
    #[peek(Token![extern], name = "Export")]
    Export (Token![extern])
}

impl SisAttr {
    #[inline]
    pub fn is_const (&self) -> bool {
        return matches!(self, Self::Const(_))
    }

    #[inline]
    pub fn is_export (&self) -> bool {
        return matches!(self, Self::Export(_))
    }

    #[inline]
    pub fn as_const (&self) -> Option<&Token![const]> {
        match self {
            Self::Const(x) => Some(x),
            #[allow(unreachable_patterns)]
            _ => None
        }
    }

    #[inline]
    pub fn as_export (&self) -> Option<&Token![extern]> {
        match self {
            Self::Export(x) => Some(x),
            #[allow(unreachable_patterns)]
            _ => None
        }
    }
}

#[derive(Parse)]
pub struct SisAttrs {
    #[call(Punctuated::parse_terminated)]
    pub attrs: Punctuated<SisAttr, Token![,]>
}