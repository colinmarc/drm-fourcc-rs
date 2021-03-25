#![feature(try_trait)]
#![allow(non_camel_case_types)]

//! [`DrmFormat`] is an enum representing every pixel format supported by DRM
//! (as of kernel version 5.8.0).
//!
//! A [fourcc][fourcc_wiki] is four bytes of ascii representing some data format. This enum contains
//! every fourcc representing a pixel format supported by [DRM][drm_wiki], the Linux Direct
//! Rendering Manager.
//!
//! To get the bytes of the fourcc representing the format, cast to `u32`.
//!
//! ```
//! # use drm_fourcc::DrmFormat;
//! assert_eq!(DrmFormat::Xrgb8888 as u32, 875713112);
//! ```
//!
//! To get the string form of the fourcc, use [`DrmFormat::string_form`].
//!
//! ```
//! # use drm_fourcc::DrmFormat;
//! assert_eq!(DrmFormat::Xrgb8888.string_form(), "XR24");
//! ```
//!
//! The enum is autogenerated from the [canonical list][canonical] in the Linux source code.
//!
//! [fourcc_wiki]: https://en.wikipedia.org/wiki/FourCC
//! [drm_wiki]: https://en.wikipedia.org/wiki/Direct_Rendering_Managerz
//! [canonical]: https://github.com/torvalds/linux/blame/master/include/uapi/drm/drm_fourcc.h

use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub use as_enum::DrmFormat;
use std::option::NoneError;

mod as_enum;
mod consts;

impl DrmFormat {
    /// Get the string representation of the format's fourcc.
    pub fn string_form(&self) -> String {
        fourcc_string_form(*self as u32).expect("Must be valid fourcc")
    }
}

impl Debug for DrmFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DrmFormat")
            .field(&self.string_form())
            .finish()
    }
}

impl Display for DrmFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl TryFrom<u32> for DrmFormat {
    type Error = UnrecognizedFourcc;

    /// Convert from an u32
    ///
    /// ```
    /// # use drm_fourcc::DrmFormat;
    /// # use std::convert::TryFrom;
    /// assert_eq!(DrmFormat::try_from(875710274).unwrap(), DrmFormat::Bgr888);
    ///
    /// assert!(DrmFormat::try_from(0).is_err());
    ///
    /// // If the u32 is in the valid format to be a fourcc, you can see its string form
    /// assert_eq!(DrmFormat::try_from(828601953).unwrap_err().string_form(), Some("avc1".to_string()));
    /// ```
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(UnrecognizedFourcc(value))
    }
}

/// Wraps some u32 that isn't a DRM fourcc we recognize
///
/// ```
/// # use drm_fourcc::{DrmFormat, UnrecognizedFourcc};
/// # use std::convert::TryFrom;
/// // Get the u32
/// assert_eq!(UnrecognizedFourcc(42).0, 42);
///
/// // Get the string form
/// assert_eq!(UnrecognizedFourcc(828601953).string_form(), Some("avc1".to_string()));
/// assert_eq!(UnrecognizedFourcc(0).string_form(), None);
/// ```
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct UnrecognizedFourcc(pub u32);

impl UnrecognizedFourcc {
    /// If the u32 is in a valid format to be a fourcc, get its string form.
    pub fn string_form(&self) -> Option<String> {
        fourcc_string_form(self.0)
    }
}

impl Debug for UnrecognizedFourcc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = &mut f.debug_tuple("UnrecognizedFourcc");

        if let Some(string_form) = self.string_form() {
            debug = debug.field(&string_form);
        }

        debug.field(&self.0).finish()
    }
}

impl Display for UnrecognizedFourcc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Error for UnrecognizedFourcc {}

fn fourcc_string_form(fourcc: u32) -> Option<String> {
    let string = String::from_utf8(fourcc.to_le_bytes().to_vec()).map_err(|_| NoneError)?;

    let mut out = String::new();

    let chars: Vec<char> = string.chars().collect();
    let (start, last_chars) = chars.split_at(3);
    let last = last_chars[0];

    // first three bytes must be characters
    for char in start {
        if char.is_ascii_alphanumeric() {
            out.push(*char);
        } else {
            return None;
        }
    }

    // last byte is allowed to be null
    if last == '\0' {
        out.push(' ');
    } else {
        out.push(last);
    }

    Some(out)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn a_specific_var_has_correct_value() {
        assert_eq!(consts::DRM_FOURCC_AYUV, 1448433985);
    }

    #[test]
    fn enum_member_casts_to_const() {
        assert_eq!(
            DrmFormat::Xrgb8888 as u32,
            consts::DRM_FOURCC_XRGB8888 as u32
        );
    }

    #[test]
    fn enum_member_has_correct_string_format() {
        assert_eq!(DrmFormat::Xrgb8888.string_form(), "XR24");
    }

    #[test]
    fn fourcc_string_form_handles_valid() {
        assert_eq!(fourcc_string_form(875713112).unwrap(), "XR24");
        assert_eq!(fourcc_string_form(828601953).unwrap(), "avc1");
        assert_eq!(fourcc_string_form(0x316376).unwrap(), "vc1 ");
    }

    #[test]
    fn unrecognized_handles_valid_fourcc() {
        assert_eq!(
            format!("{}", UnrecognizedFourcc(828601953)),
            "UnrecognizedFourcc(\"avc1\", 828601953)"
        );
    }

    #[test]
    fn unrecognized_handles_invalid_fourcc() {
        assert_eq!(
            format!("{}", UnrecognizedFourcc(0)),
            "UnrecognizedFourcc(0)"
        );
    }

    #[test]
    fn can_clone_result() {
        let a = DrmFormat::try_from(0);
        let b = a;
        assert_eq!(a, b);
    }
}
