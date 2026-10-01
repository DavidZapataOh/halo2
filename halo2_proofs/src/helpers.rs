use std::io;

use ff::{Field, PrimeField};
use pasta_curves::arithmetic::CurveAffine;

pub(crate) trait CurveRead: CurveAffine {
    /// Reads a compressed element from the buffer and attempts to parse it
    /// using `from_bytes`.
    fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut compressed = Self::Repr::default();
        reader.read_exact(compressed.as_mut())?;
        Option::from(Self::from_bytes(&compressed))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid point encoding in proof"))
    }

    /// Reads an uncompressed element written by [`CurveWrite::write_uncompressed`]
    /// and attempts to parse it using `from_xy`.
    fn read_uncompressed<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut read_base = || {
            let mut repr = <Self::Base as PrimeField>::Repr::default();
            reader.read_exact(repr.as_mut())?;
            Option::from(Self::Base::from_repr(repr))
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid field element"))
        };
        let x = read_base()?;
        let y = read_base()?;
        Option::from(Self::from_xy(x, y))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid point encoding"))
    }
}

impl<C: CurveAffine> CurveRead for C {}

pub(crate) trait CurveWrite: CurveAffine {
    /// Writes the affine coordinates of this element, with the identity
    /// encoded as `(0, 0)`.
    fn write_uncompressed<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let (x, y) = Option::from(self.coordinates().map(|c| (*c.x(), *c.y())))
            .unwrap_or((Self::Base::ZERO, Self::Base::ZERO));
        writer.write_all(x.to_repr().as_ref())?;
        writer.write_all(y.to_repr().as_ref())
    }
}

impl<C: CurveAffine> CurveWrite for C {}

/// Converts a slice of `bool` into a `u8`.
///
/// Panics if the slice has length greater than 8.
pub(crate) fn pack(bits: &[bool]) -> u8 {
    let mut value = 0u8;
    assert!(bits.len() <= 8);
    for (bit_index, bit) in bits.iter().enumerate() {
        value |= (*bit as u8) << bit_index;
    }
    value
}

/// Writes the first `bits.len()` bits of a `u8` into `bits`.
pub(crate) fn unpack(byte: u8, bits: &mut [bool]) {
    for (bit_index, bit) in bits.iter_mut().enumerate() {
        *bit = (byte >> bit_index) & 1 == 1;
    }
}
