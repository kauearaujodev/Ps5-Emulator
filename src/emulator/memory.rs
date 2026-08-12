use std::collections::HashMap;

pub const VIRTUAL_RAM_SIZE: u64 =
    5 * 1024 * 1024 * 1024 * 1024; // 5 TiB

pub const PAGE_SIZE: u64 = 4096; // 4 KiB

pub struct VirtualMemory {
    size: u64,

    pages: HashMap<u64, Box<[u8; PAGE_SIZE as usize]>>,
}

impl VirtualMemory {

    pub fn new() -> Self {
        Self {
            size: VIRTUAL_RAM_SIZE,
            pages: HashMap::new(),
        }
    }

    fn check_address(&self, address: u64) -> Result<(), String> {

        if address >= self.size {
            return Err(format!(
                "Endereço fora da RAM virtual: 0x{:X}",
                address
            ));
        }

        Ok(())
    }

    fn page_number(address: u64) -> u64 {
        address / PAGE_SIZE
    }

    fn page_offset(address: u64) -> usize {
        (address % PAGE_SIZE) as usize
    }

    fn get_page_mut(
        &mut self,
        page: u64,
    ) -> &mut Box<[u8; PAGE_SIZE as usize]> {

        self.pages
            .entry(page)
            .or_insert_with(|| {
                Box::new([0u8; PAGE_SIZE as usize])
            })
    }

    pub fn write8(
        &mut self,
        address: u64,
        value: u8,
    ) -> Result<(), String> {

        self.check_address(address)?;

        let page = Self::page_number(address);
        let offset = Self::page_offset(address);

        let memory_page = self.get_page_mut(page);

        memory_page[offset] = value;

        Ok(())
    }

    pub fn read8(
        &mut self,
        address: u64,
    ) -> Result<u8, String> {

        self.check_address(address)?;

        let page = Self::page_number(address);
        let offset = Self::page_offset(address);

        let memory_page = self.get_page_mut(page);

        Ok(memory_page[offset])
    }

    pub fn allocated_pages(&self) -> usize {
        self.pages.len()
    }

    pub fn allocated_bytes(&self) -> u64 {
        self.pages.len() as u64 * PAGE_SIZE
    }

    pub fn virtual_size(&self) -> u64 {
        self.size
    }
}
