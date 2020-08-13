use super::Device;

pub struct Screen {}

impl Screen {
    fn move_to(&self, x: usize, y: usize) {
        print!("\x1b[{};{}H", y, x)
    }

    fn clear_screen(&self) {
        print!("\x1b[24")
    }
}

impl Device for Screen {
    fn get_u16(&self, _: usize) -> u16 {
        panic!("Attempted reading from a screen")
    }

    fn get_u8(&self, _: usize) -> u8 {
        panic!("Attempted reading from a screen")
    }

    fn set_u16(&mut self, address: usize, value: u16) {
        let command = (value & 0xff00) >> 8;
        if command == 0xff {
            self.clear_screen();
        }
        let char_value = value & 0x00ff;
        let x = address % 16 + 1;
        let y = address / 16 + 1;
        self.move_to(x, y);
        print!("{}", (char_value as u8) as char)
    }

    fn set_u8(&mut self, _: usize, _: u8) {
        unimplemented!()
    }

    fn len(&self) -> usize {
        unimplemented!()
    }

    fn set_mb(&mut self, _: u16) {}
}
