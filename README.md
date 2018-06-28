# x86-mp

A crate for parsing the Intel MP tables.

## Example

     let mp_hdr_loc = KERNEL_BASE + mp_ptr.physical_address_pointer as usize;
     let mp_hdr: MPConfigurationTableHeader = *(mp_hdr_loc as *const MPConfigurationTableHeader);
     log!("MP header has {} entries, at 0x{:016x}, LAPIC at 0x{:016x}",
          mp_hdr.entry_count, mp_hdr_loc, mp_hdr.local_apic_addr);

     let mp_hdr_iter = mp_hdr.iter(mp_hdr_loc);
     for i in mp_hdr_iter {
         if i.code == MPEntryCode::Processor {
             processors += 1;
         }
     }
     log!("Found {} processors in total", processors);
