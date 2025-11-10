use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Neg, Sub, SubAssign};

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Flint {
    pub raw: i32,
}

impl Flint {
    pub const POINT_FIVE: u16 = Self::HALF_SCALE as u16;
    pub const POINT_ONE: u16 = Self::POINT_FIVE / 5;

    pub const ZERO: Self = Self::new(0, 0);
    pub const ONE: Self = Self::new(1, 0);
    pub const NEG_ONE: Self = Self::new(-1, 0);

    pub const PI: Self = Self::from_raw((31415 * Self::SCALE) / 10000);
    pub const FRAC_PI_2: Self = Self::from_raw(Self::PI.raw / 2);
    pub const DEG2RAD: Self = Self::from_raw(Self::PI.raw / 180);

    const SHIFT: i32 = 16;
    const SCALE: i32 = 1 << Self::SHIFT;
    const HALF_SCALE: i32 = Self::SCALE >> 1;
    const INV_SCALE: f32 = 1.0 / Self::SCALE as f32;

    const CORDIC_GAIN: i32 = 39796;
    const CORDIC_ATAN: [i32; 16] = [
        51471, 30385, 16054, 8149, 4090, 2047, 1023, 511, 255, 127, 63, 31, 16, 8, 4, 2,
    ];

    #[inline]
    pub const fn new(value: i16, fraction: u16) -> Self {
        Self {
            raw: (value as i32) << Self::SHIFT | fraction as i32,
        }
    }

    #[inline]
    pub const fn from_raw(raw: i32) -> Self {
        Self { raw }
    }

    #[inline]
    pub const fn from_i16(value: i16) -> Self {
        Self {
            raw: (value as i32) << Self::SHIFT,
        }
    }

    #[inline]
    pub const fn to_i16(self) -> i16 {
        (self.raw >> Self::SHIFT) as i16
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        self.raw >> Self::SHIFT
    }

    #[inline]
    pub const fn to_f32(self) -> f32 {
        self.raw as f32 * Self::INV_SCALE
    }

    #[inline]
    pub fn sqrt(self) -> Self {
        if self.raw <= 0 {
            return Self::ZERO;
        }

        let mut raw = 0;
        let mut bit = 1 << (2 * Self::SHIFT - 2);
        let mut copy = self.raw;

        while bit > self.raw {
            bit >>= 2;
        }

        while bit != 0 {
            let tmp = raw + bit;

            if copy >= tmp {
                copy -= tmp;
                raw = tmp + bit;
            }

            raw >>= 1;
            bit >>= 2;
        }

        raw <<= Self::SHIFT >> 1;

        Self { raw }
    }

    #[inline]
    pub fn to_radians(self) -> Self {
        self * Self::DEG2RAD
    }

    #[inline]
    pub fn sin_cos(self) -> (Self, Self) {
        let mut z = self.raw;
        let mut negative = false;

        while z > Self::FRAC_PI_2.raw {
            z -= Self::PI.raw;
            negative = !negative;
        }

        while z < -Self::FRAC_PI_2.raw {
            z += Self::PI.raw;
            negative = !negative;
        }

        let mut x = Self::CORDIC_GAIN;
        let mut y = 0;

        for i in 0..16 {
            let direction = if z < 0 { -1 } else { 1 };
            let xx = x;

            x -= direction * (y >> i);
            y += direction * (xx >> i);
            z -= direction * Self::CORDIC_ATAN[i];
        }

        if negative {
            x = -x;
            y = -y;
        }

        (Self { raw: y }, Self { raw: x })
    }
}

impl Add for Flint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            raw: self.raw + rhs.raw,
        }
    }
}

impl AddAssign for Flint {
    fn add_assign(&mut self, rhs: Self) {
        self.raw += rhs.raw;
    }
}

impl AddAssign<i16> for Flint {
    fn add_assign(&mut self, rhs: i16) {
        self.raw += (rhs as i32) << Self::SHIFT;
    }
}

impl SubAssign for Flint {
    fn sub_assign(&mut self, rhs: Self) {
        self.raw -= rhs.raw;
    }
}

impl Mul<i16> for Flint {
    type Output = Self;

    fn mul(self, rhs: i16) -> Self::Output {
        self * Flint::from_i16(rhs)
    }
}

impl Sub for Flint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            raw: self.raw - rhs.raw,
        }
    }
}

impl Mul for Flint {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            raw: (((self.raw as i64) * (rhs.raw as i64)) / (Self::SCALE as i64)) as i32,
        }
    }
}

impl Div for Flint {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            raw: ((self.raw as i64) * (Self::SCALE as i64) / rhs.raw as i64) as i32,
        }
    }
}

impl DivAssign for Flint {
    fn div_assign(&mut self, rhs: Self) {
        self.raw = ((self.raw as i64) * (Self::SCALE as i64) / rhs.raw as i64) as i32;
    }
}

impl Neg for Flint {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { raw: -self.raw }
    }
}

impl From<i16> for Flint {
    fn from(value: i16) -> Self {
        Flint::from_i16(value)
    }
}

impl Into<f32> for Flint {
    fn into(self) -> f32 {
        self.to_f32()
    }
}
