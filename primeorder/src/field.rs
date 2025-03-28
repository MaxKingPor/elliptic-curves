/// Implements a field element type whose internal representation is in
/// Montgomery form, providing a combination of trait impls and inherent impls
/// which are `const fn` where possible.
///
/// Accepts a set of `const fn` arithmetic operation functions as arguments.
///
/// # Inherent impls
/// - `const ZERO: Self`
/// - `const ONE: Self` (multiplicative identity)
/// - `pub fn from_bytes`
/// - `pub fn from_slice`
/// - `pub fn from_uint`
/// - `fn from_uint_unchecked`
/// - `pub fn to_bytes`
/// - `pub fn to_canonical`
/// - `pub fn is_odd`
/// - `pub fn is_zero`
/// - `pub fn double`
///
/// NOTE: field implementations must provide their own inherent impls of
/// the following methods in order for the code generated by this macro to
/// compile:
///
/// - `pub fn invert`
/// - `pub fn sqrt`
///
/// # Trait impls
/// - `AsRef<$arr>`
/// - `ConditionallySelectable`
/// - `ConstantTimeEq`
/// - `ConstantTimeGreater`
/// - `ConstantTimeLess`
/// - `Default`
/// - `DefaultIsZeroes`
/// - `Eq`
/// - `Field`
/// - `PartialEq`
///
/// ## Ops
/// - `Add`
/// - `AddAssign`
/// - `Sub`
/// - `SubAssign`
/// - `Mul`
/// - `MulAssign`
/// - `Neg`
#[macro_export]
macro_rules! impl_mont_field_element {
    (
        $curve:tt,
        $fe:tt,
        $bytes:ty,
        $uint:ty,
        $modulus:expr,
        $arr:ty,
        $from_mont:ident,
        $to_mont:ident,
        $add:ident,
        $sub:ident,
        $mul:ident,
        $neg:ident,
        $square:ident
    ) => {
        impl $fe {
            /// Zero element.
            pub const ZERO: Self = Self(<$uint>::ZERO);

            /// Multiplicative identity.
            pub const ONE: Self = Self::from_uint_unchecked(<$uint>::ONE);

            /// Create a [`
            #[doc = stringify!($fe)]
            /// `] from a canonical big-endian representation.
            pub fn from_bytes(repr: &$bytes) -> $crate::elliptic_curve::subtle::CtOption<Self> {
                use $crate::elliptic_curve::FieldBytesEncoding;
                Self::from_uint(FieldBytesEncoding::<$curve>::decode_field_bytes(repr))
            }

            /// Decode [`
            #[doc = stringify!($fe)]
            /// `] from a big endian byte slice.
            pub fn from_slice(slice: &[u8]) -> $crate::elliptic_curve::Result<Self> {
                use $crate::elliptic_curve::array::{Array, typenum::Unsigned};

                if slice.len() != <$curve as $crate::elliptic_curve::Curve>::FieldBytesSize::USIZE {
                    return Err($crate::elliptic_curve::Error);
                }

                Option::from(Self::from_bytes(&Array::try_from(slice).unwrap()))
                    .ok_or($crate::elliptic_curve::Error)
            }

            /// Decode [`
            #[doc = stringify!($fe)]
            /// `]
            /// from [`
            #[doc = stringify!($uint)]
            /// `] converting it into Montgomery form:
            ///
            /// ```text
            /// w * R^2 * R^-1 mod p = wR mod p
            /// ```
            pub fn from_uint(uint: $uint) -> $crate::elliptic_curve::subtle::CtOption<Self> {
                use $crate::elliptic_curve::subtle::ConstantTimeLess as _;
                let is_some = uint.ct_lt(&$modulus);
                $crate::elliptic_curve::subtle::CtOption::new(
                    Self::from_uint_unchecked(uint),
                    is_some,
                )
            }

            /// Parse a [`
            #[doc = stringify!($fe)]
            /// `] from big endian hex-encoded bytes.
            ///
            /// Does *not* perform a check that the field element does not overflow the order.
            ///
            /// This method is primarily intended for defining internal constants.
            #[allow(dead_code)]
            pub(crate) const fn from_hex(hex: &str) -> Self {
                Self::from_uint_unchecked(<$uint>::from_be_hex(hex))
            }

            /// Convert a `u64` into a [`
            #[doc = stringify!($fe)]
            /// `].
            pub const fn from_u64(w: u64) -> Self {
                Self::from_uint_unchecked(<$uint>::from_u64(w))
            }

            /// Decode [`
            #[doc = stringify!($fe)]
            /// `] from [`
            #[doc = stringify!($uint)]
            /// `] converting it into Montgomery form.
            ///
            /// Does *not* perform a check that the field element does not overflow the order.
            ///
            /// Used incorrectly this can lead to invalid results!
            pub(crate) const fn from_uint_unchecked(w: $uint) -> Self {
                Self(<$uint>::from_words($to_mont(w.as_words())))
            }

            /// Returns the big-endian encoding of this [`
            #[doc = stringify!($fe)]
            /// `].
            pub fn to_bytes(self) -> $bytes {
                use $crate::elliptic_curve::FieldBytesEncoding;
                FieldBytesEncoding::<$curve>::encode_field_bytes(&self.to_canonical())
            }

            /// Translate [`
            #[doc = stringify!($fe)]
            /// `] out of the Montgomery domain, returning a [`
            #[doc = stringify!($uint)]
            /// `] in canonical form.
            #[inline]
            pub const fn to_canonical(self) -> $uint {
                <$uint>::from_words($from_mont(self.0.as_words()))
            }

            /// Determine if this [`
            #[doc = stringify!($fe)]
            /// `] is odd in the SEC1 sense: `self mod 2 == 1`.
            ///
            /// # Returns
            ///
            /// If odd, return `Choice(1)`.  Otherwise, return `Choice(0)`.
            pub fn is_odd(&self) -> Choice {
                use $crate::elliptic_curve::bigint::Integer;
                self.to_canonical().is_odd()
            }

            /// Determine if this [`
            #[doc = stringify!($fe)]
            /// `] is even in the SEC1 sense: `self mod 2 == 0`.
            ///
            /// # Returns
            ///
            /// If even, return `Choice(1)`.  Otherwise, return `Choice(0)`.
            pub fn is_even(&self) -> Choice {
                !self.is_odd()
            }

            /// Determine if this [`
            #[doc = stringify!($fe)]
            /// `] is zero.
            ///
            /// # Returns
            ///
            /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
            pub fn is_zero(&self) -> Choice {
                self.ct_eq(&Self::ZERO)
            }

            /// Add elements.
            pub const fn add(&self, rhs: &Self) -> Self {
                Self(<$uint>::from_words($add(
                    self.0.as_words(),
                    rhs.0.as_words(),
                )))
            }

            /// Double element (add it to itself).
            #[must_use]
            pub const fn double(&self) -> Self {
                self.add(self)
            }

            /// Subtract elements.
            pub const fn sub(&self, rhs: &Self) -> Self {
                Self(<$uint>::from_words($sub(
                    self.0.as_words(),
                    rhs.0.as_words(),
                )))
            }

            /// Multiply elements.
            pub const fn multiply(&self, rhs: &Self) -> Self {
                Self(<$uint>::from_words($mul(
                    self.0.as_words(),
                    rhs.0.as_words(),
                )))
            }

            /// Negate element.
            pub const fn neg(&self) -> Self {
                Self(<$uint>::from_words($neg(self.0.as_words())))
            }

            /// Compute modular square.
            #[must_use]
            pub const fn square(&self) -> Self {
                Self(<$uint>::from_words($square(self.0.as_words())))
            }

            /// Returns `self^exp`, where `exp` is a little-endian integer exponent.
            ///
            /// **This operation is variable time with respect to the exponent.**
            ///
            /// If the exponent is fixed, this operation is effectively constant time.
            pub const fn pow_vartime(&self, exp: &[u64]) -> Self {
                let mut res = Self::ONE;
                let mut i = exp.len();

                while i > 0 {
                    i -= 1;

                    let mut j = 64;
                    while j > 0 {
                        j -= 1;
                        res = res.square();

                        if ((exp[i] >> j) & 1) == 1 {
                            res = res.multiply(self);
                        }
                    }
                }

                res
            }
        }

        $crate::impl_mont_field_element_arithmetic!(
            $fe, $bytes, $uint, $arr, $add, $sub, $mul, $neg
        );
    };
}

/// Add arithmetic impls to the given field element.
#[macro_export]
macro_rules! impl_mont_field_element_arithmetic {
    (
        $fe:tt,
        $bytes:ty,
        $uint:ty,
        $arr:ty,
        $add:ident,
        $sub:ident,
        $mul:ident,
        $neg:ident
    ) => {
        impl AsRef<$arr> for $fe {
            fn as_ref(&self) -> &$arr {
                self.0.as_ref()
            }
        }

        impl Default for $fe {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl Eq for $fe {}
        impl PartialEq for $fe {
            fn eq(&self, rhs: &Self) -> bool {
                self.0.ct_eq(&(rhs.0)).into()
            }
        }

        impl From<u32> for $fe {
            fn from(n: u32) -> $fe {
                Self::from_uint_unchecked(<$uint>::from(n))
            }
        }

        impl From<u64> for $fe {
            fn from(n: u64) -> $fe {
                Self::from_uint_unchecked(<$uint>::from(n))
            }
        }

        impl From<u128> for $fe {
            fn from(n: u128) -> $fe {
                Self::from_uint_unchecked(<$uint>::from(n))
            }
        }

        impl $crate::elliptic_curve::subtle::ConditionallySelectable for $fe {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                Self(<$uint>::conditional_select(&a.0, &b.0, choice))
            }
        }

        impl $crate::elliptic_curve::subtle::ConstantTimeEq for $fe {
            fn ct_eq(&self, other: &Self) -> $crate::elliptic_curve::subtle::Choice {
                self.0.ct_eq(&other.0)
            }
        }

        impl $crate::elliptic_curve::subtle::ConstantTimeGreater for $fe {
            fn ct_gt(&self, other: &Self) -> $crate::elliptic_curve::subtle::Choice {
                self.0.ct_gt(&other.0)
            }
        }

        impl $crate::elliptic_curve::subtle::ConstantTimeLess for $fe {
            fn ct_lt(&self, other: &Self) -> $crate::elliptic_curve::subtle::Choice {
                self.0.ct_lt(&other.0)
            }
        }

        impl $crate::elliptic_curve::zeroize::DefaultIsZeroes for $fe {}

        impl $crate::elliptic_curve::ff::Field for $fe {
            const ZERO: Self = Self::ZERO;
            const ONE: Self = Self::ONE;

            fn try_from_rng<R: $crate::elliptic_curve::rand_core::TryRngCore + ?Sized>(
                rng: &mut R,
            ) -> core::result::Result<Self, R::Error> {
                // NOTE: can't use ScalarPrimitive::random due to CryptoRng bound
                let mut bytes = <$bytes>::default();

                loop {
                    rng.try_fill_bytes(&mut bytes)?;
                    if let Some(fe) = Self::from_bytes(&bytes).into() {
                        return Ok(fe);
                    }
                }
            }

            fn is_zero(&self) -> Choice {
                Self::ZERO.ct_eq(self)
            }

            #[must_use]
            fn square(&self) -> Self {
                self.square()
            }

            #[must_use]
            fn double(&self) -> Self {
                self.double()
            }

            fn invert(&self) -> CtOption<Self> {
                self.invert()
            }

            fn sqrt(&self) -> CtOption<Self> {
                self.sqrt()
            }

            fn sqrt_ratio(num: &Self, div: &Self) -> (Choice, Self) {
                $crate::elliptic_curve::ff::helpers::sqrt_ratio_generic(num, div)
            }
        }

        $crate::impl_field_op!($fe, Add, add, $add);
        $crate::impl_field_op!($fe, Sub, sub, $sub);
        $crate::impl_field_op!($fe, Mul, mul, $mul);

        impl AddAssign<$fe> for $fe {
            #[inline]
            fn add_assign(&mut self, other: $fe) {
                *self = *self + other;
            }
        }

        impl AddAssign<&$fe> for $fe {
            #[inline]
            fn add_assign(&mut self, other: &$fe) {
                *self = *self + other;
            }
        }

        impl SubAssign<$fe> for $fe {
            #[inline]
            fn sub_assign(&mut self, other: $fe) {
                *self = *self - other;
            }
        }

        impl SubAssign<&$fe> for $fe {
            #[inline]
            fn sub_assign(&mut self, other: &$fe) {
                *self = *self - other;
            }
        }

        impl MulAssign<&$fe> for $fe {
            #[inline]
            fn mul_assign(&mut self, other: &$fe) {
                *self = *self * other;
            }
        }

        impl MulAssign for $fe {
            #[inline]
            fn mul_assign(&mut self, other: $fe) {
                *self = *self * other;
            }
        }

        impl Neg for $fe {
            type Output = $fe;

            #[inline]
            fn neg(self) -> $fe {
                Self($neg(self.as_ref()).into())
            }
        }

        impl Sum for $fe {
            #[allow(unused_qualifications)]
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.reduce(core::ops::Add::add).unwrap_or(Self::ZERO)
            }
        }

        impl<'a> Sum<&'a $fe> for $fe {
            fn sum<I: Iterator<Item = &'a $fe>>(iter: I) -> Self {
                iter.copied().sum()
            }
        }

        impl Product for $fe {
            #[allow(unused_qualifications)]
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.reduce(core::ops::Mul::mul).unwrap_or(Self::ONE)
            }
        }

        impl<'a> Product<&'a $fe> for $fe {
            fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.copied().product()
            }
        }
    };
}

/// Emit impls for a `core::ops` trait for all combinations of reference types,
/// which thunk to the given function.
#[macro_export]
macro_rules! impl_field_op {
    ($fe:tt, $op:tt, $op_fn:ident, $func:ident) => {
        impl ::core::ops::$op for $fe {
            type Output = $fe;

            #[inline]
            fn $op_fn(self, rhs: $fe) -> $fe {
                $fe($func(self.as_ref(), rhs.as_ref()).into())
            }
        }

        impl ::core::ops::$op<&$fe> for $fe {
            type Output = $fe;

            #[inline]
            fn $op_fn(self, rhs: &$fe) -> $fe {
                $fe($func(self.as_ref(), rhs.as_ref()).into())
            }
        }

        impl ::core::ops::$op<&$fe> for &$fe {
            type Output = $fe;

            #[inline]
            fn $op_fn(self, rhs: &$fe) -> $fe {
                $fe($func(self.as_ref(), rhs.as_ref()).into())
            }
        }
    };
}

/// Implement Bernstein-Yang field element inversion.
#[macro_export]
macro_rules! impl_bernstein_yang_invert {
    (
        $a:expr,
        $one:expr,
        $d:expr,
        $nlimbs:expr,
        $word:ty,
        $from_mont:ident,
        $mul:ident,
        $neg:ident,
        $divstep_precomp:ident,
        $divstep:ident,
        $msat:ident,
        $selectznz:ident,
    ) => {{
        // See Bernstein-Yang 2019 p.366
        const ITERATIONS: usize = (49 * $d + 57) / 17;

        let a = $from_mont($a);
        let mut d = 1;
        let mut f = $msat();
        let mut g = [0; $nlimbs + 1];
        let mut v = [0; $nlimbs];
        let mut r = $one;
        let mut i = 0;
        let mut j = 0;

        while j < $nlimbs {
            g[j] = a[j];
            j += 1;
        }

        while i < ITERATIONS - ITERATIONS % 2 {
            let (out1, out2, out3, out4, out5) = $divstep(d, &f, &g, &v, &r);
            let (out1, out2, out3, out4, out5) = $divstep(out1, &out2, &out3, &out4, &out5);
            d = out1;
            f = out2;
            g = out3;
            v = out4;
            r = out5;
            i += 2;
        }

        if ITERATIONS % 2 != 0 {
            let (_out1, out2, _out3, out4, _out5) = $divstep(d, &f, &g, &v, &r);
            v = out4;
            f = out2;
        }

        let s = ((f[f.len() - 1] >> <$word>::BITS - 1) & 1) as u8;
        let v = $selectznz(s, &v, &$neg(&v));
        $mul(&v, &$divstep_precomp())
    }};
}

/// Implement field element identity tests.
#[macro_export]
macro_rules! impl_field_identity_tests {
    ($fe:tt) => {
        #[test]
        fn zero_is_additive_identity() {
            let zero = $fe::ZERO;
            let one = $fe::ONE;
            assert_eq!(zero.add(&zero), zero);
            assert_eq!(one.add(&zero), one);
        }

        #[test]
        fn one_is_multiplicative_identity() {
            let one = $fe::ONE;
            assert_eq!(one.multiply(&one), one);
        }
    };
}

/// Implement field element inversion tests.
#[macro_export]
macro_rules! impl_field_invert_tests {
    ($fe:tt) => {
        #[test]
        fn invert() {
            let one = $fe::ONE;
            assert_eq!(one.invert().unwrap(), one);

            let three = one + &one + &one;
            let inv_three = three.invert().unwrap();
            assert_eq!(three * &inv_three, one);

            let minus_three = -three;
            let inv_minus_three = minus_three.invert().unwrap();
            assert_eq!(inv_minus_three, -inv_three);
            assert_eq!(three * &inv_minus_three, -one);
        }
    };
}

/// Implement field element square root tests.
#[macro_export]
macro_rules! impl_field_sqrt_tests {
    ($fe:tt) => {
        #[test]
        fn sqrt() {
            for &n in &[1u64, 4, 9, 16, 25, 36, 49, 64] {
                let fe = $fe::from(n);
                let sqrt = fe.sqrt().unwrap();
                assert_eq!(sqrt.square(), fe);
            }
        }
    };
}

/// Implement tests for the `PrimeField` trait.
#[macro_export]
macro_rules! impl_primefield_tests {
    ($fe:tt, $t:expr) => {
        #[test]
        fn two_inv_constant() {
            assert_eq!($fe::from(2u32) * $fe::TWO_INV, $fe::ONE);
        }

        #[test]
        fn root_of_unity_constant() {
            assert!($fe::S < 128);
            let two_to_s = 1u128 << $fe::S;

            // ROOT_OF_UNITY^{2^s} mod m == 1
            assert_eq!(
                $fe::ROOT_OF_UNITY.pow_vartime(&[
                    (two_to_s & 0xFFFFFFFFFFFFFFFF) as u64,
                    (two_to_s >> 64) as u64,
                    0,
                    0
                ]),
                $fe::ONE
            );

            // MULTIPLICATIVE_GENERATOR^{t} mod m == ROOT_OF_UNITY
            assert_eq!(
                $fe::MULTIPLICATIVE_GENERATOR.pow_vartime(&$t),
                $fe::ROOT_OF_UNITY
            )
        }

        #[test]
        fn root_of_unity_inv_constant() {
            assert_eq!($fe::ROOT_OF_UNITY * $fe::ROOT_OF_UNITY_INV, $fe::ONE);
        }

        #[test]
        fn delta_constant() {
            // DELTA^{t} mod m == 1
            assert_eq!($fe::DELTA.pow_vartime(&$t), $fe::ONE);
        }
    };
}
