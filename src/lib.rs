#![no_std]
extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

/// The MP Floating Pointer Structure as defined in [MPSpec] Section 4.1
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
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

/// The signature that should match MPConfigurationTableHeader's signature field
/// The value should be the ASCII representation of "PCMP" (excluding quotes)
pub const MPCONFIGURATIONTABLEHEADER_SIGNATURE: [u8; 4] = [80, 67, 77, 80];

impl MPFloatingPointer {
    pub fn verify_checksum(&self) -> bool {
        let mut checksum = 0;
        let mut buf = [0; 16];

        /* convert the two 32-bit numbers into an array of bytes */
        LittleEndian::write_u32(&mut buf, self.signature);
        LittleEndian::write_u32(&mut buf[4..8], self.physical_address_pointer);

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

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MPConfigurationTableHeader {
    pub signature: u32,
    pub base_table_length: u16,
    pub spec_rev: u8,
    pub checksum: u8,
    pub oem_id: [u8; 8],
    pub product_id: [u8; 12],
    pub oem_table_pointer: u32,
    pub oem_table_size: u16,
    pub entry_count: u16,
    pub local_apic_addr: u32,
    pub extended_table_length: u16,
    pub extended_table_checksum: u8,
}

impl MPConfigurationTableHeader {

    pub fn verify_checksum(&self) -> bool {
        /* FIXME: Actually verify this */
        true
    }

    pub fn verify_signature(&self) -> bool {
        let mut ascii = [0; 4];
        LittleEndian::write_u32(&mut ascii, self.signature);
        ascii == MPCONFIGURATIONTABLEHEADER_SIGNATURE
    }

    pub fn is_valid(&self) -> bool {
        self.verify_checksum() && self.verify_signature()
    }

    pub fn iter(&self, table_location: usize) -> EntryIterator {
        EntryIterator {
            table_location: table_location + 44,
            total_entries: self.entry_count as usize,
            entries_sofar: 0,
            current_offset: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MPEntryCode {
    Processor = 0,
    Bus = 1,
    IOAPIC = 2,
    IOInterruptAssignment = 3,
    LocalInterruptAssignment = 4,
    Unknown = 5,
}

impl MPEntryCode {
    pub fn length(&self) -> usize
    {
        match self {
            MPEntryCode::Processor => 20,
            MPEntryCode::Bus => 8,
            MPEntryCode::IOAPIC => 8,
            MPEntryCode::IOInterruptAssignment => 8,
            MPEntryCode::LocalInterruptAssignment => 8,
            MPEntryCode::Unknown =>
                panic!("Trying to get length of unknown MP entry: {:?}", self),
        }
    }

    pub fn from_u8(num: u8) -> Self {
        match num {
            0 => MPEntryCode::Processor,
            1 => MPEntryCode::Bus,
            2 => MPEntryCode::IOAPIC,
            3 => MPEntryCode::IOInterruptAssignment,
            4 => MPEntryCode::LocalInterruptAssignment,
            _ => MPEntryCode::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct EntryIterator {
    table_location: usize,
    entries_sofar: usize,
    total_entries: usize,
    current_offset: usize,
}

impl Iterator for EntryIterator {
    type Item = MPEntryCode;

    fn next(&mut self) -> Option<MPEntryCode> {
        if self.entries_sofar >= self.total_entries {
            None
        } else {
            let current_addr = self.table_location + self.current_offset;
            let current_ptr = current_addr as *const u8;
            let current_code: MPEntryCode = unsafe { MPEntryCode::from_u8(*current_ptr) };
            self.entries_sofar += 1;
            self.current_offset += current_code.length();
            Some(current_code)
        }
    }
}

union MPPossibleEntries {
    processor: ProcessorEntry,
    bus: BusEntry,
    ioapic: IOAPICEntry,
    io_interrupt_assignment: IOInterruptAssignmentEntry,
    local_interrupt_assignment: LocalInterruptAssignmentEntry,
}

pub struct MPEntry {
    pub code: MPEntryCode,
    entries: MPPossibleEntries,
}

impl MPEntry {
    pub fn get_processor_entry(&self) -> Option<ProcessorEntry> {
        if self.code == MPEntryCode::Processor {
            Some(unsafe { self.entries.processor })
        } else {
            None
        }
    }

    pub fn get_bus_entry(&self) -> Option<BusEntry> {
        if self.code == MPEntryCode::Bus {
            Some(unsafe { self.entries.bus })
        } else {
            None
        }
    }

    pub fn get_ioapic_entry(&self) -> Option<IOAPICEntry> {
        if self.code == MPEntryCode::IOAPIC {
            Some(unsafe { self.entries.ioapic})
        } else {
            None
        }
    }

    pub fn get_io_interrupt_assignment_entry(&self) -> Option<IOInterruptAssignmentEntry> {
        if self.code == MPEntryCode::IOInterruptAssignment {
            Some(unsafe { self.entries.io_interrupt_assignment })
        } else {
            None
        }
    }

    pub fn get_local_interrupt_assignment_entry(&self) -> Option<LocalInterruptAssignmentEntry> {
        if self.code == MPEntryCode::LocalInterruptAssignment {
            Some(unsafe { self.entries.local_interrupt_assignment })
        } else {
            None
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct ProcessorEntry {
    pub entry_type: u8,
    pub lapic_id: u8,
    pub lapic_version: u8,
    pub cpu_flags: u8,
    pub cpu_signature: [u8; 2],
    unused: [u8; 2],
    pub feature_flags: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct BusEntry {
    pub entry_type: u8,
    pub bus_id: u8,
    pub bus_type_string: [u8; 6],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct IOAPICEntry {
    pub entry_type: u8,
    pub ioapic_id: u8,
    pub ioapic_version: u8,
    pub ioapic_flags: u8,
    pub ioapic_address: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct IOInterruptAssignmentEntry {
    pub entry_type: u8,
    pub interrupt_type: u8,
    pub interrupt_mode: u8,
    unused: u8,
    pub source_bus_id: u8,
    pub source_bus_irq: u8,
    pub dest_ioapic_id: u8,
    pub dest_ioapic_int: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct LocalInterruptAssignmentEntry {
    pub entry_type: u8,
    pub interrupt_type: u8,
    pub interrupt_mode: u8,
    unused: u8,
    pub source_bus_id: u8,
    pub source_bus_irq: u8,
    pub dest_ioapic_id: u8,
    pub dest_ioapic_int: u8,
}
