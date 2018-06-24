extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

/// The MP Floating Pointer Structure as defined in [MPSpec] Section 4.1
#[repr(C, packed)]
pub struct MPFloatingPointer {
    pub signature: u32,
    pub physical_address_pointer: u32,
    pub length: u8,
    pub spec_rev: u8,
    pub checksum: u8,
    pub mp_feature_info_bytes: [u8; 5],
}

/// The signature that should match MPFloatingPointer's signature field
/// The value should be the ASCII representation of "_MP_" (excluding quotes)
pub const MPFLOATINGPOINTER_SIGNATURE: [u8; 4] = [95, 77, 80, 95];

impl MPFloatingPointer {
    pub fn verify_checksum(&self) -> bool {
        let mut checksum = 0;
        let mut buf = [0; 16];

        /* convert the two 32-bit numbers into an array of bytes */
        LittleEndian::write_u32(&mut buf, self.signature);
        LittleEndian::write_u32(&mut buf[4..7], self.physical_address_pointer);

        /* Add everything up */
        checksum += buf.iter().map(|&b| usize::from(b)).sum::<usize>();
        checksum += self.length as usize;
        checksum += self.spec_rev as usize;
        checksum += self.checksum as usize;
        checksum += self.mp_feature_info_bytes.iter().map(|&b| usize::from(b)).sum::<usize>();

        /* The [MPSpec] says (Table 4-1.) that the checksum is valid if
         * all the bytes add up to zero.
         */
        return (checksum & 0x0f) == 0
    }

    pub fn verify_signature(&self) -> bool {
        let mut ascii = [0; 4];
        LittleEndian::write_u32(&mut ascii, self.signature);
        ascii == MPFLOATINGPOINTER_SIGNATURE
    }

    pub fn is_valid(&self) -> bool {
        self.verify_checksum() && self.verify_signature()
    }
}
