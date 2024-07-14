use std::{any::type_name, collections::{BTreeMap, HashMap, HashSet}, hash::Hash};


pub enum DataItem {
    SmallInt(u8),
    SmallNegInt(i8),
    Uint1,
    Uint2,
    Uint4,
    Uint8,
    NegUint1,
    NegUint2,
    NegUint4,
    NegUint8,
    SmallByteString(usize),
    ByteString1,
    ByteString2,
    ByteString4,
    ByteString8,
    TerminatedByteString,
    SmallTextString(usize),
    TextString1,
    TextString2,
    TextString4,
    TextString8,
    TerminatedTextString,
    SmallArray(usize),
    Array1,
    Array2,
    Array4,
    Array8,
    TerminatedArray,
    SmallMap(usize),
    Map1,
    Map2,
    Map4,
    Map8,
    TerminatedMap,
    Tag(u8),
    SimpleOrFloat,
    NotSupported,
    UnsignedBigNum,
    NegativeBigNum,
    Bool(bool),
    Null,
    Undefined,
    Float2,
    Float4,
    Float8,
    Stop,
    InvalidByte,
}

#[inline]
pub fn decode_cbor<T>(bytes: &[u8]) -> Result<T, CborError> where T: Cbor {
    let (t, _) = <T as Cbor>::from_cbor_bytes(bytes)?;
    Ok(t)
}

#[derive(Debug)]
pub enum CborError {
    IllFormed(String),
    Unexpected(String),
}

pub trait Cbor {
    fn to_cbor_bytes(&self) -> Vec<u8>;

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized;
}

pub trait ToCbor {
    fn to_cbor_bytes(&self) -> Vec<u8>;
}

impl<T> ToCbor for &[T] where T: Cbor +  {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
            if self.len() < 24 {
                bytes.push(0x80 + self.len() as u8);
            } else {
                bytes.push(0x9b);
                bytes.extend_from_slice(&self.len().to_be_bytes());
            }
            for i in 0..self.len() {
                bytes.extend_from_slice(&self[i].to_cbor_bytes());
            }
        bytes
    }
}

pub fn byteslice_to_cbor(byteslice: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
        if byteslice.len() < 24 {
            bytes.push(0x40 + byteslice.len() as u8);
            bytes.extend_from_slice(byteslice);
            bytes
        } else {
            bytes.push(0x5b);
            bytes.extend_from_slice(&byteslice.len().to_be_bytes());
            bytes.extend_from_slice(byteslice);
            bytes
        }
}

pub fn byteslice_from_cbor(bytes: &[u8]) -> Result<(Vec<u8>, usize), CborError> {
    let mut v = Vec::new();
        let bytes_read;
        match expected_data_item(bytes[0]) {
            DataItem::SmallByteString(byte) => {
                v.extend_from_slice(&bytes[1..1+byte as usize]);
                bytes_read = byte+1;
            }
            DataItem::ByteString8 => {
                let data_len = u64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
                v.extend_from_slice(&bytes[9..9+data_len]);
                bytes_read = 9+data_len;
            },
            _ => return Err(CborError::Unexpected("Error from byteslice_from_cbor() function".to_owned()))
        };
        Ok((v, bytes_read))
}


impl Cbor for bool {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        match self {
            false => vec![0xf4],
            true => vec![0xf5],
        }
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Bool(b) => Ok((b, 1)),
            _ => return Err(CborError::Unexpected("Error from bool implementation".to_owned()))
        }
    }
}

impl Cbor for u8 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            0x00..0x18 => bytes.push(*self),
            _ => {
                bytes.push(0x18);
                bytes.push(*self);
            }
        }
        bytes
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::SmallInt(byte) => Ok((byte, 1)),
            DataItem::Uint1 => Ok((bytes[1], 2)),
            _ => return Err(CborError::Unexpected("Error from u8 implementation".to_owned()))
        }
    }
}

impl Cbor for u16 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        vec![
            0x19,
            self.to_be_bytes()[0],
            self.to_be_bytes()[1]
        ]
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Uint2 => Ok((
                u16::from_be_bytes([
                    bytes[1],
                    bytes[2]
                ]),
                3
            )),
            _ => return Err(CborError::Unexpected("Error from u16 implementation".to_owned()))
        }
    }
}

impl Cbor for u32 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        vec![
            0x1a,
            self.to_be_bytes()[0],
            self.to_be_bytes()[1],
            self.to_be_bytes()[2],
            self.to_be_bytes()[3]
        ]
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Uint4 => Ok(
                (u32::from_be_bytes([
                    bytes[1],
                    bytes[2],
                    bytes[3],
                    bytes[4]
                ]), 
                5
            )
            ),
            _ => return Err(CborError::Unexpected("Error from u32 implementation".to_owned()))
        }
    }
}

impl Cbor for u64 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        vec![
            0x1b,
            self.to_be_bytes()[0],
            self.to_be_bytes()[1],
            self.to_be_bytes()[2],
            self.to_be_bytes()[3],
            self.to_be_bytes()[4],
            self.to_be_bytes()[5],
            self.to_be_bytes()[6],
            self.to_be_bytes()[7]
        ]
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Uint8 => Ok((u64::from_be_bytes([
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
                bytes[6],
                bytes[7],
                bytes[8]
            ]), 
            9)),
            _ => return Err(CborError::Unexpected("Error from u64 implementation".to_owned()))
        }
    }
}

impl Cbor for usize {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let num = *self as u64;
        vec![
            0x1b,
            num.to_be_bytes()[0],
            num.to_be_bytes()[1],
            num.to_be_bytes()[2],
            num.to_be_bytes()[3],
            num.to_be_bytes()[4],
            num.to_be_bytes()[5],
            num.to_be_bytes()[6],
            num.to_be_bytes()[7]
        ]
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Uint8 => Ok((u64::from_be_bytes([
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
                bytes[6],
                bytes[7],
                bytes[8]
            ]) as usize, 
            9)),
            _ => return Err(CborError::Unexpected("Error from usize implementation".to_owned()))
        }
    }
}

impl Cbor for i8 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if *self < 0 {
            match self {
                0x20..0x38 => bytes.push(*self as u8),
                _ => {
                    bytes.push(0x38);
                    bytes.push((-self.abs()).to_be_bytes()[0]);
                },
            };
        } else {
            bytes.extend_from_slice(&(*self as u8).to_cbor_bytes());
        }
        bytes
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::SmallNegInt(byte) => Ok((byte, 1)),
            DataItem::NegUint1 => Ok((i8::from_be_bytes([
                bytes[1]
            ]), 2)),
            _ => return Err(CborError::Unexpected("Error from i8 implementation".to_owned()))
        }
    }
}

impl Cbor for i16 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        if *self < 0 {
            let s = - self.abs();
            vec![
                0x39,
                s.to_be_bytes()[0],
                s.to_be_bytes()[1]
            ]
        } else {
            (*self as u16).to_cbor_bytes()
        }
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Uint2 => {
                let (num, bytes_read) = <u16 as Cbor>::from_cbor_bytes(bytes)?;
                Ok((num as i16, bytes_read))
            },
            DataItem::NegUint2 => Ok((i16::from_be_bytes([
                bytes[1], 
                bytes[2]
            ]), 3)),
            _ => return Err(CborError::Unexpected("Error from i16 implementation".to_owned()))
        }
    }
}

impl Cbor for i32 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        if *self < 0 {
            let s = - self.abs();
            vec![
                0x3a,
                s.to_be_bytes()[0],
                s.to_be_bytes()[1],
                s.to_be_bytes()[2],
                s.to_be_bytes()[3],
            ]
        } else {
            (*self as u32).to_cbor_bytes()
        }
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Uint4 => {
                let (num, bytes_read) = <u32 as Cbor>::from_cbor_bytes(bytes)?;
                Ok((num as i32, bytes_read))
            },
            DataItem::NegUint4 => Ok((i32::from_be_bytes([
                bytes[1], 
                bytes[2],
                bytes[3],
                bytes[4],
            ]), 5)),
            _ => return Err(CborError::Unexpected("Error from i32 implementation".to_owned()))
        }
    }
}

impl Cbor for i64 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        if *self < 0 {
            let s = - self.abs();
            vec![
                0x3a,
                s.to_be_bytes()[0],
                s.to_be_bytes()[1],
                s.to_be_bytes()[2],
                s.to_be_bytes()[3],
                s.to_be_bytes()[4],
                s.to_be_bytes()[5],
                s.to_be_bytes()[6],
                s.to_be_bytes()[7],
            ]
        } else {
            (*self as u32).to_cbor_bytes()
        }
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Uint8 => {
                let (num, bytes_read) = <u64 as Cbor>::from_cbor_bytes(bytes)?;
                Ok((num as i64, bytes_read))
            },
            DataItem::NegUint4 => Ok((i64::from_be_bytes([
                bytes[1], 
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
                bytes[6],
                bytes[7],
                bytes[8],
            ]), 9)),
            _ => return Err(CborError::Unexpected("Error from i64 implementation".to_owned()))
        }
    }
}

impl Cbor for f32 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        vec![
            0xfa,
            self.to_be_bytes()[0],
            self.to_be_bytes()[1],
            self.to_be_bytes()[2],
            self.to_be_bytes()[3]
        ]
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Float4 => Ok((f32::from_be_bytes([
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4]
            ]), 5)),
            _ => return Err(CborError::Unexpected("Error from f32 implementation".to_owned()))
        }
    }
}

impl Cbor for f64 {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        vec![
            0xfb,
            self.to_be_bytes()[0],
            self.to_be_bytes()[1],
            self.to_be_bytes()[2],
            self.to_be_bytes()[3],
            self.to_be_bytes()[4],
            self.to_be_bytes()[5],
            self.to_be_bytes()[6],
            self.to_be_bytes()[7]
        ]
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Float8 => Ok((f64::from_be_bytes([
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
                bytes[6],
                bytes[7],
                bytes[8]
            ]), 9)),
            _ => return Err(CborError::Unexpected("Error from f64 implementation".to_owned()))
        }
    }
}

impl Cbor for String {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if self.len() < 24 {
            bytes.push(0x60+self.len() as u8);
            bytes.extend_from_slice(&self.as_bytes());
        } else {
            bytes.push(0x7b);
            bytes.extend_from_slice(&self.len().to_be_bytes());
            bytes.extend_from_slice(self.as_bytes());
        }
        bytes
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        let mut v = String::new();
        let bytes_read;
        match expected_data_item(bytes[0]) {
            DataItem::SmallTextString(byte) => {
                let encoded_text = match std::str::from_utf8(&bytes[1..1+byte as usize]) {
                    Ok(text) => text,
                    Err(_) => return Err(CborError::IllFormed(format!("Decoded string is not valid utf-8"))),
                };
                v.push_str(encoded_text);
                bytes_read = v.len() + 1;
            }
            DataItem::TextString8 => {
                let data_len = u64::from_be_bytes([
                    bytes[1],
                    bytes[2],
                    bytes[3],
                    bytes[4],
                    bytes[5],
                    bytes[6],
                    bytes[7],
                    bytes[8]
                    ]) as usize;
                let encoded_text = match std::str::from_utf8(&bytes[9..9+data_len]) {
                    Ok(text) => text,
                    Err(_) => return Err(CborError::IllFormed(format!("Decoded string is not valid utf-8"))),
                };
                v.push_str(encoded_text);
                bytes_read = v.len() + 9;
            },
            _ => return Err(CborError::Unexpected("Error from String implementation".to_owned()))
        };
        Ok((v, bytes_read))
    }
}


impl<T> Cbor for Vec<T> where T: Cbor {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        if self.len() < 24 {
            v.push(0x80 + self.len() as u8);
        } else {
            v.push(0x9b);
            v.extend_from_slice(&self.len().to_be_bytes());
        }
        for item in self {
            v.extend_from_slice(&item.to_cbor_bytes());
        }
        v
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        let mut v = Vec::new();
        let mut i = 0;
        match expected_data_item(bytes[0]) {
            DataItem::SmallArray(byte) => {
                i += 1;
                let mut count = 0;
                while count < byte {
                    let (t, bytes_read) = <T as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    v.push(t);
                    i += bytes_read;
                    count += 1;
                }
            }
            DataItem::Array8 => {
                let data_len = u64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
                i += 9;
                let mut count = 0;
                while count < data_len {
                    let (t, bytes_read) = <T as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    v.push(t);
                    i += bytes_read;
                    count += 1;
                }
            },
            _ => return Err(CborError::Unexpected(format!("Error from {} implementation", type_name::<T>())))
        }
        Ok((v, i))
    }
}

impl<T> Cbor for HashSet<T> where T: Cbor + Hash + Eq {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        if self.len() < 24 {
            v.push(0x80 + self.len() as u8);
        } else {
            v.push(0x9b);
            v.extend_from_slice(&self.len().to_be_bytes());
        }
        for item in self {
            v.extend_from_slice(&item.to_cbor_bytes());
        }
        v
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        let mut v = HashSet::new();
        let mut i = 0;
        match expected_data_item(bytes[0]) {
            DataItem::SmallArray(byte) => {
                i += 1;
                let mut count = 0;
                while count < byte {
                    let (t, bytes_read) = <T as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    v.insert(t);
                    i += bytes_read;
                    count += 1;
                }
            }
            DataItem::Array8 => {
                let data_len = u64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
                i += 9;
                let mut count = 0;
                while count < data_len {
                    let (t, bytes_read) = <T as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    v.insert(t);
                    i += bytes_read;
                    count += 1;
                }
            },
            _ => return Err(CborError::Unexpected(format!("Error from {} implementation", type_name::<T>())))
        }
        Ok((v, i))
    }
}

impl<K, V> Cbor for HashMap<K, V> 
where 
    K: Cbor + Hash + Eq,
    V: Cbor 
{
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if self.len() < 24 {
            bytes.push(0xa0 + self.len() as u8);
            for (key, value) in self {
                bytes.extend_from_slice(&key.to_cbor_bytes());
                bytes.extend_from_slice(&value.to_cbor_bytes());
            }
        } else {
            bytes.push(0xbb);
            bytes.extend_from_slice(&self.len().to_be_bytes());
            for (key, value) in self {
                bytes.extend_from_slice(&key.to_cbor_bytes());
                bytes.extend_from_slice(&value.to_cbor_bytes());
            }
        }
        bytes
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        // println!("bytes: {:x?}", bytes);
        let mut map = HashMap::new();
        let mut i = 0;
        match expected_data_item(bytes[0]) {
            DataItem::SmallMap(byte) => {
                i += 1;
                let mut count = 0;
                while count < byte {
                    let (key, key_bytes_read) = <K as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    let (value, value_bytes_read) = <V as Cbor>::from_cbor_bytes(&bytes[i+key_bytes_read..])?;
                    map.insert(key, value);
                    i += key_bytes_read + value_bytes_read;
                    count += 1;
                }
            }
            DataItem::Map8 => {
                let data_len = u64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
                i += 9;
                let mut count = 0;
                while count < data_len {
                    let (key, key_bytes_read) = <K as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    let (value, value_bytes_read) = <V as Cbor>::from_cbor_bytes(&bytes[i+key_bytes_read..])?;
                    map.insert(key, value);
                    i += key_bytes_read + value_bytes_read;
                    count += 1;
                }
            },
            _ => return Err(CborError::Unexpected(format!("Error from HashMap<{}, {}> implementation", type_name::<K>(), type_name::<V>())))
        }
        Ok((map, i))
    }
}


impl<K, V> Cbor for BTreeMap<K, V> 
where 
    K: Cbor + Hash + Eq + Ord,
    V: Cbor 
{
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if self.len() < 24 {
            bytes.push(0xa0 + self.len() as u8);
            for (key, value) in self {
                bytes.extend_from_slice(&key.to_cbor_bytes());
                bytes.extend_from_slice(&value.to_cbor_bytes());
            }
        } else {
            bytes.push(0xbb);
            bytes.extend_from_slice(&self.len().to_be_bytes());
            for (key, value) in self {
                bytes.extend_from_slice(&key.to_cbor_bytes());
                bytes.extend_from_slice(&value.to_cbor_bytes());
            }
        }
        bytes
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        // println!("bytes: {:x?}", bytes);
        let mut map = BTreeMap::new();
        let mut i = 0;
        match expected_data_item(bytes[0]) {
            DataItem::SmallMap(byte) => {
                i += 1;
                let mut count = 0;
                while count < byte {
                    let (key, key_bytes_read) = <K as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    let (value, value_bytes_read) = <V as Cbor>::from_cbor_bytes(&bytes[i+key_bytes_read..])?;
                    map.insert(key, value);
                    i += key_bytes_read + value_bytes_read;
                    count += 1;
                }
            }
            DataItem::Map8 => {
                let data_len = u64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
                i += 9;
                let mut count = 0;
                while count < data_len {
                    let (key, key_bytes_read) = <K as Cbor>::from_cbor_bytes(&bytes[i..])?;
                    let (value, value_bytes_read) = <V as Cbor>::from_cbor_bytes(&bytes[i+key_bytes_read..])?;
                    map.insert(key, value);
                    i += key_bytes_read + value_bytes_read;
                    count += 1;
                }
            },
            _ => return Err(CborError::Unexpected(format!("Error from BTreeMap<{}, {}> implementation", type_name::<K>(), type_name::<V>())))
        }
        Ok((map, i))
    }
}

///This is a sample impl for an enum.
#[derive(PartialEq, PartialOrd, Debug)]
pub enum Item {
    Int(Vec<i32>),
    Float(Vec<f32>),
    String(Vec<String>),
}

impl Cbor for Item {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Item::Int(item) => {
                bytes.push(0xc6);
                bytes.extend_from_slice(&item.to_cbor_bytes());
            },
            Item::Float(item) => {
                bytes.push(0xc6+1);
                bytes.extend_from_slice(&item.to_cbor_bytes());
            },
            Item::String(item) => {
                bytes.push(0xc6+2);
                bytes.extend_from_slice(&item.to_cbor_bytes());
            },
        };
        bytes
    }

    fn from_cbor_bytes(bytes: &[u8]) -> Result<(Self, usize), CborError>
        where 
            Self: Sized 
    {
        match expected_data_item(bytes[0]) {
            DataItem::Tag(byte) => match byte {
                0 => {
                    let (item, bytes_read) = <Vec<i32> as Cbor>::from_cbor_bytes(&bytes[1..])?;
                    Ok((Self::Int(item), bytes_read+1)) // The +1 is to account for the Tag
                },
                1 => {
                    let (item, bytes_read) = <Vec<f32> as Cbor>::from_cbor_bytes(&bytes[1..])?;
                    Ok((Self::Float(item), bytes_read+1)) // The +1 is to account for the Tag
                },
                2 => {
                    let (item, bytes_read) = <Vec<String> as Cbor>::from_cbor_bytes(&bytes[1..])?;
                    Ok((Self::String(item), bytes_read+1)) // The +1 is to account for the Tag
                },
                _ => Err(CborError::Unexpected(format!("Error from Item implementation. Expected either 0x0, 0x1, or 0x2. Got {:x}", byte)))
            },
            _ => Err(CborError::Unexpected(format!("Error from Item implementation."))),
        }
    }
}



#[inline]
pub fn expected_data_item(byte: u8) -> DataItem {
    // println!("decoding byte: {:x}", byte);
    match byte {
        0x00..0x18  => DataItem::SmallInt(byte),                    //unsigned integer 0x00..0x17 (0..23),
        0x18        => DataItem::Uint1,                           //unsigned integer (one-byte uint8_t follows),
        0x19        => DataItem::Uint2,                           //unsigned integer (two-byte uint16_t follows),
        0x1a        => DataItem::Uint4,                           //unsigned integer (four-byte uint32_t follows),
        0x1b        => DataItem::Uint8,                           //unsigned integer (eight-byte uint64_t follows),
        0x20..0x38  => DataItem::SmallNegInt(0x20 - byte as i8),    //negative integer -1-0x00..-1-0x17 (-1..-24),
        0x38        => DataItem::NegUint1,                        //negative integer -1-n (one-byte uint8_t for n follows),
        0x39        => DataItem::NegUint2,                        //negative integer -1-n (two-byte uint16_t for n follows),
        0x3a        => DataItem::NegUint4,                        //negative integer -1-n (four-byte uint32_t for n follows),
        0x3b        => DataItem::NegUint8,                        //negative integer -1-n (eight-byte uint64_t for n follows),
        0x40..0x58  => DataItem::SmallByteString(byte as usize-0x40),    //byte string (0x00..0x17 bytes follow),
        0x58        => DataItem::ByteString1,      //byte string (one-byte uint8_t for n, and then  n bytes follow),
        0x59        => DataItem::ByteString2,      //byte string (two-byte uint16_t for n, and then n bytes follow),
        0x5a        => DataItem::ByteString4,      //byte string (four-byte uint32_t for n, and then n bytes follow),
        0x5b        => DataItem::ByteString8,      //byte string (eight-byte uint64_t for n, and then n bytes follow),
        0x5f        => DataItem::TerminatedByteString,      //byte string, byte strings follow, terminated by "break",
        0x60..0x78  => DataItem::SmallTextString(byte as usize-0x60),      //UTF-8 string (0x00..0x17 bytes follow),
        0x78        => DataItem::TextString1,      //UTF-8 string (one-byte uint8_t for n, and then n bytes follow),
        0x79        => DataItem::TextString2,      //UTF-8 string (two-byte uint16_t for n, and then n bytes follow),
        0x7a        => DataItem::TextString4,      //UTF-8 string (four-byte uint32_t for n, and then n bytes follow),
        0x7b        => DataItem::TextString8,      //UTF-8 string (eight-byte uint64_t for n, and then n bytes follow),
        0x7f        => DataItem::TerminatedTextString,      //UTF-8 string, UTF-8 strings follow, terminated by "break",
        0x80..0x98  => DataItem::SmallArray(byte as usize - 0x80),      //array (0x00..0x17 data items follow),
        0x98        => DataItem::Array1,      //array (one-byte uint8_t for n, and then n data  items follow),
        0x99        => DataItem::Array2,      //array (two-byte uint16_t for n, and then n data items follow),
        0x9a        => DataItem::Array4,      //array (four-byte uint32_t for n, and then n data items follow),
        0x9b        => DataItem::Array8,      //array (eight-byte uint64_t for n, and then n data items follow),
        0x9f        => DataItem::TerminatedArray,      //array, data items follow, terminated by "break",
        0xa0..0xb8  => DataItem::SmallMap(byte as usize - 0xa0),      //map (0x00..0x17 pairs of data items follow),
        0xb8        => DataItem::Map1,      //map (one-byte uint8_t for n, and then n pairs of data items follow),
        0xb9        => DataItem::Map2,      //map (two-byte uint16_t for n, and then n pairs of data items follow),
        0xba        => DataItem::Map4,      //map (four-byte uint32_t for n, and then n pairs of data items follow),
        0xbb        => DataItem::Map8,      //map (eight-byte uint64_t for n, and then n pairs of data items follow),
        0xbf        => DataItem::TerminatedMap,      //map, pairs of data items follow, terminated by "break",
        0xc0        => DataItem::NotSupported,      //text-based date/time (data item follows; see Section 3.4.1),
        0xc1        => DataItem::NotSupported,      //epoch-based date/time (data item follows; see Section 3.4.2),
        0xc2        => DataItem::UnsignedBigNum,      //unsigned bignum (data item "byte string" follows),
        0xc3        => DataItem::NegativeBigNum,      //negative bignum (data item "byte string" follows),
        0xc4        => DataItem::NotSupported,      //decimal Fraction (data item "array" follows; see Section 3.4.4),
        0xc5        => DataItem::NotSupported,      //bigfloat (data item "array" follows; see Section 3.4.4),
        0xc6..0xd5  => DataItem::Tag(byte - 0xc6),      //(tag),
        0xd5..0xd8  => DataItem::NotSupported,      //expected conversion (data item follows; see Section 3.4.5.2),
        0xd8..0xdb  => DataItem::NotSupported,      //(more tags; 1/2/4/8 bytes of tag number and then a data item follow),
        0xe0..0xf4  => DataItem::NotSupported,      //(simple value),
        0xf4        => DataItem::Bool(false),      //false,
        0xf5        => DataItem::Bool(true),      //true,
        0xf6        => DataItem::Null,      //null,
        0xf7        => DataItem::Undefined,      //undefine,
        0xf8        => DataItem::NotSupported,      //(simple value, one byte follows),
        0xf9        => DataItem::Float2,      //half-precision float (two-byte IEEE 754),
        0xfa        => DataItem::Float4,      //single-precision float (four-byte IEEE 754),
        0xfb        => DataItem::Float8,      //double-precision float (eight-byte IEEE 754),
        0xff        => DataItem::Stop,      //"break" stop code,
        _           => DataItem::InvalidByte,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_u8_from_cbor() {
        let v: Vec<u8> = vec![1,2,3,4,5,6,7,8,9];
        let bytes = v.to_cbor_bytes();
        println!("bytes: {:x?}", bytes);
        let (z, _) = <Vec<u8> as Cbor>::from_cbor_bytes(&bytes).unwrap();
        assert_eq!(v, z);
    }

    #[test]
    fn test_str() {
        let str = "here is a str".to_owned();
        let long_str = "here is a string that is longer than 23 characters. Here are a couple extra words to make sure".to_owned();
        let encoded_str = str.to_cbor_bytes();
        let encoded_long_str = long_str.to_cbor_bytes();
        let (decoded_str, _) = <String as Cbor>::from_cbor_bytes(&encoded_str).unwrap();
        let (decoded_long_str, _) = <String as Cbor>::from_cbor_bytes(&encoded_long_str).unwrap();
        assert_eq!(str, decoded_str);
        assert_eq!(long_str, decoded_long_str);
    }

    #[test]
    fn test_byteslice() {
        let slice: &[u8] = &[0,1,2,3,4,5,6,7,8,9];
        let long_slice: &[u8] = &[0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9, ];
        let encoded_slice = byteslice_to_cbor(slice);
        let encoded_long_slice = byteslice_to_cbor(long_slice);
        let (decoded_slice, _) = byteslice_from_cbor(&encoded_slice).unwrap();
        let (decoded_long_slice, _) = byteslice_from_cbor(&encoded_long_slice).unwrap();
        assert_eq!(slice, decoded_slice);
        assert_eq!(long_slice, decoded_long_slice);
    }

    #[test]
    fn test_small_array() {
        let array = vec![1,2,3];
        let encoded_array = array.to_cbor_bytes();
        let decoded_array = decode_cbor::<Vec<i32>>(&encoded_array).unwrap();
        println!("decoded: {:?}", decoded_array);
        assert_eq!(array, decoded_array);
        let arrarray = vec![vec![vec![1]],vec![vec![2]],vec![vec![3]]];
        let encoded_array = arrarray.to_cbor_bytes();
        let decoded_array = decode_cbor::<Vec<Vec<Vec<i32>>>>(&encoded_array).unwrap();
        println!("decoded: {:?}", decoded_array);
        assert_eq!(arrarray, decoded_array);
    }

    #[test]
    fn test_array() {
        let array = vec![1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,-8,9,1,2,3,4,5,6,7,8,9,];
        let encoded_array = array.to_cbor_bytes();
        let decoded_array = decode_cbor::<Vec<i32>>(&encoded_array).unwrap();
        println!("decoded: {:?}", decoded_array);
        assert_eq!(array, decoded_array);
        let arrarray = vec![vec![vec![1]],vec![vec![2]],vec![vec![3]],vec![vec![4]],vec![vec![5]],vec![vec![6]],vec![vec![7]],vec![vec![8]],vec![vec![9]]];
        let encoded_array = arrarray.to_cbor_bytes();
        let decoded_array = decode_cbor::<Vec<Vec<Vec<i32>>>>(&encoded_array).unwrap();
        println!("decoded: {:?}", decoded_array);
        assert_eq!(arrarray, decoded_array);
    }

    #[test]
    fn test_slice() {
        let array: Vec<usize> = vec![1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9,];
        let encoded_array = (&array).to_cbor_bytes();
        let decoded_array = decode_cbor::<Vec<usize>>(&encoded_array).unwrap();
        println!("decoded: {:?}", decoded_array);
        assert_eq!(&array, &decoded_array);
        let arrarray = vec![vec![vec![1]],vec![vec![2]],vec![vec![3]],vec![vec![4]],vec![vec![5]],vec![vec![6]],vec![vec![7]],vec![vec![8]],vec![vec![9]]];
        let encoded_array = (&arrarray).to_cbor_bytes();
        let decoded_array = decode_cbor::<Vec<Vec<Vec<i32>>>>(&encoded_array).unwrap();
        println!("decoded: {:?}", decoded_array);
        assert_eq!(&arrarray, &decoded_array);
    }

    #[test]
    fn test_hashmap() {
        let mut map = HashMap::new();
        for i in 0..30 {
            map.insert(i, format!("value number {}", i));
        }
        let bytes = map.to_cbor_bytes();
        let decoded_map: HashMap<i32, String> = decode_cbor(&bytes).unwrap();
        assert_eq!(map, decoded_map);
    }

    #[test]
    fn test_btreemap() {
        let mut map = BTreeMap::new();
        for i in 0..30 {
            map.insert(i, format!("value number {}", i));
        }
        println!("plain: {:x?}", map);

        let bytes = map.to_cbor_bytes();
        let decoded_map: BTreeMap<i32, String> = decode_cbor(&bytes).unwrap();
        println!("decoded: {:x?}", decoded_map);
        assert_eq!(map, decoded_map);
    }

    #[test]
    fn test_enum() {
        let mut item = Item::Int(vec![1,2,10]);
        let bytes = item.to_cbor_bytes();
        let decoded_bytes = decode_cbor(&bytes).unwrap();
        assert_eq!(item, decoded_bytes);
    }

    #[test]
    fn test_btreemap_of_enums() {
        let mut map = HashMap::new();
        for i in 0..3 {
            map.insert(format!("{}",i),  Item::Int(vec![i, i+1, i+2]));
        }
        let bytes = map.to_cbor_bytes();
        println!("bytes: {:x?}", bytes);
        let decoded_map: HashMap<String, Item> = decode_cbor(&bytes).unwrap();
        assert_eq!(map, decoded_map);
    }

    #[test]
    fn test_hashset() {
        let array = vec![1,2,3,4,5,6,7,8,9,11,12,13,14,15,16,17,-18,19,21,22,23,24,25,26,27,28,29,];
        let mut set = HashSet::new();
        for item in array {
            set.insert(item);
        }
        let encoded_array = set.to_cbor_bytes();
        let decoded_array = decode_cbor::<HashSet<i32>>(&encoded_array).unwrap();
        println!("decoded: {:?}", decoded_array);
        assert_eq!(set, decoded_array);
    }

    #[test]
    fn test_bool() {
        let t = true;
        let f = false;

        let cbor_true = t.to_cbor_bytes();
        let cbor_false = f.to_cbor_bytes();

        let decoded_t: bool = decode_cbor(&cbor_true).unwrap();
        let decoded_f: bool = decode_cbor(&cbor_false).unwrap();

        assert_eq!(t, decoded_t);
        assert_eq!(f, decoded_f);

    }

}
