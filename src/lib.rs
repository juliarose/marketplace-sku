//! # tf2-sku
//! 
//! SKU parser for Team Fortress 2 items.
//! 
//! ## Usage
//!
//! ```
//! use tf2_sku::SKU;
//! use tf2_sku::tf2_enum::{Quality, KillstreakTier};
//! 
//! let sku = SKU::try_from("264;11;kt-3").unwrap();
//! 
//! assert_eq!(sku.defindex, 264);
//! assert_eq!(sku.quality, Quality::Strange);
//! assert_eq!(sku.killstreak_tier, Some(KillstreakTier::Professional));
//! assert_eq!(sku.to_string(), "264;11;kt-3");
//! ```

pub use tf2_enum;

use std::num::{IntErrorKind, ParseIntError};
use std::fmt;
use std::convert::TryFrom;
use tf2_enum::num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use tf2_enum::{Quality, KillstreakTier, Wear, Paint, Sheen, Killstreaker};
use serde::{Serialize, Serializer, de::{self, Visitor}};

/// Trait for converting to a SKU string.
pub trait SKUString {
    fn to_sku_string(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SKU {
    /// This can be negative at times to refer to items that are not defined in the schema e.g. 
    /// "Random Craft Hat".
    pub defindex: i32,
    pub quality: Quality,
    pub craftable: bool,
    pub australium: bool,
    pub strange: bool,
    pub festivized: bool,
    pub particle: Option<u32>,
    pub skin: Option<u32>,
    pub killstreak_tier: Option<KillstreakTier>,
    pub wear: Option<Wear>,
    pub target_defindex: Option<u32>,
    pub output_defindex: Option<u32>,
    pub output_quality: Option<Quality>,
    pub craft_number: Option<u32>,
    pub crate_number: Option<u32>,
    pub paint: Option<Paint>,
    pub sheen: Option<Sheen>,
    pub killstreaker: Option<Killstreaker>,
}

/// Creates a SKU with default values. All `Option` fields will be `None`, and all `bool` fields 
/// will be `false`, with the exception of craftable, which is `true`. `quality` will be 
/// [`Quality::Normal`]. 
impl Default for SKU {
    fn default() -> Self {
        Self {
            defindex: 0,
            quality: Quality::Normal,
            craftable: true,
            australium: false,
            strange: false,
            festivized: false,
            particle: None,
            skin: None,
            killstreak_tier: None,
            wear: None,
            target_defindex: None,
            output_defindex: None,
            output_quality: None,
            craft_number: None,
            crate_number: None,
            paint: None,
            sheen: None,
            killstreaker: None,
        }
    }
}

impl SKU {
    /// Creates a new SKU using the given `defindex` and `quality`. All `Option` fields will be 
    /// `None`, and all `bool` fields will be `false`, with the exception of craftable, which is 
    /// `true`. 
    /// 
    /// # Examples
    ///
    /// ```
    /// use tf2_sku::{SKU, tf2_enum::Quality};
    /// 
    /// let sku = SKU::new(264, Quality::Strange);
    /// assert_eq!(sku.to_string(), "264;11");
    /// ```
    pub fn new(
        defindex: i32,
        quality: Quality,
    ) -> Self {
        Self {
            defindex,
            quality,
            ..Self::default()
        }
    }
    
    /// Infallible method for parsing from a string. Always produces an output regardless of 
    /// format. It's advised to use [`TryFrom<&str>`] over this method to ensure predictable output. 
    /// If no `defindex` is detected, it will default to `-1`. `quality` defaults to 
    /// [`Quality::Rarity2`]. If the SKU is properly formatted this functions identical to 
    /// [`TryFrom<&str>`].
    /// 
    /// # Examples
    /// 
    /// ```
    /// use tf2_sku::{SKU, tf2_enum::Quality};
    /// 
    /// let sku = SKU::from_str("12;u43;kt-0;gibus");
    /// assert_eq!(sku.defindex, 12);
    /// assert_eq!(sku.quality, Quality::Rarity2);
    /// assert_eq!(sku.particle, Some(43));
    /// assert!(sku.killstreak_tier.is_none());
    /// 
    /// // valid sku
    /// let sku = SKU::try_from("200;11;australium;kt-3").unwrap();
    /// // produces the same output if the SKU is valid
    /// assert_eq!(SKU::from_str("200;11;australium;kt-3"), sku);
    /// // invalid quality, produces a different output
    /// assert_ne!(SKU::from_str("200;100;australium;kt-3"), sku);
    /// ```
    pub fn from_str(string: &str) -> Self {
        let mut parsed = Self::default();
        let mut sku_split = string.split(';');
        let defindex_str = sku_split.next()
            .unwrap_or_default();
        let quality_str = sku_split.next()
            .unwrap_or_default();
        
        if let Ok(defindex) = defindex_str.parse::<i32>() {
            parsed.defindex = defindex;
        } else {
            parsed.defindex = -1;
            parse_sku_element(&mut parsed, defindex_str).ok();
        }
        
        if let Ok(quality) = parse_enum_u32::<Quality>("quality", quality_str) {
            parsed.quality = quality;
        } else {
            parsed.quality = Quality::Rarity2;
            parse_sku_element(&mut parsed, quality_str).ok();
        }
        
        while let Some(element) = sku_split.next() {
            parse_sku_element(&mut parsed, element).ok();
        }
        
        parsed
    }
}

/// This is the same as `to_string`.
impl SKUString for SKU {
    fn to_sku_string(&self) -> String {
        self.to_string()
    }
}

/// This is the same as `to_string`.
impl SKUString for &SKU {
    fn to_sku_string(&self) -> String {
        self.to_string()
    }
}

/// Formats SKU attributes into a string.
/// 
/// # Examples
///
/// ```
/// use tf2_sku::{SKU, tf2_enum::{Quality, KillstreakTier}};
/// 
/// let mut sku = SKU::new(264, Quality::Strange);
/// 
/// sku.killstreak_tier = Some(KillstreakTier::Professional);
/// 
/// assert_eq!(sku.to_string(), "264;11;kt-3");
/// ```
impl fmt::Display for SKU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = format!("{};{}", self.defindex, u32::from(self.quality));
        
        if let Some(particle) = self.particle {
            string.push_str(";u");
            string.push_str(particle.to_string().as_str());
        }
        
        if !self.craftable {
            string.push_str(";uncraftable");
        }
        
        if self.australium {
            string.push_str(";australium");
        }
        
        if self.strange {
            string.push_str(";strange");
        }
        
        if let Some(wear) = self.wear {
            string.push_str(";w");
            string.push_str(u32::from(wear).to_string().as_str());
        }
        
        if let Some(skin) = self.skin {
            string.push_str(";pk");
            string.push_str(skin.to_string().as_str());
        }
        
        if let Some(killstreak_tier) = self.killstreak_tier {
            string.push_str(";kt-");
            string.push_str(u32::from(killstreak_tier).to_string().as_str());
        }
        
        if self.festivized {
            string.push_str(";festive");
        }

        if let Some(crate_number) = self.crate_number {
            string.push_str(";c");
            string.push_str(crate_number.to_string().as_str());
        }

        if let Some(craft_number) = self.craft_number {
            string.push_str(";n");
            string.push_str(craft_number.to_string().as_str());
        }
        
        if let Some(target_defindex) = self.target_defindex {
            string.push_str(";td-");
            string.push_str(target_defindex.to_string().as_str());
        }
        
        if let Some(output_defindex) = self.output_defindex {
            string.push_str(";od-");
            string.push_str(output_defindex.to_string().as_str());
        }
        
        if let Some(output_quality) = self.output_quality {
            string.push_str(";oq-");
            string.push_str(u32::from(output_quality).to_string().as_str());
        }
        
        if let Some(paint) = self.paint {
            string.push_str(";p");
            string.push_str(u32::from(paint).to_string().as_str());
        }
        
        if let Some(sheen) = self.sheen {
            string.push_str(";ks-");
            string.push_str(u32::from(sheen).to_string().as_str());
        }
        
        if let Some(killstreaker) = self.killstreaker {
            string.push_str(";ke-");
            string.push_str(u32::from(killstreaker).to_string().as_str());
        }
        
        write!(f, "{}", string)
    }
}

/// Attempts to parse a SKU from a string. Fails if SKU contains invalid attribute e.g. a 
/// [`Quality`] not defined, `"kt-5"` is an invalid [`KillstreakTier`]. Ignores unknown 
/// attributes.
/// 
/// # Examples
///
/// ```
/// use tf2_sku::{SKU, tf2_enum::{Quality, KillstreakTier}};
/// 
/// let sku = SKU::try_from("264;11;kt-3").unwrap();
/// 
/// assert_eq!(sku.defindex, 264);
/// assert_eq!(sku.quality, Quality::Strange);
/// assert_eq!(sku.killstreak_tier, Some(KillstreakTier::Professional));
/// ```
impl TryFrom<&str> for SKU {
    type Error = ParseError;
        
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        let mut sku_split = string.split(';');
        let defindex_str = sku_split.next()
            .ok_or(ParseError::InvalidFormat)?;
        let quality_str = sku_split.next()
            .ok_or(ParseError::InvalidFormat)?;
        let defindex = defindex_str.parse::<i32>()
            .map_err(|error| ParseError::ParseInt {
                key: "defindex",
                error,
            })?;
        let quality = parse_enum_u32::<Quality>("quality", quality_str)?;
        let mut parsed = SKU::new(defindex, quality);
        
        while let Some(element) = sku_split.next() {
            parse_sku_element(&mut parsed, element)?;
        }
        
        Ok(parsed)
    }
}

/// Parses a single SKU attribute.
fn parse_sku_element<'a>(
    parsed: &mut SKU,
    element: &str,
) -> Result<(), ParseError> {
    let mut split_at = element.len();
    
    // Walk back through chars until a non-digit is found
    for c in element.chars().rev() {
        if c.is_digit(10) {
            split_at -= 1;
        } else {
            break;
        }
    }
    
    // Split at the last digit (`value` will be an empty string if no digit was found)
    // This shouldn't cause issues with strings that contain varying byte lengths. If the last 
    // character is multi-byte it is not a valid digit, so it will stop immediately and `split_at`
    // will be the total byte length of the string.
    let (name, value) = element.split_at(split_at);
    
    match name {
        "u" => parsed.particle = Some(parse_u32("particle", value)?),
        "w" => parsed.wear = Some(parse_enum_u32("wear", value)?),
        "n" => parsed.craft_number = Some(parse_u32("craft number", value)?),
        "c" => parsed.crate_number = Some(parse_u32("crate number", value)?),
        "p" => parsed.paint = Some(parse_enum_u32("paint", value)?),
        "pk" => parsed.skin = Some(parse_u32("skin", value)?),
        "kt-" => parsed.killstreak_tier = Some(parse_enum_u32("killstreak tier", value)?),
        "td-" => parsed.target_defindex = Some(parse_u32("target defindex", value)?),
        "od-" => parsed.output_defindex = Some(parse_u32("output defindex", value)?),
        "oq-" => parsed.output_quality = Some(parse_enum_u32("output quality", value)?),
        "ks-" => parsed.sheen = Some(parse_enum_u32("sheen", value)?),
        "ke-" => parsed.killstreaker = Some(parse_enum_u32("killstreaker", value)?),
        "uncraftable" => parsed.craftable = false,
        "australium" => parsed.australium = true,
        "strange" => parsed.strange = true,
        "festive" => parsed.festivized = true,
        // ignore
        _ => {},
    }
    
    Ok(())
}

/// An error when parsing from a string.
#[derive(Debug)]
pub enum ParseError {
    /// An integer failed to parse.
    ParseInt {
        key: &'static str,
        error: ParseIntError,
    },
    /// The SKU format is not valid. Must begin with a defindex and a quality e.g. "5021;6".
    InvalidFormat,
    /// An attribute value is not valid.
    InvalidValue {
        key: &'static str,
        number: u32,
    },
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ParseInt {
                key,
                error ,
            } => match *error.kind() {
                IntErrorKind::Empty => write!(f, "Value for {key} in SKU is empty."),
                IntErrorKind::InvalidDigit => write!(f, "Value for {key} in SKU contains invalid digit."),
                IntErrorKind::PosOverflow => write!(f, "Value for {key} in SKU overflows integer bounds."),
                IntErrorKind::NegOverflow => write!(f, "Value for {key} in SKU underflows integer bounds."),
                // shouldn't occur
                IntErrorKind::Zero => write!(f, "Value for {key} in SKU zero for non-zero type."),
                _ => write!(f, "Value for {key} in SKU could not be parsed: {error}"),
            },
            ParseError::InvalidFormat => write!(f, "Invalid SKU format. Must begin with a defindex followed by a quality e.g. \"5021;6\""),
            ParseError::InvalidValue {
                key,
                number,
            } => write!(f, "Unknown {key}: {number}"),
        }
    }
}

impl Serialize for SKU {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> de::Deserialize<'de> for SKU {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct SKUVisitor;

        impl<'de> Visitor<'de> for SKUVisitor {
            type Value = SKU;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a string")
            }
            
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::Value::try_from(s).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(SKUVisitor)
    }
}

fn parse_enum_u32<T>(key: &'static str, s: &str) -> Result<T, ParseError>
where T:
    TryFromPrimitive<Primitive = u32>,
{
    T::try_from_primitive(parse_u32(key, s)?)
        .map_err(|TryFromPrimitiveError { number }| ParseError::InvalidValue {
            key,
            number,
        })
}

fn parse_u32(key: &'static str, value: &str) -> Result<u32, ParseError> {
    value.parse::<u32>()
        .map_err(|error| ParseError::ParseInt {
            key,
            error,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::{self, json};
    use std::sync::Arc;
    
    #[derive(Serialize, Deserialize)]
    struct Item {
        sku: SKU,
    }
    
    #[test]
    fn golden_frying_pan_correct() {
        assert_eq!(SKU::try_from("1071;11;kt-3").unwrap(), SKU {
            defindex: 1071,
            quality: Quality::Strange,
            craftable: true,
            australium: false,
            strange: false,
            festivized: false,
            particle: None,
            skin: None,
            killstreak_tier: Some(KillstreakTier::Professional),
            wear: None,
            craft_number: None,
            crate_number: None,
            target_defindex: None,
            output_defindex: None,
            output_quality: None,
            sheen: None,
            killstreaker: None,
            paint: None,
        });
    }
    
    #[test]
    fn professional_unusual_killstreak_skin() {
        assert_eq!(SKU::try_from("424;15;u703;w3;pk307;kt-3;ks-1;ke-2008").unwrap(), SKU {
            defindex: 424,
            quality: Quality::DecoratedWeapon,
            craftable: true,
            australium: false,
            strange: false,
            festivized: false,
            particle: Some(703),
            skin: Some(307),
            killstreak_tier: Some(KillstreakTier::Professional),
            wear: Some(Wear::FieldTested),
            craft_number: None,
            crate_number: None,
            target_defindex: None,
            output_defindex: None,
            output_quality: None,
            sheen: Some(Sheen::TeamShine),
            killstreaker: Some(Killstreaker::HypnoBeam),
            paint: None,
        });
    }
    
    #[test]
    fn attribute_with_four_byte_utf8_char_is_ignored() {
        assert!(SKU::try_from("1071;1;u-🍌🍌122;🍌🍌").unwrap().particle.is_none());
        assert!(SKU::try_from("1071;1;u🍌122;🍌🍌").unwrap().particle.is_none());
        assert!(SKU::try_from("1071;1;u🍌122🍌;🍌🍌").unwrap().particle.is_none());
    }
    #[test]
    
    fn parses_from_str() {
        let sku = SKU::from_str("u43;;;pk1;kt-0;gibus🍌");
        
        assert_eq!(sku.defindex, -1);
        assert_eq!(sku.quality, Quality::Rarity2);
        assert_eq!(sku.particle, Some(43));
        assert_eq!(sku.skin, Some(1));
        assert!(sku.killstreak_tier.is_none());
    }
    
    #[test]
    fn bad_quality_is_err() {
        assert!(SKU::try_from("1071;122").is_err());
    }
    
    #[test]
    fn empty_quality_is_err() {
        assert!(SKU::try_from("1;5;u;pk1").is_err());
    }
    
    #[test]
    fn unknown_attribute_is_ok() {
        assert!(SKU::try_from("1;5;superspecial").is_ok());
        assert_eq!(SKU::try_from("1;5;superspecial").unwrap().to_string(), "1;5");
    }
    
    #[test]
    fn bad_quality_is_err_check_error_key() {
        if let ParseError::InvalidValue { key, number } = SKU::try_from("1071;122").unwrap_err() {
            assert_eq!(key, "quality");
            assert_eq!(number, 122);
        } else {
            panic!("wrong error");
        }
    }
    
    #[test]
    fn negative_defindex_is_ok() {
        assert!(SKU::try_from("-1;11").is_ok());
    }
    
    #[test]
    fn paint_kit_correct() {
        assert!(SKU::try_from("16310;15;u703;w2;pk310").is_ok());
    }

    #[test]
    fn deserializes_from_json() {
        let item = serde_json::from_value::<Item>(json!({
            "sku": "16310;15;u703;w2;pk310"
        })).unwrap();

        assert_eq!(item.sku.defindex, 16310);
    }
    
    #[test]
    fn deserializes_to_json() {
        let sku = SKU::try_from("16310;15;u703;w2;pk310").unwrap();
        let s = serde_json::to_string(&Item { sku }).unwrap();

        assert_eq!(s, r#"{"sku":"16310;15;u703;w2;pk310"}"#);
    }
    
    #[test]
    fn to_sku_string_in_arc() {
        let sku = Arc::new(SKU::try_from("16310;15;u703;w2;pk310").unwrap());
        
        assert_eq!(sku.as_ref().to_sku_string(), "16310;15;u703;w2;pk310");
    }
}