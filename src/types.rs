#[allow(unused_imports)]
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use derive_header::MutViewNew;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::rc::Rc;
#[allow(unused_attributes)]
#[macro_use]
#[allow(unused_imports)]
#[macro_use]
use assert_hex::assert_eq_hex;
///###
#[derive(Debug, PartialEq)]
pub struct SimpleVal<'a, T>
where
    Self: ModSimpleVal<'a, T>,
{
    val: &'a mut [u8],
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> SimpleVal<'a, T>
where
    Self: ModSimpleVal<'a, T>,
{
    pub fn new(arr: &'a mut [u8]) -> (Self, &'a mut [u8]) {
        let (val, leftover) = arr.split_at_mut(std::mem::size_of::<T>());

        (
            Self {
                val,
                _marker: std::marker::PhantomData::<T>,
            },
            leftover,
        )
    }
}

impl<'a, T> ArrayVal<'a, T> {
    pub fn new(arr: &'a mut [u8]) -> (Self, &'a mut [u8]) {
        let (val, leftover) = arr.split_at_mut(std::mem::size_of::<T>());

        (
            Self {
                buf: Rc::new(RefCell::new(val)),
                _marker: std::marker::PhantomData::<T>,
            },
            leftover,
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct ArrayVal<'a, T> {
    buf: Rc<RefCell<&'a mut [u8]>>,
    _marker: std::marker::PhantomData<T>,
}

pub trait ModSimpleVal<'a, T> {
    fn val(&self) -> T;
    fn set(&mut self, v: T);
}

impl<'a> ModSimpleVal<'a, u8> for SimpleVal<'a, u8> {
    fn val(&self) -> u8 {
        self.val[0]
    }

    fn set(&mut self, v: u8) {
        self.val[0] = v
    }
}

impl<'a, const L: usize> ArrayVal<'a, [u8; L]> {
    pub fn as_mut_ref(&self) -> RefMut<&'a mut [u8]> {
        self.buf.borrow_mut()
    }

    pub fn as_ref(&self) -> Ref<&'a mut [u8]> {
        self.buf.borrow()
    }

    pub fn rc_clone(&self) -> Rc<RefCell<&'a mut [u8]>> {
        self.buf.clone()
    }

    pub fn set(&mut self, src: &[u8]) {
        let mut dst = self.buf.borrow_mut();

        for i in 0..L {
            dst[i] = src[i]
        }
    }
}

#[macro_export]
macro_rules! impl_modSimpleVal {
    ($target:tt, $type:tt, $read:ident, $write:ident, $endian:ident) => {
        impl<'a> ModSimpleVal<'a, $type> for $target<'a, $type> {
            fn val(&self) -> $type {
                $endian::$read(self.val)
            }

            fn set(&mut self, v: $type) {
                $endian::$write(self.val, v)
            }
        }
    };
}

impl_modSimpleVal!(SimpleVal, u16, read_u16, write_u16, LittleEndian);
impl_modSimpleVal!(SimpleVal, u32, read_u32, write_u32, LittleEndian);
impl_modSimpleVal!(SimpleVal, u64, read_u64, write_u64, LittleEndian);

#[macro_export]
macro_rules! impl_oper_assign_overload {
    ($oper_name:ident, $bound:ident, $fname:ident, $oper:tt, $gen:tt) => {
        impl<'a, $gen> $oper_name<$gen> for SimpleVal<'a, $gen>
        where
            SimpleVal<'a, $gen>: ModSimpleVal<'a, $gen>,
            $gen: $bound + $bound<Output = $gen>,
        {
            fn $fname(&mut self, rhs: $gen) {
                self.set(self.val() $oper rhs)
            }
        }
    };
}

impl_oper_assign_overload!(AddAssign, Add, add_assign, +, T);
impl_oper_assign_overload!(SubAssign, Sub, sub_assign, -, T);
impl_oper_assign_overload!(MulAssign, Mul, mul_assign, *, T);
impl_oper_assign_overload!(DivAssign, Div, div_assign, /, T);

#[allow(dead_code)]
#[derive(MutViewNew)]
struct SimpleValTest<'a> {
    pub unsigned_8: SimpleVal<'a, u8>,
    pub unsigned_16: SimpleVal<'a, u16>,
    pub unsigned_32: SimpleVal<'a, u32>,
    pub unsigned_64: SimpleVal<'a, u64>,
    pub unsigned_arr: ArrayVal<'a, [u8; 4]>,
}

#[allow(dead_code)]
const SIMPLEVAL_TESTDATA: [u8; 0x13] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13,
];

#[test]
fn simpleval_val() {
    let mut buf = SIMPLEVAL_TESTDATA.to_vec();
    let (t, _): (SimpleValTest, _) = SimpleValTest::new(&mut buf);

    assert_eq_hex!(t.unsigned_8.val(), 01);
    assert_eq_hex!(t.unsigned_16.val(), 0x0302);
    assert_eq_hex!(t.unsigned_32.val(), 0x07060504);
    assert_eq_hex!(t.unsigned_64.val(), 0x0F0E0D0C0B0A0908);
}

#[test]
fn arrayval_deref() {
    let arr = [0x01, 0x02, 0x03, 0x04];
    let mut buf = arr.clone();
    let (t, _): (ArrayVal<[u8; 4]>, _) = ArrayVal::new(&mut buf);

    assert_eq_hex!(*t.as_ref(), arr);
}

#[test]
fn arrayval_deref_mut() {
    let mut buf = [0x01];
    let (t, _): (ArrayVal<[u8; 1]>, _) = ArrayVal::new(&mut buf);

    t.as_mut_ref()[0] = 0x0A;

    assert_eq_hex!(*t.as_ref(), [0x0A]);
}

/*
#[test]
fn SimpleVal_set() -> Result<(), ()> {
    let data = SimpleVal_TESTDATA.to_vec();
    let mut buf = std::io::Cursor::new(data);

    let mut SimpleValtest = SimpleValTest::new(&mut buf).map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_8
        .set(&mut buf, 0x13)
        .map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_16
        .set(&mut buf, 0x1112)
        .map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_32
        .set(&mut buf, 0x0D0E0F10)
        .map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_64
        .set(&mut buf, 0x05060708090A0B0C)
        .map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_u8_arr
        .set(&mut buf, [04, 03, 02, 01])
        .map_err(|e| eprintln!("{}", e))?;

    let data_ref = buf.get_ref();

    assert_eq_hex!(data_ref[0], 0x13);
    assert_eq_hex!(data_ref[1..3], [0x12, 0x11]);
    assert_eq_hex!(data_ref[3..7], [0x10, 0xF, 0xE, 0xD]);
    assert_eq_hex!(data_ref[7..15], [0xC, 0xB, 0xA, 0x9, 0x8, 0x7, 0x6, 0x5]);
    assert_eq_hex!(data_ref[15..19], [0x4, 0x3, 0x2, 0x1]);

    assert_eq_hex!(*SimpleValtest.unsigned_8, 0x13);
    assert_eq_hex!(*SimpleValtest.unsigned_16, 0x1112);
    assert_eq_hex!(*SimpleValtest.unsigned_32, 0x0D0E0F10);
    assert_eq_hex!(*SimpleValtest.unsigned_64, 0x05060708090A0B0C);
    assert_eq_hex!(*SimpleValtest.unsigned_u8_arr, [0x4, 0x3, 0x2, 0x1]);

    Ok(())
}

#[test]
fn SimpleVal_add() -> Result<(), ()> {
    let data = SimpleVal_TESTDATA.to_vec();
    let mut buf = std::io::Cursor::new(data);

    let mut SimpleValtest = SimpleValTest::new(&mut buf).map_err(|e| eprintln!("{}", e))?;
    const VAL_TO_ADD: u8 = 0x10;

    // For the moment, we are not using Add for arrays

    SimpleValtest
        .unsigned_8
        .add(&mut buf, VAL_TO_ADD)
        .map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_16
        .add(&mut buf, VAL_TO_ADD as u16)
        .map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_32
        .add(&mut buf, VAL_TO_ADD as u32)
        .map_err(|e| eprintln!("{}", e))?;

    SimpleValtest
        .unsigned_64
        .add(&mut buf, VAL_TO_ADD as u64)
        .map_err(|e| eprintln!("{}", e))?;

    let data_ref = buf.get_ref();

    assert_eq_hex!(data_ref[0], 0x11);
    assert_eq_hex!(LittleEndian::read_u16(&data_ref[1..3]), 0x0312);
    assert_eq_hex!(LittleEndian::read_u32(&data_ref[3..7]), 0x07060514);
    assert_eq_hex!(LittleEndian::read_u64(&data_ref[7..15]), 0x0F0E0D0C0B0A0918);
    //assert_eq_hex!(data_ref[15..19], [0x4, 0x3, 0x2, 0x1]);

    Ok(())
}
*/
