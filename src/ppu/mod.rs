const OAM_SEARCH_DURATION: u16 = 20 * 4;
const PIXEL_TRANSFER_DURATION: u16 = 43 * 4;
const HBLANK_DURATION: u16 = 51 * 4;
const LINE_TOTAL_DURATION: u16 = OAM_SEARCH_DURATION + PIXEL_TRANSFER_DURATION + HBLANK_DURATION;
const DRAWN_LINES: u8 = 144;
const VBLANK_LINES: u8 = 10;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum PPU_Mode { OAM, PixelTransfer, HBlank, VBlank }

fn mode_duration(mode: &PPU_Mode) -> u16 {
    match mode {
        PPU_Mode::OAM => OAM_SEARCH_DURATION,
        PPU_Mode::PixelTransfer => PIXEL_TRANSFER_DURATION,
        PPU_Mode::HBlank => HBLANK_DURATION,
        PPU_Mode::VBlank => VBLANK_LINES as u16 * LINE_TOTAL_DURATION,
    }
}
fn next_mode(mode: &PPU_Mode, current_line: &u8) -> PPU_Mode {
    match mode {
        PPU_Mode::OAM => PPU_Mode::PixelTransfer,
        PPU_Mode::PixelTransfer => PPU_Mode::HBlank,
        PPU_Mode::HBlank => { if *current_line <= 144 { PPU_Mode::OAM } else { PPU_Mode::VBlank} },
        PPU_Mode::VBlank => PPU_Mode::OAM,
    }
}

pub struct PPU {
    pub cycle_count: u64,
    current_line: u8,
    current_mode: PPU_Mode,
    cycles_in_current_mode: u16,
    cycles_in_current_line: u16,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            cycle_count: 0,
            current_line: 0,
            current_mode: PPU_Mode::OAM, // FIXME CONFIRM
            cycles_in_current_mode: 0,
            cycles_in_current_line: 0,
        }
    }

    pub fn cycle(&mut self) {
        self.cycle_count += 1;
        self.cycles_in_current_mode += 1;
        self.cycles_in_current_line += 1;

        let duration = mode_duration(&self.current_mode);

        if duration > 0 && self.cycles_in_current_mode >= duration {
            self.current_mode = next_mode(&self.current_mode, &self.current_line);
            self.cycles_in_current_mode = 0;
        }

        if self.cycles_in_current_line == LINE_TOTAL_DURATION {
            self.cycles_in_current_line = 0;
            self.current_line += 1;
            if self.current_line == DRAWN_LINES + VBLANK_LINES {
                self.current_line = 0;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle() {
        let mut ppu = PPU::new();
        ppu.cycle();
        assert_eq!(ppu.cycle_count, 1);
    }

    #[test]
    fn mode_timings() {
        let mut ppu = PPU::new();
        for i in 0..(20 * 4) {
            assert_eq!(ppu.cycles_in_current_mode, i);
            assert_eq!(ppu.current_mode, PPU_Mode::OAM);
            ppu.cycle();
        }
        for i in 0..(43 * 4) {
            assert_eq!(ppu.cycles_in_current_mode, i);
            assert_eq!(ppu.current_mode, PPU_Mode::PixelTransfer);
            ppu.cycle();
        }
        for i in 0..(51 * 4) {
            assert_eq!(ppu.cycles_in_current_mode, i);
            assert_eq!(ppu.current_mode, PPU_Mode::HBlank);
            ppu.cycle();
        }
    }
}
