use crate::{
    mutex::Mutex,
    x86::io::{io_in_u8, io_out_u8},
};

pub const MASTER_INTERRUPT_OFFSET: u8 = 32;
pub const SLAVE_INTERRUPT_OFFSET: u8 = MASTER_INTERRUPT_OFFSET + 8;
pub const PIC: Mutex<Pics> = Mutex::new(Pics::new(MASTER_INTERRUPT_OFFSET, SLAVE_INTERRUPT_OFFSET));

const MASTER_PIC_COMMAND_IO_ADDRESS: u16 = 0x20;
const MASTER_PIC_DATA_IO_ADDRESS: u16 = MASTER_PIC_COMMAND_IO_ADDRESS + 1;

const SLAVE_PIC_COMMAND_IO_ADDRESS: u16 = 0xA0;
const SLAVE_PIC_DATA_IO_ADDRESS: u16 = SLAVE_PIC_COMMAND_IO_ADDRESS + 1;

const END_OF_INTERRUPT_COMMAND: u8 = 0x20;

const INIT_COMMAND: u8 = 0x11;

const MODE_8086_COMMAND: u8 = 0x1;

const IO_WAIT_WRITE_ADDRESS: u16 = 0x80;

pub struct Pic {
    irq_offset: u8,
    command_address: u16,
    data_address: u16,
}

impl Pic {
    pub const fn new(irq_offset: u8, command_address: u16, data_address: u16) -> Self {
        Self {
            irq_offset,
            command_address,
            data_address,
        }
    }

    pub unsafe fn end_of_interrupt(&self, interrupt_id: u8) {
        if self.irq_offset <= interrupt_id && interrupt_id < (self.irq_offset + 8) {
            io_out_u8(self.command_address, END_OF_INTERRUPT_COMMAND);
        }
    }

    pub unsafe fn read_mask(&self) -> u8 {
        io_in_u8(self.data_address)
    }
    pub unsafe fn write_mask(&self, mask: u8) {
        io_out_u8(self.data_address, mask);
    }
}

pub struct Pics {
    pub master: Pic,
    pub slave: Pic,
}

impl Pics {
    pub const fn new(master_offset: u8, slave_offset: u8) -> Self {
        Self {
            master: Pic::new(
                master_offset,
                MASTER_PIC_COMMAND_IO_ADDRESS,
                MASTER_PIC_DATA_IO_ADDRESS,
            ),
            slave: Pic::new(
                slave_offset,
                SLAVE_PIC_COMMAND_IO_ADDRESS,
                SLAVE_PIC_DATA_IO_ADDRESS,
            ),
        }
    }

    pub unsafe fn init(&mut self) {
        let master_mask = self.master.read_mask();
        let slave_mask = self.slave.read_mask();

        let io_wait = || io_out_u8(IO_WAIT_WRITE_ADDRESS, 0);

        io_out_u8(self.master.command_address, INIT_COMMAND);
        io_wait();

        io_out_u8(self.slave.command_address, INIT_COMMAND);
        io_wait();

        io_out_u8(self.master.data_address, self.master.irq_offset);
        io_wait();

        io_out_u8(self.slave.data_address, self.slave.irq_offset);
        io_wait();

        io_out_u8(self.master.data_address, 4); // slave on IRQ2
        io_wait();

        io_out_u8(self.slave.data_address, 2); // tell slave his identity is 2
        io_wait();

        io_out_u8(self.master.data_address, MODE_8086_COMMAND); // slave on IRQ2
        io_wait();

        io_out_u8(self.slave.data_address, MODE_8086_COMMAND); // tell slave his identity is 2
        io_wait();

        io_out_u8(self.master.data_address, master_mask);
        io_out_u8(self.slave.data_address, slave_mask);
    }

    pub unsafe fn notify_end_of_interrupt(&self, interrupt_id: u8) {
        self.master.end_of_interrupt(interrupt_id);
        self.slave.end_of_interrupt(interrupt_id);
    }
}
