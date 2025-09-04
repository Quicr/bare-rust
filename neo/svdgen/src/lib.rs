use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Debug, Copy, Clone)]
struct U32(u32);

impl<'de> Deserialize<'de> for U32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(U32Visitor)
    }
}

struct U32Visitor;

impl Visitor<'_> for U32Visitor {
    type Value = U32;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a decimal or hex integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let original = v.to_ascii_lowercase();
        let digits = original.trim_start_matches("0x");
        let radix = if original == digits {
            10 // No 0x prefix => decimal
        } else {
            16 // 0x prefix present => hex
        };

        u32::from_str_radix(digits, radix)
            .map(|n| U32(n))
            .map_err(|e| {
                Error::custom(format!(
                    "Invalid integer (base {}): {} '{}'",
                    radix, e, digits
                ))
            })
    }
}

#[derive(Copy, Clone, Debug)]
enum Access {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl Default for Access {
    fn default() -> Self {
        Self::ReadWrite
    }
}

impl ToTokens for Access {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::ReadOnly => tokens.extend(quote! { ReadOnly }),
            Self::WriteOnly => tokens.extend(quote! { WriteOnly }),
            Self::ReadWrite => tokens.extend(quote! { ReadWrite }),
        }
    }
}

impl<'de> Deserialize<'de> for Access {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AccessVisitor)
    }
}

struct AccessVisitor;

impl Visitor<'_> for AccessVisitor {
    type Value = Access;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an Access enum string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "read-only" => Ok(Access::ReadOnly),
            "write-only" => Ok(Access::WriteOnly),
            "read-write" => Ok(Access::ReadWrite),
            _ => Err(Error::custom(format!("Invalid access value: {}", v))),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Field {
    name: String,
    bit_offset: usize,
    bit_width: usize,
    access: Option<Access>,
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Some(access) = self.access else {
            eprintln!("Skipping field {}: Missing access", self.name);
            return;
        };

        let name = format_ident!("{}", self.name);
        let bit_offset = self.bit_offset;
        let bit_width = self.bit_width;

        let access_type = match self.bit_width {
            1 => format_ident!("bool"),
            ..=8 => format_ident!("u8"),
            ..=16 => format_ident!("u16"),
            ..=32 => format_ident!("u16"),
            _ => return Default::default(),
        };

        tokens.extend(quote! {
            pub type #name = Field<#access_type, { ADDR }, #bit_offset, #bit_width, #access>;
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Fields {
    #[serde(rename = "field")]
    fields: Vec<Field>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Register {
    name: String,
    address_offset: U32,
    // size: Usize, // TODO: Do something with this
    access: Option<Access>,
    fields: Fields,
}

impl ToTokens for Register {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name);
        let address_offset = self.address_offset.0;
        let fields = self.fields.fields.iter();

        tokens.extend(quote! {
            pub mod #name {
                use super::*;

                pub const ADDR: u32 = BASE_ADDR + #address_offset;

                #(#fields)*
            }
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Registers {
    #[serde(rename = "register")]
    registers: Vec<Register>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Peripheral {
    #[serde(rename = "@derivedFrom")]
    derived_from: Option<String>,
    name: String,
    base_address: Option<U32>,
    registers: Option<Registers>,
}

impl Peripheral {
    fn rebase(&mut self, base: &Peripheral) {
        // Use values from `base` if not present in this object
        // This will need to be updated if more fields are introduced to Peripheral
        let Peripheral {
            base_address,
            registers,
            ..
        } = base.clone();

        let base_address = self.base_address.clone().or(base_address);
        let registers = self.registers.clone().or(registers);

        self.base_address = base_address;
        self.registers = registers;
    }
}

impl ToTokens for Peripheral {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Some(base_address) = self.base_address.as_ref() else {
            eprintln!("Skipping peripheral {}: Missing base_address", self.name);
            return;
        };

        let Some(registers) = self.registers.as_ref() else {
            eprintln!("Skipping peripheral {}: Missing registers", self.name);
            return;
        };

        let name = format_ident!("{}", self.name);
        let base_addr = base_address.0;
        let registers = registers.registers.iter();

        tokens.extend(quote! {
            pub mod #name {
                use super::{Field, ReadOnly, WriteOnly, ReadWrite};

                const BASE_ADDR: u32 = #base_addr;

                #(#registers)*
            }
        })
    }
}

#[derive(Deserialize, Debug)]
struct Peripherals {
    #[serde(rename = "peripheral")]
    peripherals: Vec<Peripheral>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Device {
    peripherals: Peripherals,
}

impl Device {
    fn find_peripheral(&self, name: &str) -> Peripheral {
        self.peripherals
            .peripherals
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .expect(&format!("Unknown peripheral: {}", name))
    }

    fn normalize(&mut self) {
        use std::collections::HashMap;

        // Follow derivedFrom relationships
        let rewrite: HashMap<String, Peripheral> = self
            .peripherals
            .peripherals
            .iter()
            .filter_map(|p| {
                p.derived_from
                    .as_ref()
                    .map(|df| (p.name.clone(), df.clone()))
            })
            .map(|(n, df)| (n, self.find_peripheral(&df)))
            .collect();

        for p in self.peripherals.peripherals.iter_mut() {
            rewrite.get(&p.name).map(|q| p.rebase(q));
        }

        // Apply access rules from registers down to fields
        for p in self.peripherals.peripherals.iter_mut() {
            for r in p.registers.as_mut().unwrap().registers.iter_mut() {
                for f in r.fields.fields.iter_mut() {
                    f.access = f.access.or(r.access);
                }
            }
        }
    }

    fn filter(&mut self, only: &[String]) {
        if only.is_empty() {
            return;
        }

        self.peripherals
            .peripherals
            .retain(|p| only.contains(&p.name));
    }
}

const PRELUDE: &str = "
pub struct ReadOnly;
pub struct WriteOnly;
pub struct ReadWrite;

pub trait FromU32 {
    fn from_u32(x: u32) -> Self;
}

impl FromU32 for bool {
    fn from_u32(x: u32) -> Self { x != 0 }
}

impl FromU32 for u8 {
    fn from_u32(x: u32) -> Self { x as Self }
}

impl FromU32 for u16 {
    fn from_u32(x: u32) -> Self { x as Self }
}

pub trait Read<T> {
    fn read() -> T;
}

pub trait Write<T> {
    fn write(t: T);
}

#[derive(Default)]
pub struct Field<T, const ADDR: u32, const OFF: usize, const W: usize, A>(
    core::marker::PhantomData<(T, A)>,
);

impl<T, const ADDR: u32, const OFF: usize, const W: usize, A> Field<T, ADDR, OFF, { W }, A>
{
    const ADDRESS: *mut u32 = ADDR as *mut u32;
    const MASK: u32 = ((1 << W) - 1) << OFF;
    const OFFSET: usize = OFF;
}

impl<T, const ADDR: u32, const OFF: usize, const W: usize> Read<T> for Field<T, ADDR, OFF, { W }, ReadOnly>
where
    T: FromU32,
{
    fn read() -> T {
        let raw = unsafe { Self::ADDRESS.read_volatile() };
        T::from_u32((raw & Self::MASK) >> Self::OFFSET)
    }
}

impl<T, const ADDR: u32, const OFF: usize, const W: usize> Read<T> for Field<T, ADDR, OFF, { W }, ReadWrite>
where
    T: FromU32,
{
    fn read() -> T {
        let raw = unsafe { Self::ADDRESS.read_volatile() };
        T::from_u32((raw & Self::MASK) >> Self::OFFSET)
    }
}

impl<T, const ADDR: u32, const OFF: usize, const W: usize> Write<T> for Field<T, ADDR, OFF, { W }, WriteOnly>
where
    u32: From<T>,
{
    fn write(t: T) {
        // TODO: Enforce a critical section around this
        let v = u32::from(t) << Self::OFFSET;
        let curr = unsafe { Self::ADDRESS.read_volatile() };
        let next = (curr & !Self::MASK) | (v & Self::MASK);
        unsafe { Self::ADDRESS.write_volatile(next) };
    }
}

impl<T, const ADDR: u32, const OFF: usize, const W: usize> Write<T> for Field<T, ADDR, OFF, { W }, ReadWrite>
where
    T: FromU32,
    u32: From<T>,
{
    fn write(t: T) {
        // TODO: Enforce a critical section around this
        let v = u32::from(t) << Self::OFFSET;
        let curr = unsafe { Self::ADDRESS.read_volatile() };
        let next = (curr & !Self::MASK) | (v & Self::MASK);
        unsafe { Self::ADDRESS.write_volatile(next) };
    }
}
";

#[derive(Default)]
struct Options {
    svd_file: String,
    only: Vec<String>,
}

/// A Builder is used to configure an SVD-to-Rust translation task.  Typical usage:
///
/// ```
/// let svd_file = svd::Builder::default()
///     .svd_file("STM32F405.svd")
///     .include("GPIOA")
///     .include("UART1")
///     .build();
/// ```
#[derive(Default)]
pub struct Builder {
    options: Options,
}

impl Builder {
    /// Specify the SVD file to be translated.
    pub fn svd_file(mut self, svd_file: &str) -> Self {
        self.options.svd_file = svd_file.to_string();
        self
    }

    /// Specify a peripheral to be translated.  If this method is never invoked, then all
    /// peripherals are translated.
    pub fn include(mut self, peripheral: &str) -> Self {
        self.options.only.push(peripheral.to_string());
        self
    }

    /// Parse the SVD file with the provided options.  Returns a representation of the parsed SVD
    /// file which can then be written to a file.
    pub fn build(self) -> Result<DeviceDescription> {
        let file = File::open(&self.options.svd_file)?;
        let reader = BufReader::new(file);

        let mut device: Device = serde_xml_rs::from_reader(reader)?;
        device.normalize();
        device.filter(&self.options.only);

        Ok(DeviceDescription { device })
    }
}

/// A device description parsed from an SVD file, ready to be translated to Rust.
pub struct DeviceDescription {
    device: Device,
}

impl DeviceDescription {
    /// Write Rust code to the provided path.
    pub fn write_to_file<P: AsRef<Path>>(self, path: P) -> Result<()> {
        let peripherals = self.device.peripherals.peripherals.iter();
        let tokens = quote! { #(#peripherals)* };
        let ast = syn::parse2(tokens).unwrap();
        let formatted = prettyplease::unparse(&ast);

        let mut file = File::create(path)?;
        file.write_all(PRELUDE.as_bytes())?;
        file.write_all(formatted.as_bytes())?;
        Ok(())
    }
}
