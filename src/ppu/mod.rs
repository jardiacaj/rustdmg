const OAM_SEARCH_DURATION: u16 = 20 * 4;
const PIXEL_TRANSFER_DURATION: u16 = 43 * 4;
const HBLANK_DURATION: u16 = 51 * 4;
const LINE_TOTAL_DURATION: u16 = OAM_SEARCH_DURATION + PIXEL_TRANSFER_DURATION + HBLANK_DURATION;
const DRAWN_LINES: u8 = 144;
const VBLANK_LINES: u8 = 10;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum PpuMode { OAM, PixelTransfer, HBlank, VBlank }

fn mode_duration(mode: &PpuMode) -> u16 {
    match mode {
        PpuMode::OAM => OAM_SEARCH_DURATION,
        PpuMode::PixelTransfer => PIXEL_TRANSFER_DURATION,
        PpuMode::HBlank => HBLANK_DURATION,
        PpuMode::VBlank => VBLANK_LINES as u16 * LINE_TOTAL_DURATION,
    }
}
fn next_mode(mode: &PpuMode, current_line: u8) -> PpuMode {
    match mode {
        PpuMode::OAM => PpuMode::PixelTransfer,
        PpuMode::PixelTransfer => PpuMode::HBlank,
        PpuMode::HBlank => { if current_line < DRAWN_LINES { PpuMode::OAM } else { PpuMode::VBlank} },
        PpuMode::VBlank => PpuMode::OAM,
    }
}

pub struct PPU {
    pub cycle_count: u64,
    pub current_line: u8,
    current_mode: PpuMode,
    cycles_in_current_mode: u16,
    cycles_in_current_line: u16,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            cycle_count: 0,
            current_line: 0,
            current_mode: PpuMode::OAM, // FIXME CONFIRM
            cycles_in_current_mode: 0,
            cycles_in_current_line: 0,
        }
    }

    pub fn cycle(&mut self) {
        self.cycle_count += 1;
        self.cycles_in_current_mode += 1;
        self.cycles_in_current_line += 1;

        let duration = mode_duration(&self.current_mode);

        if self.cycles_in_current_line == LINE_TOTAL_DURATION {
            self.cycles_in_current_line = 0;
            self.current_line += 1;
            if self.current_line >= DRAWN_LINES + VBLANK_LINES {
                self.current_line = 0;
            }
        }

        if duration > 0 && self.cycles_in_current_mode >= duration {
            self.current_mode = next_mode(&self.current_mode, self.current_line);
            self.cycles_in_current_mode = 0;
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

        for _frame in 0..2 {
            for line in 0..144 {
                assert_eq!(ppu.current_line, line);
                for i in 0..(20 * 4) {
                    assert_eq!(ppu.cycles_in_current_mode, i);
                    assert_eq!(ppu.current_mode, PpuMode::OAM);
                    ppu.cycle();
                }
                for i in 0..(43 * 4) {
                    assert_eq!(ppu.cycles_in_current_mode, i);
                    assert_eq!(ppu.current_mode, PpuMode::PixelTransfer);
                    ppu.cycle();
                }
                for i in 0..(51 * 4) {
                    assert_eq!(ppu.cycles_in_current_mode, i);
                    assert_eq!(ppu.current_mode, PpuMode::HBlank);
                    ppu.cycle();
                }
            }
            for line_in_vblank in 0..10 as u8 {
                assert_eq!(ppu.current_line, line_in_vblank + 144);
                for cycles_per_vblank in 0..((20 + 43 + 51) * 4) {
                    println!("{} {}", cycles_per_vblank, ppu.current_line);
                    assert_eq!(ppu.cycles_in_current_mode, cycles_per_vblank + line_in_vblank as u16 * LINE_TOTAL_DURATION);
                    assert_eq!(ppu.current_mode, PpuMode::VBlank);
                    ppu.cycle();
                }
            }
        }
    }
}
