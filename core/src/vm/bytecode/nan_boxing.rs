use super::VmValue;

const QNAN: u64 = 0x7FFC_0000_0000_0000;
const TAG_NULL: u64 = QNAN | 0x01;
const TAG_UNDEF: u64 = QNAN | 0x02;
const TAG_TRUE: u64 = QNAN | 0x03;
const TAG_FALSE: u64 = QNAN | 0x04;

const SMI_TAG: u64 = 0x7FFD_0000_0000_0000;
const PTR_TAG: u64 = 0x7FFE_0000_0000_0000;
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const PTR_PAYLOAD: u64 = 0x0000_FFFF_FFFF_FFFF;

#[derive(Clone, Copy, Debug)]
pub struct NanBoxedValue(u64);

pub enum Decoded {
    Number(f64),
    Int(i32),
    Bool(bool),
    Null,
    Undefined,
    Pointer(usize),
}

impl NanBoxedValue {
    pub fn from_f64(n: f64) -> Self {
        let bits = n.to_bits();
        if n.is_nan() {
            return Self(QNAN);
        }
        Self(bits)
    }

    pub fn from_bool(b: bool) -> Self {
        if b {
            Self(TAG_TRUE)
        } else {
            Self(TAG_FALSE)
        }
    }

    pub fn null() -> Self {
        Self(TAG_NULL)
    }

    pub fn undefined() -> Self {
        Self(TAG_UNDEF)
    }

    pub fn from_int(i: i32) -> Self {
        Self(SMI_TAG | (i as u32 as u64))
    }

    pub fn from_pointer(idx: usize) -> Self {
        debug_assert!(idx as u64 <= PTR_PAYLOAD);
        Self(PTR_TAG | (idx as u64 & PTR_PAYLOAD))
    }

    pub fn decode(self) -> Decoded {
        let bits = self.0;
        if bits == TAG_NULL {
            return Decoded::Null;
        }
        if bits == TAG_UNDEF {
            return Decoded::Undefined;
        }
        if bits == TAG_TRUE {
            return Decoded::Bool(true);
        }
        if bits == TAG_FALSE {
            return Decoded::Bool(false);
        }
        if bits & TAG_MASK == SMI_TAG {
            return Decoded::Int(bits as u32 as i32);
        }
        if bits & TAG_MASK == PTR_TAG {
            return Decoded::Pointer((bits & PTR_PAYLOAD) as usize);
        }
        Decoded::Number(f64::from_bits(bits))
    }

    pub fn raw(self) -> u64 {
        self.0
    }

    pub fn is_number(self) -> bool {
        matches!(self.decode(), Decoded::Number(_) | Decoded::Int(_))
    }

    pub fn to_f64(self) -> f64 {
        match self.decode() {
            Decoded::Number(n) => n,
            Decoded::Int(i) => i as f64,
            Decoded::Bool(true) => 1.0,
            Decoded::Bool(false) | Decoded::Null => 0.0,
            Decoded::Undefined | Decoded::Pointer(_) => f64::NAN,
        }
    }

    pub fn to_bool(self) -> bool {
        match self.decode() {
            Decoded::Bool(b) => b,
            Decoded::Null | Decoded::Undefined => false,
            Decoded::Number(n) => n != 0.0 && !n.is_nan(),
            Decoded::Int(i) => i != 0,
            Decoded::Pointer(_) => true,
        }
    }
}

pub struct HeapStore {
    objects: Vec<VmValue>,
}

impl HeapStore {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn alloc(&mut self, value: VmValue) -> NanBoxedValue {
        let idx = self.objects.len();
        self.objects.push(value);
        NanBoxedValue::from_pointer(idx)
    }

    pub fn get(&self, idx: usize) -> &VmValue {
        &self.objects[idx]
    }
}

impl NanBoxedValue {
    pub fn encode(value: &VmValue, heap: &mut HeapStore) -> Self {
        match value {
            VmValue::Undefined => Self::undefined(),
            VmValue::Null => Self::null(),
            VmValue::Boolean(b) => Self::from_bool(*b),
            VmValue::Number(n) => {
                let i = *n as i32;
                if i as f64 == *n && *n != 0.0 {
                    Self::from_int(i)
                } else if *n == 0.0 && !n.is_sign_negative() {
                    Self::from_int(0)
                } else {
                    Self::from_f64(*n)
                }
            }
            VmValue::String(_) | VmValue::Function(_) => heap.alloc(value.clone()),
        }
    }

    pub fn decode_to_vm(self, heap: &HeapStore) -> VmValue {
        match self.decode() {
            Decoded::Undefined => VmValue::Undefined,
            Decoded::Null => VmValue::Null,
            Decoded::Bool(b) => VmValue::Boolean(b),
            Decoded::Number(n) => VmValue::Number(n),
            Decoded::Int(i) => VmValue::Number(i as f64),
            Decoded::Pointer(idx) => heap.get(idx).clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_f64() {
        let v = NanBoxedValue::from_f64(3.14);
        assert!(matches!(v.decode(), Decoded::Number(n) if (n - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn roundtrip_smi() {
        let v = NanBoxedValue::from_int(42);
        assert!(matches!(v.decode(), Decoded::Int(42)));
    }

    #[test]
    fn roundtrip_negative_smi() {
        let v = NanBoxedValue::from_int(-7);
        assert!(matches!(v.decode(), Decoded::Int(-7)));
    }

    #[test]
    fn roundtrip_primitives() {
        assert!(matches!(NanBoxedValue::null().decode(), Decoded::Null));
        assert!(matches!(
            NanBoxedValue::undefined().decode(),
            Decoded::Undefined
        ));
        assert!(matches!(
            NanBoxedValue::from_bool(true).decode(),
            Decoded::Bool(true)
        ));
        assert!(matches!(
            NanBoxedValue::from_bool(false).decode(),
            Decoded::Bool(false)
        ));
    }

    #[test]
    fn roundtrip_pointer() {
        let v = NanBoxedValue::from_pointer(12345);
        assert!(matches!(v.decode(), Decoded::Pointer(12345)));
    }

    #[test]
    fn encode_decode_vm_values() {
        let mut heap = HeapStore::new();
        let cases = vec![
            VmValue::Undefined,
            VmValue::Null,
            VmValue::Boolean(true),
            VmValue::Boolean(false),
            VmValue::Number(42.0),
            VmValue::Number(3.14),
            VmValue::Number(-0.0),
            VmValue::String("hello".to_string()),
        ];
        for original in &cases {
            let boxed = NanBoxedValue::encode(original, &mut heap);
            let decoded = boxed.decode_to_vm(&heap);
            assert_eq!(original.to_output(), decoded.to_output());
        }
    }
}
