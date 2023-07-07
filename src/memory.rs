// use x86_64::{
//     structures::paging::PageTable,
//     structures::paging::OffsetPageTable,
//     VirtAddr,
// };

// /// 初始化一个新的OffsetPageTable。
// ///
// /// 这个函数是不安全的，因为调用者必须保证完整的物理内存在
// /// 传递的`physical_memory_offset`处被映射到虚拟内存。另
// /// 外，这个函数必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
// pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
//     let level_4_table = active_level_4_table(physical_memory_offset);
//     OffsetPageTable::new(level_4_table, physical_memory_offset)
// }



// /// 返回一个对活动的4级表的可变引用。
// ///
// /// 这个函数是不安全的，因为调用者必须保证完整的物理内存在传递的 
// /// `physical_memory_offset`处被映射到虚拟内存。另外，这个函数
// /// 必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
// pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
//     -> &'static mut PageTable
// {
//     use x86_64::registers::control::Cr3;

//     let (level_4_table_frame, _) = Cr3::read();

//     let phys = level_4_table_frame.start_address();
//     let virt = physical_memory_offset + phys.as_u64();
//     let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

//     &mut *page_table_ptr // unsafe
// }

// use x86_64::PhysAddr;

// /// 将给定的虚拟地址转换为映射的物理地址，如果地址没有被映射，则为`None'。
// ///
// /// 这个函数是不安全的，因为调用者必须保证完整的物理内存在传递的`physical_memory_offset`处被映射到虚拟内存。
// pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
//     -> Option<PhysAddr>
// {
//     translate_addr_inner(addr, physical_memory_offset)
// }

// /// 由 `translate_addr`调用的私有函数。
// ///
// /// 这个函数是安全的，可以限制`unsafe`的范围，
// /// 因为Rust将不安全函数的整个主体视为不安全块。
// /// 这个函数只能通过`unsafe fn`从这个模块的外部到达。
// fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
//     -> Option<PhysAddr>
// {
//     use x86_64::structures::paging::page_table::FrameError;
//     use x86_64::registers::control::Cr3;

//     // 从CR3寄存器中读取活动的4级 frame
//     let (level_4_table_frame, _) = Cr3::read();

//     let table_indexes = [
//         addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
//     ];
//     let mut frame = level_4_table_frame;

//     // 遍历多级页表
//     for &index in &table_indexes {
//         // 将该框架转换为页表参考
//         let virt = physical_memory_offset + frame.start_address().as_u64();
//         let table_ptr: *const PageTable = virt.as_ptr();
//         let table = unsafe {&*table_ptr};

//         // 读取页表条目并更新`frame`。
//         let entry = &table[index];
//         frame = match entry.frame() {
//             Ok(frame) => frame,
//             Err(FrameError::FrameNotPresent) => return None,
//             Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
//         };
//     }

//     // 通过添加页面偏移量来计算物理地址
//     Some(frame.start_address() + u64::from(addr.page_offset()))
// }


// use x86_64::{
//     structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator}
// };

// /// 为给定的页面创建一个实例映射到框架`0xb8000`。
// pub fn create_example_mapping(
//     page: Page,
//     mapper: &mut OffsetPageTable,
//     frame_allocator: &mut impl FrameAllocator<Size4KiB>,
// ) {
//     use x86_64::structures::paging::PageTableFlags as Flags;

//     let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
//     let flags = Flags::PRESENT | Flags::WRITABLE;

//     let map_to_result = unsafe {
//         // FIXME: 这并不安全，我们这样做只是为了测试。
//         mapper.map_to(page, frame, flags, frame_allocator)
//     };
//     map_to_result.expect("map_to failed").flush();
// }

// pub struct EmptyFrameAllocator;

// unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame> {
//         None
//     }
// }

// use bootloader::bootinfo::MemoryMap;
// use bootloader::bootinfo::MemoryRegionType;
// /// 一个FrameAllocator，从bootloader的内存地图中返回可用的 frames。
// pub struct BootInfoFrameAllocator {
//     memory_map: &'static MemoryMap,
//     next: usize,
// }

// impl BootInfoFrameAllocator {
//     /// 从传递的内存 map 中创建一个FrameAllocator。
//     ///
//     /// 这个函数是不安全的，因为调用者必须保证传递的内存 map 是有效的。
//     /// 主要的要求是，所有在其中被标记为 "可用 "的帧都是真正未使用的。
//     pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
//         BootInfoFrameAllocator {
//             memory_map,
//             next: 0,
//         }
//     }
//     /// 返回内存映射中指定的可用框架的迭代器。
//     fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
//         // 从内存 map 中获取可用的区域
//         let regions = self.memory_map.iter();
//         let usable_regions = regions
//             .filter(|r| r.region_type == MemoryRegionType::Usable);
//         // 将每个区域映射到其地址范围
//         let addr_ranges = usable_regions
//             .map(|r| r.range.start_addr()..r.range.end_addr());
//         // 转化为一个帧起始地址的迭代器
//         let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
//         // 从起始地址创建 `PhysFrame`  类型 
//         frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
//     }
// }

// unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame> {
//         let frame = self.usable_frames().nth(self.next);
//         self.next += 1;
//         frame
//     }
// }


use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}   

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}