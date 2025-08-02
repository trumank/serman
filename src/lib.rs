use std::{io::Read, io::Write};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};

pub trait Readable<E = std::io::Error>
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E>
    where
        Self: Sized;
    fn de_vec<S: Read>(len: usize, stream: &mut S) -> Result<Vec<Self>, E>
    where
        Self: Sized,
    {
        read_array(len, stream, Self::de)
    }
    fn de_array<S: Read, const N: usize>(stream: &mut S) -> Result<[Self; N], E>
    where
        Self: Sized + Copy + Default,
    {
        let mut buf = [Default::default(); N];
        for i in buf.iter_mut() {
            *i = Self::de(stream)?;
        }
        Ok(buf)
    }
}
pub trait Writeable<E = std::io::Error>
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E>;
    fn ser_array<S: Write, T: AsRef<[Self]>>(this: T, stream: &mut S) -> Result<(), E>
    where
        Self: Sized,
    {
        for i in this.as_ref() {
            Self::ser(i, stream)?;
        }
        Ok(())
    }
}
pub trait ReadableCtx<C, E = std::io::Error>
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S, ctx: C) -> Result<Self, E>
    where
        Self: Sized;
}

impl<T> ReadExt for T where T: Read {}
pub trait ReadExt: Read {
    fn de<T: Readable<E>, E>(&mut self) -> Result<T, E>
    where
        Self: Sized,
        E: From<std::io::Error>,
    {
        T::de(self)
    }
    fn de_ctx<T: ReadableCtx<C, E>, C, E>(&mut self, ctx: C) -> Result<T, E>
    where
        Self: Sized,
        E: From<std::io::Error>,
    {
        T::de(self, ctx)
    }
}
impl<T> WriteExt for T where T: Write {}
pub trait WriteExt: Write {
    fn ser<T: Writeable<E>, E>(&mut self, value: &T) -> Result<(), E>
    where
        Self: Sized,
        E: From<std::io::Error>,
    {
        value.ser(self)
    }
    /// Serialize &[T] without length prefix
    fn ser_no_length<T: Writeable<E>, S: AsRef<[T]>, E>(&mut self, value: &S) -> Result<(), E>
    where
        Self: Sized,
        E: From<std::io::Error>,
    {
        T::ser_array(value.as_ref(), self)
    }
}

impl<const N: usize, T: Readable<E> + Default + Copy, E> Readable<E> for [T; N]
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        T::de_array(stream)
    }
}
impl<const N: usize, T: Writeable<E>, E> Writeable<E> for [T; N]
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        T::ser_array(self, stream)
    }
}

impl<E> Readable<E> for String
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(s: &mut S) -> Result<Self, E> {
        let len: i32 = s.read_i32::<LE>()?;
        read_string(len, s)
    }
}
impl<E> Writeable<E> for String
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        write_string(stream, self)
    }
}
impl<E> Writeable<E> for &str
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        write_string(stream, self)
    }
}

impl<T: Readable<E>, E> Readable<E> for Vec<T>
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        let len = stream.read_u32::<LE>()? as usize;
        T::de_vec(len, stream)
    }
}
impl<T: Readable<E>, E> ReadableCtx<usize, E> for Vec<T>
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S, ctx: usize) -> Result<Self, E> {
        T::de_vec(ctx, stream)
    }
}
impl<T: Writeable<E>, E> Writeable<E> for Vec<T>
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        stream.write_u32::<LE>(self.len() as u32)?;
        T::ser_array(self, stream)
    }
}

impl<E> Readable<E> for bool
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_u32::<LE>()? != 0)
    }
}
impl<E> Writeable<E> for bool
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_u32::<LE>(if *self { 1 } else { 0 })?)
    }
}
impl<E> Readable<E> for u8
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_u8()?)
    }
    fn de_vec<S: Read>(len: usize, stream: &mut S) -> Result<Vec<Self>, E>
    where
        Self: Sized,
    {
        let mut buf = vec![0; len];
        stream.read_exact(&mut buf)?;
        Ok(buf)
    }
    fn de_array<S: Read, const N: usize>(stream: &mut S) -> Result<[Self; N], E>
    where
        Self: Sized + Copy + Default,
    {
        let mut buf = [0; N];
        stream.read_exact(&mut buf)?;
        Ok(buf)
    }
}
impl<E> Writeable<E> for u8
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_u8(*self)?)
    }
    fn ser_array<S: Write, T: AsRef<[Self]>>(this: T, stream: &mut S) -> Result<(), E>
    where
        Self: Sized,
    {
        Ok(stream.write_all(this.as_ref())?)
    }
}
impl<E> Readable<E> for i8
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_i8()?)
    }
}
impl<E> Writeable<E> for i8
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_i8(*self)?)
    }
}
impl<E> Readable<E> for u16
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_u16::<LE>()?)
    }
}
impl<E> Writeable<E> for u16
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_u16::<LE>(*self)?)
    }
}
impl<E> Readable<E> for i16
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_i16::<LE>()?)
    }
}
impl<E> Writeable<E> for i16
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_i16::<LE>(*self)?)
    }
}
impl<E> Readable<E> for u32
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_u32::<LE>()?)
    }
}
impl<E> Writeable<E> for u32
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_u32::<LE>(*self)?)
    }
}
impl<E> Readable<E> for i32
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_i32::<LE>()?)
    }
}
impl<E> Writeable<E> for i32
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_i32::<LE>(*self)?)
    }
}
impl<E> Readable<E> for u64
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_u64::<LE>()?)
    }
}
impl<E> Writeable<E> for u64
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_u64::<LE>(*self)?)
    }
}
impl<E> Readable<E> for i64
where
    E: From<std::io::Error>,
{
    fn de<S: Read>(stream: &mut S) -> Result<Self, E> {
        Ok(stream.read_i64::<LE>()?)
    }
}
impl<E> Writeable<E> for i64
where
    E: From<std::io::Error>,
{
    fn ser<S: Write>(&self, stream: &mut S) -> Result<(), E> {
        Ok(stream.write_i64::<LE>(*self)?)
    }
}

pub fn read_array<S: Read, T, F, E>(len: usize, stream: &mut S, mut f: F) -> Result<Vec<T>, E>
where
    F: FnMut(&mut S) -> Result<T, E>,
    E: From<std::io::Error>,
{
    let mut array = Vec::with_capacity(len);
    for _ in 0..len {
        array.push(f(stream)?);
    }
    Ok(array)
}

pub fn read_string<S: Read, E>(len: i32, stream: &mut S) -> Result<String, E>
where
    E: From<std::io::Error>,
{
    if len < 0 {
        let chars = read_array((-len) as usize, stream, |r| r.read_u16::<LE>())?;
        let length = chars.iter().position(|&c| c == 0).unwrap_or(chars.len());
        Ok(String::from_utf16(&chars[..length]).unwrap())
    } else {
        let mut chars = vec![0; len as usize];
        stream.read_exact(&mut chars)?;
        let length = chars.iter().position(|&c| c == 0).unwrap_or(chars.len());
        Ok(String::from_utf8_lossy(&chars[..length]).into_owned())
    }
}

pub fn write_string<S: Write, E>(stream: &mut S, value: &str) -> Result<(), E>
where
    E: From<std::io::Error>,
{
    if value.is_empty() {
        stream.write_u32::<LE>(0)?;
    } else if value.is_ascii() {
        stream.write_u32::<LE>(value.len() as u32 + 1)?;
        stream.write_all(value.as_bytes())?;
        stream.write_u8(0)?;
    } else {
        let chars: Vec<u16> = value.encode_utf16().collect();
        stream.write_i32::<LE>(-(chars.len() as i32 + 1))?;
        for c in chars {
            stream.write_u16::<LE>(c)?;
        }
        stream.write_u16::<LE>(0)?;
    }
    Ok(())
}
