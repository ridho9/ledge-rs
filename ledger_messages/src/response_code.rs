#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ResponseCode {
    OK = 0x0_u8, 
    ERROR = 0x1_u8, 
    #[default]
    NullVal = 0xff_u8, 
}
impl From<u8> for ResponseCode {
    #[inline]
    fn from(v: u8) -> Self {
        match v {
            0x0_u8 => Self::OK, 
            0x1_u8 => Self::ERROR, 
            _ => Self::NullVal,
        }
    }
}
impl From<ResponseCode> for u8 {
    #[inline]
    fn from(v: ResponseCode) -> Self {
        match v {
            ResponseCode::OK => 0x0_u8, 
            ResponseCode::ERROR => 0x1_u8, 
            ResponseCode::NullVal => 0xff_u8,
        }
    }
}
impl core::str::FromStr for ResponseCode {
    type Err = ();

    #[inline]
    fn from_str(v: &str) -> core::result::Result<Self, Self::Err> {
        match v {
            "OK" => Ok(Self::OK), 
            "ERROR" => Ok(Self::ERROR), 
            _ => Ok(Self::NullVal),
        }
    }
}
impl core::fmt::Display for ResponseCode {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OK => write!(f, "OK"), 
            Self::ERROR => write!(f, "ERROR"), 
            Self::NullVal => write!(f, "NullVal"),
        }
    }
}
