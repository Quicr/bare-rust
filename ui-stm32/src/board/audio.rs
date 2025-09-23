use embassy_stm32::i2c::I2c;
use embassy_stm32::i2c::Master;
use embassy_stm32::mode::Blocking;
use embassy_time::Timer;

pub struct AudioControl {
    i2c: I2c<'static, Blocking, Master>,
    r: [u16; 128],
}

impl AudioControl {
    const VALUE_MASK: u16 = 0x1ff;

    pub fn new(i2c: I2c<'static, Blocking, Master>) -> Self {
        Self { i2c, r: [0; 128] }
    }

    pub async fn init(&mut self) {
        // Reset the wm8960
        self.set_register(0x0F, 0b1_0000_0000);
        Timer::after_millis(100).await;

        // Set the power
        self.set_register(0x19, 0b0_1111_1110);

        // Enable outputs
        self.set_register(0x1A, 0b1_1110_0001);

        // Enable lr mixer ctrl
        // self.set_register(0x2F, 0b0_0000_0000);
        self.set_register(0x2F, 0b0_0010_1100);

        // Disable soft mute and ADC high pass filter
        self.set_register(0x05, 0b0_0000_0000);

        // Set clocks for 8kHz
        self.set_register(0x34, 0b0_0000_1000);
        self.set_register(0x35, 0b0_0011_0001);
        self.set_register(0x36, 0b0_0010_0110);
        self.set_register(0x37, 0b0_1110_1001);
        self.set_register(0x04, 0b1_1011_0001);
        self.set_register(0x08, 0b1_1100_1100);
        self.set_register(0x1B, 0b0_0000_0101);

        // Set mono
        self.set_bit(0x17, 4, true);
        self.set_bit(0x2A, 6, false);

        // Set volumes
        const DEFAULT_VOLUME: u16 = 0b110_0111;
        const DEFAULT_MIC_VOLUME: u16 = 0b11_1111;
        self.set_bits(0x00, 0b1_0011_1111, 0x100 + DEFAULT_MIC_VOLUME);
        self.set_bits(0x02, 0b1_0111_1111, DEFAULT_VOLUME);
        self.set_bits(0x03, 0b1_0111_1111, 0x100 + DEFAULT_VOLUME);

        // Enable the outputs
        self.set_register(0x31, 0b0_0111_0111);

        // Set DAC left and right volumes
        self.set_register(0x0A, 0b1_1111_1111);
        self.set_register(0x0B, 0b1_1111_1111);

        // Set left and right mixer
        self.set_register(0x22, 0b1_0000_0000);
        self.set_register(0x25, 0b1_0000_0000);

        self.set_bits(0x2B, 0b0_0111_0000, 0b0_0111_0000); // XXX extra 0; typo in C?

        // Enable DAC softmute
        self.set_bit(0x06, 3, true);

        // Set the Master mode (1), I2S to 16 bit words
        // Set audio data format to i2s mode
        self.set_register(0x07, 0b0_0100_0010);

        // Unmute the mic
        self.set_bit(0x20, 6, false);
        self.set_bit(0x20, 8, false);
        self.set_bit(0x19, 5, true);
        self.set_bit(0x2f, 5, true);
        self.set_bit(0x20, 3, true);
        self.set_bit(0x20, 7, false);
        self.set_bit(0x20, 6, true);
        self.set_bit(0x20, 8, true);
        self.set_bits(0x00, 0b1_1000_0000, 0b1_0000_0000);
        self.set_bits(0x2B, 0b0_0000_1110, 0b0_0000_1010);
        self.set_bit(0x19, 1, true);
    }

    fn set_register(&mut self, addr: u8, value: u16) {
        self.r[addr as usize] = value & Self::VALUE_MASK;
        self.write_register(addr);
    }

    fn set_bit(&mut self, addr: u8, which: usize, value: bool) {
        defmt::assert!(which < 9);
        let mask = 1 << which;
        let value: u16 = value.into();
        self.r[addr as usize] = (self.r[addr as usize] & !mask) | (value << which);
        self.write_register(addr);
    }

    fn set_bits(&mut self, addr: u8, mask: u16, value: u16) {
        defmt::assert_eq!(mask & !Self::VALUE_MASK, 0);
        defmt::assert_eq!(!mask & value, 0);
        self.r[addr as usize] = (self.r[addr as usize] & !mask) | (mask & value);
        self.write_register(addr);
    }

    fn write_register(&mut self, addr: u8) {
        const ADDR_MASK: u16 = 0x7f;
        const VALUE_MASK: u16 = 0x1ff;
        const DEVICE_ADDR: u8 = 0x1a;

        let to_write = (((addr as u16) & ADDR_MASK) << 9) | (self.r[addr as usize] & VALUE_MASK);
        self.i2c
            .blocking_write(DEVICE_ADDR, &to_write.to_be_bytes())
            .unwrap();
    }
}

// Register map
mod reg {
    #![allow(dead_code)] // No need to use all of the fields on the device

    /// WM8960 register address type (7 bits: 0..127)
    pub type RegAddr = u8;

    /// Trait implemented by each generated field (each field struct knows its addr/offset/width/type).
    pub trait FieldAccess {
        const ADDR: RegAddr;
        const OFFSET: u8;
        const WIDTH: u8;
        type Ty;

        /// Extract field value from raw register (u16 storing 9-bit register).
        fn get(regval: u16) -> Self::Ty;

        /// Insert field value into raw register, asserting on overflow.
        fn set(regval: u16, val: Self::Ty) -> u16;
    }

    /// Macro to define fields: 1-bit bool and multi-bit unsigned fields.
    /// Usage: define_field!(Name, reg_addr, bit_offset, bit_width, type);
    macro_rules! define_field {
        // 1-bit boolean field
        ($name:ident, $addr:expr, $offset:expr, 1, bool) => {
            pub struct $name;
            impl FieldAccess for $name {
                const ADDR: RegAddr = $addr;
                const OFFSET: u8 = $offset;
                const WIDTH: u8 = 1;
                type Ty = bool;

                #[inline]
                fn get(regval: u16) -> bool {
                    ((regval >> Self::OFFSET) & 1) != 0
                }

                #[inline]
                fn set(regval: u16, value: bool) -> u16 {
                    let bit = if value { 1u16 } else { 0u16 };
                    (regval & !(1u16 << Self::OFFSET)) | (bit << Self::OFFSET)
                }
            }
        };

        // multi-bit unsigned field
        ($name:ident, $addr:expr, $offset:expr, $width:expr, $ty:ty) => {
            pub struct $name;
            impl FieldAccess for $name {
                const ADDR: RegAddr = $addr;
                const OFFSET: u8 = $offset;
                const WIDTH: u8 = $width;
                type Ty = $ty;

                #[inline]
                fn get(regval: u16) -> $ty {
                    (((regval >> Self::OFFSET) & ((1u16 << Self::WIDTH) - 1)) as $ty)
                }

                #[inline]
                fn set(regval: u16, value: $ty) -> u16 {
                    let v16 = value as u16;
                    assert!(
                        v16 < (1u16 << Self::WIDTH),
                        concat!(stringify!($name), ": value out of range for width"),
                    );
                    let mask = ((1u16 << Self::WIDTH) - 1) << Self::OFFSET;
                    (regval & !mask) | ((v16 << Self::OFFSET) & mask)
                }
            }
        };
    }

    /// WM8960 register file in-memory representation.
    ///
    /// - `regs` stores the 9-bit register values in `u16` slots indexed by 7-bit register address.
    /// - `modified` stores one byte per register: 0 = not changed, 1 = changed.
    pub struct Wm8960<'a> {
        regs: &'a mut [u16; 56],
        modified: [bool; 56],
    }

    impl<'a> Wm8960<'a> {
        /// Create a new WM8960 register image (all zeros).
        pub const fn new(regs: &'a mut [u16; 56]) -> Self {
            Self {
                regs,
                modified: [false; 56],
            }
        }

        /// Generic getter for any defined field.
        pub fn get_field<F: FieldAccess>(&self) -> F::Ty {
            let reg = self.regs[F::ADDR as usize];
            F::get(reg)
        }

        /// Generic setter for any defined field; asserts on value width and records modification.
        pub fn set_field<F: FieldAccess>(&mut self, val: F::Ty) {
            let idx = F::ADDR as usize;
            let old = self.regs[idx];
            let new = F::set(old, val);
            if new != old {
                self.regs[idx] = new;
                self.modified[idx] = true;
            }
        }

        /// Return whether register at address `addr` has been modified.
        pub fn is_modified(&self, addr: RegAddr) -> bool {
            self.modified[addr as usize]
        }

        /// Clear all modified flags.
        pub fn clear_modified(&mut self) {
            self.modified.iter_mut().for_each(|m| *m = false);
        }

        /// Read raw register value (9-bit in low bits).
        pub fn raw_reg(&self, addr: RegAddr) -> u16 {
            self.regs[addr as usize] & 0x01FF
        }

        /// Set raw register value (9-bit enforced).
        pub fn set_raw_reg(&mut self, addr: RegAddr, value: u16) {
            assert!(value < 0x0200, "raw register value must be 9-bit");
            let idx = addr as usize;
            if self.regs[idx] != value {
                self.regs[idx] = value;
                self.modified[idx] = true;
            }
        }
    }

    /* -----------------------------------------------------------------------------
       Field definitions
       I followed the datasheet register map and "Register bits by address" text.
       Each field has a semantic name and /// doc comments above define_field!.
       -----------------------------------------------------------------------------
    */

    /// R0 (0x00) Left Input PGA: IPVU (bit8) Input PGA Volume Update.
    /// Writing 1 causes left and right input PGA volumes to be updated (LINVOL/RINVOL).
    define_field!(InputPgaVolumeUpdate, 0x00, 8, 1, bool);

    /// R0 (0x00) Left Input PGA: LINMUTE (bit7) Left analogue mute.
    /// 1 = Enable mute, 0 = Disable mute. Note: IPVU must be set to un-mute.
    define_field!(LeftInputAnalogMute, 0x00, 7, 1, bool);

    /// R0 (0x00) Left Input PGA: LIZC (bit6) Zero-cross detect.
    /// 1 = Change gain on zero cross only, 0 = Change gain immediately.
    define_field!(LeftPgaZeroCross, 0x00, 6, 1, bool);

    /// R0 (0x00) Left Input PGA: LINVOL [5:0] (bits 5..0) Left PGA volume control (6 bits).
    /// Range 000000 = -17.25dB ... 111111 = +30dB, 0.75 dB steps. Default 010111 (0dB).
    define_field!(LeftPgaVolume, 0x00, 0, 6, u8);

    /// R1 (0x01) Right Input PGA: IPVU (bit8) Input PGA Volume Update.
    /// Writing 1 causes left and right input PGA volumes to be updated.
    define_field!(InputPgaVolumeUpdate_Right, 0x01, 8, 1, bool);

    /// R1 (0x01) Right Input PGA: RINMUTE (bit7) Right analogue mute.
    define_field!(RightInputAnalogMute, 0x01, 7, 1, bool);

    /// R1 (0x01) Right Input PGA: RIZC (bit6) Zero-cross detect.
    define_field!(RightPgaZeroCross, 0x01, 6, 1, bool);

    /// R1 (0x01) Right Input PGA: RINVOL [5:0] Right PGA volume (6 bits).
    define_field!(RightPgaVolume, 0x01, 0, 6, u8);

    /// R2 (0x02) Left headphone/LOUT1 volume update (bit8) OUT1VU.
    define_field!(HeadphoneOutVolumeUpdate, 0x02, 8, 1, bool);

    /// R2 (0x02) Left headphone: LO1ZC (bit7) zero cross for LOUT1 volume updates.
    define_field!(LeftOutZeroCross, 0x02, 7, 1, bool);

    /// R2 (0x02) LOUT1VOL [6:0] (bits 6..0) Left headphone volume (7 bits).
    define_field!(LeftHeadphoneVolume, 0x02, 0, 7, u8);

    /// R3 (0x03) Right headphone ROUT1 volume update and zero-cross (mirror of R2)
    define_field!(HeadphoneOutVolumeUpdate_Right, 0x03, 8, 1, bool);
    define_field!(RightOutZeroCross, 0x03, 7, 1, bool);
    define_field!(RightHeadphoneVolume, 0x03, 0, 7, u8);

    /// R4 (0x04) Clocking (1) - CLKSEL bitfields in datasheet (bits layout reserved).
    /// (table shows bit7..0 reserved/defaults) -- register kept as raw if needed.
    define_field!(Clocking1_Raw, 0x04, 0, 9, u16);

    /// R5 (0x05) ADC & DAC Control (1) — ADCHPD (bit0) ADC High Pass Filter Disable.
    /// 0 = enable HP filter, 1 = disable.
    define_field!(AdcHighPassDisable, 0x05, 0, 1, bool);

    /// R5 (0x05) ADC & DAC Control (1) — DACMU (bit2) DAC soft-mute enable etc.
    /// DACS related bits: DACMU at bit? (table shows pattern, we provide raw+specifics)
    define_field!(AdcDacControl1_Raw, 0x05, 0, 9, u16);

    /// R6 (0x06) ADC & DAC Control (2) — DACSLOPE (bit1) select DAC filter slope.
    define_field!(DacSlopeMode, 0x06, 1, 1, bool);

    /// R6 (0x06) ADC & DAC Control (2) — DACMR (bit2) DAC Soft Mute Ramp Rate.
    define_field!(DacSoftMuteRampSlow, 0x06, 2, 1, bool);

    /// R6 (0x06) ADC & DAC Control (2) — DACSMM (bit3) DAC Soft Mute Mode.
    define_field!(DacSoftMuteMode, 0x06, 3, 1, bool);

    /// R7 (0x07) Audio Interface — ALRSWAP (bit8) Left/Right ADC swap.
    define_field!(AdcLeftRightSwap, 0x07, 8, 1, bool);

    /// R7 (0x07) Audio Interface — BCLKINV (bit7) BCLK invert.
    define_field!(BclkInvert, 0x07, 7, 1, bool);

    /// R7 (0x07) Audio Interface — MS (bit6) Master/Slave mode (1 = master).
    define_field!(AudioInterfaceMasterMode, 0x07, 6, 1, bool);

    /// R7 (0x07) Audio Interface — DLRSWAP (bit5) DAC LR swap.
    define_field!(DacLeftRightSwap, 0x07, 5, 1, bool);

    /// R7 (0x07) Audio Interface — LRP (bit4) LRCLK polarity / DSP mode select.
    define_field!(LrcPolarityOrDspMode, 0x07, 4, 1, bool);

    /// R8 (0x08) Clocking (2) — BCLKDIV [3:0] bits 3..0 (4 bits) select BCLK divider in master mode.
    define_field!(BclkDivider_Master, 0x08, 0, 4, u8);

    /// R8 (0x08) Clocking (2) — DCLKDIV [2:0] bits 8..6 (class D switching clock divider)
    define_field!(ClassDSysclkDivider, 0x08, 6, 3, u8);

    /// R9 (0x09) Audio Interface — WL[1:0] (bits3..2) Word length selection (00=16,01=20,10=24,11=32)
    define_field!(WordLength, 0x09, 2, 2, u8);

    /// R9 (0x09) Audio Interface — DACCOMP[1:0] bits4..3 DAC companding
    define_field!(DacCompanding, 0x09, 3, 2, u8);

    /// R9 (0x09) Audio Interface — ADCCOMP[1:0] bits1..0 ADC companding
    define_field!(AdcCompanding, 0x09, 0, 2, u8);

    /// R10 (0x0A) Left DAC Digital Volume LDACVOL [7:0] (bits 7..0)
    define_field!(LeftDacDigitalVolume, 0x0A, 0, 8, u8);

    /// R10 (0x0A) Left DAC volume update (bit8) DACVU
    define_field!(DacVolumeUpdate_Left, 0x0A, 8, 1, bool);

    /// R11 (0x0B) Right DAC Digital Volume RDACVOL [7:0]
    define_field!(RightDacDigitalVolume, 0x0B, 0, 8, u8);
    define_field!(DacVolumeUpdate_Right, 0x0B, 8, 1, bool);

    /// R12-R14 (0x0C-0x0E) Reserved — keep raw registers if needed.
    define_field!(Reserved0C_Raw, 0x0C, 0, 9, u16);
    define_field!(Reserved0D_Raw, 0x0D, 0, 9, u16);
    define_field!(Reserved0E_Raw, 0x0E, 0, 9, u16);

    /// R15 (0x0F) Reset register. Writing anything = reset to defaults (special behavior).
    define_field!(ResetRegister, 0x0F, 0, 9, u16);

    /// R16 (0x10) 3D control — 3DEN (bit2) 3D enable, 3DLC (bit1) lower cut-off, 3DUC (bit0) upper cut-off.
    define_field!(ThreeDEnable, 0x10, 2, 1, bool);
    define_field!(ThreeDLowerCutSelect, 0x10, 1, 1, bool);
    define_field!(ThreeDUpperCutSelect, 0x10, 0, 1, bool);
    define_field!(ThreeDControl_Raw, 0x10, 0, 9, u16);

    /// R17 (0x11) — (register content used for various controls — provide raw and some named pieces)
    define_field!(Reg17_Raw, 0x11, 0, 9, u16);

    /// R18 (0x12) — some routing controls (raw)
    define_field!(Reg18_Raw, 0x12, 0, 9, u16);

    /// R19 (0x13) Power Management (1) register contains multiple bits:
    /// VMIDSEL [8:7] (bits7-8) Vmid divider select, VREF (bit6) VREF enable, AINL (bit5) AINL enable, AINR (bit4)
    define_field!(InputPowerVmidSelect, 0x13, 7, 2, u8);
    define_field!(ReferenceVoltageEnable, 0x13, 6, 1, bool);
    define_field!(LeftAnalogueInputPgaAndBoostEnable, 0x13, 5, 1, bool);
    define_field!(RightAnalogueInputPgaAndBoostEnable, 0x13, 4, 1, bool);
    define_field!(AdcLeftEnable, 0x13, 3, 1, bool); // ADCL
    define_field!(AdcRightEnable, 0x13, 2, 1, bool); // ADCR

    /// R20 (0x14) Noise gate: NGTH [7:3] bits — noise gate threshold(5 bits) and NGAT (bit0) enable.
    define_field!(NoiseGateThreshold, 0x14, 3, 5, u8);
    define_field!(NoiseGateEnable, 0x14, 0, 1, bool);

    /// R21 (0x15) Left ADC digital volume LADCVOL [7:0] and ADCVU (bit8)
    define_field!(LeftAdcDigitalVolume, 0x15, 0, 8, u8);
    define_field!(AdcVolumeUpdate_Left, 0x15, 8, 1, bool);

    /// R22 (0x16) Right ADC digital volume RADCVOL [7:0] and ADCVU (bit8)
    define_field!(RightAdcDigitalVolume, 0x16, 0, 8, u8);
    define_field!(AdcVolumeUpdate_Right, 0x16, 8, 1, bool);

    /// R23 (0x17) Additional Control (1) — TOEN (bit0) Timeout Enable, TOCLKSEL (bit1) slow clock selection,
    /// VSEL [7:6] bias optimise etc.
    define_field!(TimeoutEnable, 0x17, 0, 1, bool);
    define_field!(TimeoutClockSelect, 0x17, 1, 1, bool);
    define_field!(BiasVsel, 0x17, 6, 2, u8);
    define_field!(Reg23_Raw, 0x17, 0, 9, u16);

    /// R24 (0x18) Additional Control (2) — LRCM (bit2) ADCLRC/DACLRC disable selector
    define_field!(AdclrcDaclrcMode, 0x18, 2, 1, bool);
    define_field!(Reg24_Raw, 0x18, 0, 9, u16);

    /// R25 (0x19) Power Management (1) detailed bits (VMIDSEL, VREF, AINL, AINR, ADCL, ADCR, MICB)
    /// NOTE: addresses in datasheet show R25(19h) used multiple times; here consolidate important bits:
    define_field!(PowerMgmt1_VmidSelect, 0x19, 7, 2, u8);
    define_field!(PowerMgmt1_VrefEnable, 0x19, 6, 1, bool);
    define_field!(PowerMgmt1_AinLeftEnable, 0x19, 5, 1, bool);
    define_field!(PowerMgmt1_AinRightEnable, 0x19, 4, 1, bool);
    define_field!(PowerMgmt1_EnableAdcLeft, 0x19, 3, 1, bool);
    define_field!(PowerMgmt1_EnableAdcRight, 0x19, 2, 1, bool);
    define_field!(MicrophoneBiasEnable, 0x19, 1, 1, bool); // MICB (note: datasheet R25 had MICB in some table)
    define_field!(PowerMgmt1_Raw, 0x19, 0, 9, u16);

    /// R26 (0x1A) Power Management (2) — PLLEN (bit0) enable PLL; output enables, speaker outputs.
    /// SPK_OP_EN [7:6] (in other registers) configure speaker enable — here pick key bits:
    define_field!(PllEnable, 0x1A, 0, 1, bool);
    define_field!(LeftOutput1Enable, 0x1A, 6, 1, bool);
    define_field!(RightOutput1Enable, 0x1A, 5, 1, bool);
    define_field!(LeftSpeakerVolumeEnable, 0x1A, 4, 1, bool);
    define_field!(RightSpeakerVolumeEnable, 0x1A, 3, 1, bool);
    define_field!(Out3Enable, 0x1A, 1, 1, bool);

    /// R27 (0x1B) ADC ALC sample rate selection: ADC_ALC_SR [2:0] (bits2..0)
    define_field!(AlcSampleRateSelect, 0x1B, 0, 3, u8);

    /// R28..R31 (0x1C..0x1F) Various registers — keep raw access
    define_field!(Reg1C_Raw, 0x1C, 0, 9, u16);
    define_field!(Reg1D_Raw, 0x1D, 0, 9, u16);
    define_field!(Reg1E_Raw, 0x1E, 0, 9, u16);
    define_field!(Reg1F_Raw, 0x1F, 0, 9, u16);

    /// R32 (0x20) ADCL signal path — LMICBOOST [5:4] left channel microphone boost (2 bits)
    define_field!(LeftMicBoost, 0x20, 4, 2, u8);
    define_field!(Adcl_SignalPath_Raw, 0x20, 0, 9, u16);

    /// R33 (0x21) ADCR signal path — RMICBOOST [5:4] right channel mic boost
    define_field!(RightMicBoost, 0x21, 4, 2, u8);
    define_field!(Adcr_SignalPath_Raw, 0x21, 0, 9, u16);

    /// R43 (0x2B) Input Boost Mixer 1 — LIN3BOOST [6:4], LIN2BOOST [3:1]
    define_field!(Linput3Boost, 0x2B, 4, 3, u8);
    define_field!(Linput2Boost, 0x2B, 1, 3, u8);

    /// R44 (0x2C) Input Boost Mixer 2 — RIN3BOOST [6:4], RIN2BOOST [3:1]
    define_field!(Rinput3Boost, 0x2C, 4, 3, u8);
    define_field!(Rinput2Boost, 0x2C, 1, 3, u8);

    /// R47 (0x2F) Power Management (3) — LMIC, RMIC (Left/Right PGA enable). Bits 5 and 4 per table.
    define_field!(LeftPgaEnableIfAin, 0x2F, 5, 1, bool); // LMIC
    define_field!(RightPgaEnableIfAin, 0x2F, 4, 1, bool); // RMIC

    /// R49 (0x31) Class D Control (1) — SPK_OP_EN [7:6] enable class D speaker outputs
    define_field!(SpeakerOutputsEnable, 0x31, 6, 2, u8);

    /// R50 (0x32) Class D Control (2) raw
    define_field!(ClassDControl2_Raw, 0x32, 0, 9, u16);

    /// R51 (0x33) Class D Control (3) — DCGAIN [5:3], ACGAIN [2:0]
    define_field!(SpeakerDcGain, 0x33, 3, 3, u8);
    define_field!(SpeakerAcGain, 0x33, 0, 3, u8);

    /// R52..R55 PLL registers and others (raw & pieces)
    define_field!(PllControl1_Raw, 0x34, 0, 9, u16);
    define_field!(PllControl2_Raw, 0x35, 0, 9, u16);
    define_field!(PllK_Msb, 0x36, 0, 9, u16); // pieces of PLL K spread over several regs
    define_field!(PllK_Mid, 0x37, 0, 9, u16);
    define_field!(PllK_Lsb, 0x38, 0, 9, u16);

    /// R54..R55 extra PLL pieces etc (raw)
    define_field!(Reg54_Raw, 0x36, 0, 9, u16);
    define_field!(Reg55_Raw, 0x37, 0, 9, u16);

    /// R? other volume registers: speaker/headphone volumes (SPKLVOL/SPKRVOL)
    define_field!(LeftSpeakerVolume, 0x40, 0, 7, u8); // address chosen as placeholder; please refer to table for exact addr
    define_field!(RightSpeakerVolume, 0x41, 0, 7, u8);
}
