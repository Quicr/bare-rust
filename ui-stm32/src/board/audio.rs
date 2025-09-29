#![allow(dead_code)] // No need to use all of the fields on the device

use embassy_stm32::i2c::I2c;
use embassy_stm32::i2c::Master;
use embassy_stm32::mode::Blocking;
use embassy_time::Timer;

type I2C = I2c<'static, Blocking, Master>;
const I2C_ADDR: u8 = 0x1a;

pub struct AudioControl {
    i2c: I2c<'static, Blocking, Master>,
    regs: Registers,
}

impl AudioControl {
    const VALUE_MASK: u16 = 0x1ff;

    pub fn new(i2c: I2c<'static, Blocking, Master>) -> Self {
        Self {
            i2c,
            regs: Registers::default(),
        }
    }

    async fn reset(&mut self) {
        // address = 0x0f, value = 0b0_0000_0000
        const RESET_SIGNAL: [u8; 2] = [0x1e, 0x00];
        self.i2c.blocking_write(I2C_ADDR, &RESET_SIGNAL).unwrap();
        Timer::after_millis(100).await;
    }

    pub async fn init(&mut self) {
        self.reset().await;

        // Route input to output
        self.regs.modify(&mut self.i2c, |r| {
            // 0. Enable Vref and master clock
            r.set::<PowerMgmt1VrefEnable>(true);
            r.set::<MasterClockDisable>(false); // ???
            r.set::<MicrophoneBiasEnable>(true); // ???

            // 1. Configure the input path
            // 1.1. Power on input devices
            r.set::<PowerMgmt1VmidSelect>(0b01);
            r.set::<PowerMgmt1AinLeftEnable>(true);
            r.set::<LeftMicEnable>(true);

            // 2.2. Disable unused inputs on the left side
            r.set::<Linput2Boost>(0b000);
            r.set::<Linput3Boost>(0b000);
            r.set::<LeftInput3ToOutputMixer>(false);
            r.set::<LeftInput3ToOutputMixerVolume>(0b000);

            // 2.3. Disable the right side inputs
            r.set::<RightInputAnalogMute>(true);
            r.set::<Rinput2Boost>(0b000);
            r.set::<Rinput3Boost>(0b000);
            r.set::<RightInput3ToOutputMixer>(false);
            r.set::<RightInput3ToOutputMixerVolume>(0b000);

            // 2.4. Enable the left side
            r.set::<LeftInput1ToInverting>(true);
            r.set::<LeftInput3ToNonInverting>(false);
            r.set::<LeftInput2ToNonInverting>(true);
            r.set::<LeftInputToBoost>(true);
            r.set::<InputPgaVolumeUpdateRight>(true);
            r.set::<LeftPgaVolume>(0b01_0111); // 0dB
            r.set::<LeftBoostGain>(0b10); // 20dB
            r.set::<LeftInputAnalogMute>(false);

            // 3. Configure the output path
            // 3.1. Power out output devices
            r.set::<LeftOutput1Enable>(true);
            r.set::<LeftOutputMixEnable>(true);

            // 3.2. Disable unused paths on the left side
            r.set::<LeftDacToOutputMixer>(false);
            r.set::<LeftInput3ToOutputMixer>(false);
            r.set::<LeftInput3ToOutputMixerVolume>(0b000);
            r.set::<LeftSpeakerVolumeUpdate>(true);
            r.set::<LeftSpeakerVolume>(0b000_0000);

            // 3.3. Disable the right side (NOOP)
            // 3.4. Enable the good path on the left side
            r.set::<LeftBoostToLeftOutputMix>(true);
            r.set::<LeftBoostToLeftOutputMixVolume>(0b000); // 0dB
            r.set::<HeadphoneOutVolumeUpdate>(true);
            r.set::<LeftHeadphoneVolume>(0b111_1111); // 6dB
        });

        /*
        // Startup classic
        self.regs.modify(&mut self.i2c, |r| {
            // Turn everything on
            r.set::<PowerMgmt1VmidSelect>(0b01);
            r.set::<PowerMgmt1VrefEnable>(true);
            r.set::<PowerMgmt1AinLeftEnable>(true);
            r.set::<PowerMgmt1AinRightEnable>(true);
            r.set::<PowerMgmt1EnableAdcLeft>(true);
            r.set::<PowerMgmt1EnableAdcRight>(true);
            r.set::<MicrophoneBiasEnable>(true);
            r.set::<MasterClockDisable>(false);

            r.set::<LeftDacEnable>(true);
            r.set::<RightDacEnable>(true);
            r.set::<LeftOutput1Enable>(true);
            r.set::<RightOutput1Enable>(true);
            r.set::<PllEnable>(true);

            r.set::<LeftMicEnable>(true);
            r.set::<LeftOutputMixEnable>(true);
            r.set::<RightOutputMixEnable>(true);

            // Disable soft mut and ADC high pass filter
            r.set::<DacSoftMuteEnable>(false);
            r.set::<AdcHighPassDisable>(false);

            // Set clocks for 8khz
            r.set::<PllN>(0b1000);
            r.set::<PllKMsb>(0b0011_0001);
            r.set::<PllKMid>(0b0010_0110);
            r.set::<PllKLsb>(0b1110_1001);
            r.set::<Adc1Divider>(0b110);
            r.set::<DacDivider>(0b110);
            r.set::<SysClkDiv>(0b00);
            r.set::<ClockSelect>(true);
            r.set::<BclkFrequency>(0b1100);
            r.set::<ClassDSysclkDivider>(0b111);
            r.set::<AdcAlcSampleRateSelect>(0b101);

            // Set mono
            r.set::<DacMonoMix>(true);
            r.set::<MonoOutVolume>(false);

            // Set volumes
            r.set::<InputPgaVolumeUpdate>(true);
            r.set::<LeftPgaVolume>(0b11_1111);
            r.set::<HeadphoneOutVolumeUpdate>(true);
            r.set::<LeftHeadphoneVolume>(0b110_0111);
            r.set::<RightHeadphoneVolume>(0b111_1111);

            // Enable the outputs
            r.set::<ClassDSpeakerOutputEnable>(0b01);

            // Set the DAC left and right volumes
            r.set::<DacVolumeUpdate>(true);
            r.set::<LeftDacDigitalVolume>(0b1111_1111);
            r.set::<RightDacDigitalVolume>(0b1111_1111);

            // Set left and right mixer
            r.set::<LeftDacToOutputMixer>(true);
            r.set::<LeftInput3ToOutputMixer>(false);
            r.set::<LeftInput3ToOutputMixerVolume>(0b000);
            r.set::<RightDacToOutputMixer>(true);
            r.set::<RightInput3ToOutputMixerVolume>(0b000);
            r.set::<Linput3Boost>(0b111);

            // Enable DAC softmute
            r.set::<DacSoftMuteMode>(true);

            // Set master mode, I2S, 16-bit words
            r.set::<AudioInterfaceMasterMode>(true);
            r.set::<AudioWordLength>(0b00);
            r.set::<AudioFormat>(0b10);

            // Unmute the mic
            // XXX(RLB) These are done in the C version, but they get overwritten by the
            // later writes in this version.  Does the order matter?
            //r.set::<LeftInput1ToInverting>(false);
            //r.set::<LeftInput3ToNonInverting>(false);
            r.set::<PowerMgmt1EnableAdcLeft>(true);
            r.set::<LeftMicEnable>(true);
            r.set::<LeftInputToBoost>(true);
            r.set::<LeftInput2ToNonInverting>(true);
            r.set::<LeftInput3ToNonInverting>(true);
            r.set::<LeftInput1ToInverting>(true);
            r.set::<InputPgaVolumeUpdate>(true);
            r.set::<Linput2Boost>(0b101);
            r.set::<MicrophoneBiasEnable>(true);
        });
        */
    }
}

pub type RegAddr = u8;

trait ToFromU16 {
    fn from_u16(x: u16) -> Self;
    fn into_u16(self) -> u16;
}

impl ToFromU16 for bool {
    fn from_u16(x: u16) -> Self {
        x != 0
    }

    fn into_u16(self) -> u16 {
        if self {
            1
        } else {
            0
        }
    }
}

impl ToFromU16 for u8 {
    fn from_u16(x: u16) -> Self {
        x as Self
    }

    fn into_u16(self) -> u16 {
        self.into()
    }
}

impl ToFromU16 for u16 {
    fn from_u16(x: u16) -> Self {
        x
    }

    fn into_u16(self) -> u16 {
        self
    }
}

pub trait FieldAccess {
    const ADDR: RegAddr;
    const OFFSET: u8;
    const WIDTH: u8;
    const MAX: u16 = (1 << Self::WIDTH) - 1;
    const MASK: u16 = Self::MAX << Self::OFFSET;
    type Value;

    fn get(regval: u16) -> Self::Value;
    fn set(regval: u16, val: Self::Value) -> u16;
}

macro_rules! define_field {
    ($name:ident, $addr:expr, $offset:expr, $width:expr, $val:ty) => {
        pub struct $name;
        impl FieldAccess for $name {
            const ADDR: RegAddr = $addr;
            const OFFSET: u8 = $offset;
            const WIDTH: u8 = $width;
            type Value = $val;

            #[inline]
            fn get(regval: u16) -> $val {
                <$val>::from_u16((regval & Self::MASK) >> Self::OFFSET)
            }

            #[inline]
            fn set(regval: u16, value: $val) -> u16 {
                let v16 = value.into_u16();
                assert!(
                    v16 <= Self::MAX,
                    concat!(stringify!($name), ": value out of range"),
                );
                let mask = ((1u16 << Self::WIDTH) - 1) << Self::OFFSET;
                (regval & !mask) | ((v16 << Self::OFFSET) & mask)
            }
        }
    };
}

#[derive(Clone)]
pub struct Registers {
    regs: [u16; 56],
}

impl Default for Registers {
    fn default() -> Self {
        let init: [(u8, u16); 56] = [
            (0x00, 0b0_1001_0111), // R0  Left Input volume
            (0x01, 0b0_1001_0111), // R1  Right Input volume
            (0x02, 0b0_0000_0000), // R2  LOUT1 volume
            (0x03, 0b0_0000_0000), // R3  ROUT1 volume
            (0x04, 0b0_0000_0000), // R4  Clocking (1)
            (0x05, 0b0_0000_1000), // R5  ADC & DAC Control (1)
            (0x06, 0b0_0000_0000), // R6  ADC & DAC Control (2)
            (0x07, 0b0_0000_1010), // R7  Audio Interface
            (0x08, 0b1_1100_0000), // R8  Clocking (2)
            (0x09, 0b0_0000_0000), // R9  Audio Interface
            (0x0A, 0b0_1111_1111), // R10 Left DAC volume
            (0x0B, 0b0_1111_1111), // R11 Right DAC volume
            (0x0C, 0b0_0000_0000), // R12 Reserved
            (0x0D, 0b0_0000_0000), // R13 Reserved
            (0x0E, 0b0_0000_0000), // R14 Reserved
            (0x0F, 0b0_0000_0000), // R15 Reset (not reset)
            (0x10, 0b0_0000_0000), // R16 3D control
            (0x11, 0b0_0111_1011), // R17 ALC1
            (0x12, 0b1_0000_0000), // R18 ALC2
            (0x13, 0b0_0011_0010), // R19 ALC3
            (0x14, 0b0_0000_0000), // R20 Noise Gate
            (0x15, 0b0_1100_0011), // R21 Left ADC volume
            (0x16, 0b0_1100_0011), // R22 Right ADC volume
            (0x17, 0b1_1100_0000), // R23 Additional control (1)
            (0x18, 0b0_0000_0000), // R24 Additional control (2)
            (0x19, 0b0_0000_0000), // R25 Power Mgmt (1)
            (0x1A, 0b0_0000_0000), // R26 Power Mgmt (2)
            (0x1B, 0b0_0000_0000), // R27 Additional Control (3)
            (0x1C, 0b0_0000_0000), // R28 Anti-pop 1
            (0x1D, 0b0_0000_0000), // R29 Anti-pop 2
            (0x1E, 0b0_0000_0000), // R30 Reserved
            (0x1F, 0b0_0000_0000), // R31 Reserved
            (0x20, 0b1_0000_0000), // R32 ADCL signal path
            (0x21, 0b1_0000_0000), // R33 ADCR signal path
            (0x22, 0b0_0101_0000), // R34 Left out Mix (1)
            (0x23, 0b0_0101_0000), // R35 Reserved
            (0x24, 0b0_0101_0000), // R36 Reserved
            (0x25, 0b0_0101_0000), // R37 Right out Mix (2)
            (0x26, 0b0_0000_0000), // R38 Mono out Mix (1)
            (0x27, 0b0_0000_0000), // R39 Mono out Mix (2)
            (0x28, 0b0_0000_0000), // R40 LOUT2 volume
            (0x29, 0b0_0000_0000), // R41 ROUT2 volume
            (0x2A, 0b0_0100_0000), // R42 MONOOUT volume
            (0x2B, 0b0_0000_0000), // R43 Input boost mixer (1)
            (0x2C, 0b0_0000_0000), // R44 Input boost mixer (2)
            (0x2D, 0b0_0101_0000), // R45 Bypass (1)
            (0x2E, 0b0_0101_0000), // R46 Bypass (2)
            (0x2F, 0b0_0000_0000), // R47 Power Mgmt (3)
            (0x30, 0b0_0000_0010), // R48 Additional Control (4)
            (0x31, 0b0_0011_0111), // R49 Class D Control (1)
            (0x32, 0b0_0100_1101), // R50 Reserved
            (0x33, 0b0_1000_0000), // R51 Class D Control (3)
            (0x34, 0b0_0000_1000), // R52 PLL N
            (0x35, 0b0_0011_0001), // R53 PLL K1
            (0x36, 0b0_0010_0110), // R54 PLL K2
            (0x37, 0b0_1110_1001), // R55 PLL K3
        ];

        let mut regs = [0u16; 56];
        for (i, (addr, val)) in init.iter().enumerate() {
            regs[i] = ((*addr as u16) << 9) | (val & 0x01FF);
        }

        Self { regs }
    }
}

impl Registers {
    fn modify<F>(&mut self, i2c: &mut I2C, f: F)
    where
        F: FnOnce(&mut RegisterView),
    {
        let mut r = RegisterView::new(&mut self.regs);
        f(&mut r);

        let modified = r
            .modified
            .iter()
            .enumerate()
            .filter_map(|(i, m)| m.then_some(i));
        for i in modified {
            let addr = self.regs[i] >> 9;
            i2c.blocking_write(I2C_ADDR, &self.regs[i].to_be_bytes())
                .unwrap();
        }
    }
}

pub struct RegisterView<'a> {
    regs: &'a mut [u16; 56],
    modified: [bool; 56],
}

impl<'a> RegisterView<'a> {
    pub const fn new(regs: &'a mut [u16; 56]) -> Self {
        Self {
            regs,
            modified: [false; 56],
        }
    }

    // Generic getter for any defined field.
    pub fn get<F: FieldAccess>(&self) -> F::Value {
        let reg = self.regs[F::ADDR as usize];
        F::get(reg)
    }

    // Generic setter for any defined field; asserts on value width and records modification.
    pub fn set<F: FieldAccess>(&mut self, val: F::Value) {
        let idx = F::ADDR as usize;
        let old = self.regs[idx];
        let new = F::set(old, val);
        if new != old {
            self.regs[idx] = new;
            self.modified[idx] = true;
        }
    }
}

// XXX(RLB) A first draft of these controls was generated by ChatGPT.  I have verified their
// correctness for the registers that are touched in AudioControl::init().  If you're going to use
// any other registers, you should verify that they match the data sheet.

// R0 (0x00) Left Input Volume
define_field!(InputPgaVolumeUpdate, 0x00, 8, 1, bool);
define_field!(LeftInputAnalogMute, 0x00, 7, 1, bool);
define_field!(LeftPgaZeroCross, 0x00, 6, 1, bool);
define_field!(LeftPgaVolume, 0x00, 0, 6, u8);

// R1 (0x01) Right Input Volume
define_field!(InputPgaVolumeUpdateRight, 0x01, 8, 1, bool);
define_field!(RightInputAnalogMute, 0x01, 7, 1, bool);
define_field!(RightPgaZeroCross, 0x01, 6, 1, bool);
define_field!(RightPgaVolume, 0x01, 0, 6, u8);

// R2 (0x02) LOUT1 volume
define_field!(HeadphoneOutVolumeUpdate, 0x02, 8, 1, bool);
define_field!(LeftOutZeroCross, 0x02, 7, 1, bool);
define_field!(LeftHeadphoneVolume, 0x02, 0, 7, u8);

// R3 (0x03) ROUT1 volume
define_field!(HeadphoneOutVolumeUpdateRight, 0x03, 8, 1, bool);
define_field!(RightOutZeroCross, 0x03, 7, 1, bool);
define_field!(RightHeadphoneVolume, 0x03, 0, 7, u8);

// R4 (0x04) Clocking (1)
define_field!(Adc1Divider, 0x04, 6, 3, u8);
define_field!(DacDivider, 0x04, 3, 3, u8);
define_field!(SysClkDiv, 0x04, 1, 2, u8);
define_field!(ClockSelect, 0x04, 0, 1, bool);

// R5 (0x05) ADC & DAC Control (CTR1)
define_field!(Dac6dBAttenuateEnable, 0x05, 7, 1, bool);
define_field!(AdcPolarityControl, 0x05, 5, 2, u8);
define_field!(DacSoftMuteEnable, 0x05, 3, 1, bool);
define_field!(DeEmphasisControl, 0x05, 3, 2, u8);
define_field!(AdcHighPassDisable, 0x05, 0, 1, bool);

// R6 (0x06) ADC & DAC Control (CTR2)
define_field!(DacSlopeMode, 0x06, 1, 1, bool);
define_field!(DacSoftMuteRampSlow, 0x06, 2, 1, bool);
define_field!(DacSoftMuteMode, 0x06, 3, 1, bool);

// R7 (0x07) Audio Interface
define_field!(AdcLeftRightSwap, 0x07, 8, 1, bool);
define_field!(BclkInvert, 0x07, 7, 1, bool);
define_field!(AudioInterfaceMasterMode, 0x07, 6, 1, bool);
define_field!(DacLeftRightSwap, 0x07, 5, 1, bool);
define_field!(LrcPolarityOrDspMode, 0x07, 4, 1, bool);
define_field!(AudioWordLength, 0x07, 2, 2, u8);
define_field!(AudioFormat, 0x07, 0, 2, u8);

// R8 (0x08) Clocking (2)
define_field!(ClassDSysclkDivider, 0x08, 6, 3, u8);
define_field!(BclkFrequency, 0x08, 0, 4, u8);

// R9 (0x09) Audio Interface
define_field!(WordLength, 0x09, 2, 2, u8);
define_field!(DacCompanding, 0x09, 3, 2, u8);
define_field!(AdcCompanding, 0x09, 0, 2, u8);

// R10 (0x0A) Left DAC Volume
define_field!(DacVolumeUpdate, 0x0A, 8, 1, bool);
define_field!(LeftDacDigitalVolume, 0x0A, 0, 8, u8);

// R11 (0x0B) Right DAC Volume
define_field!(DacVolumeUpdateRight, 0x0B, 8, 1, bool);
define_field!(RightDacDigitalVolume, 0x0B, 0, 8, u8);

// R12-R14 (0x0C-0x0E) Reserved

// R15 (0x0F) Reset register (special behavior).

// R16 (0x10) 3D control
define_field!(ThreeDEnable, 0x10, 2, 1, bool);
define_field!(ThreeDLowerCutSelect, 0x10, 1, 1, bool);
define_field!(ThreeDUpperCutSelect, 0x10, 0, 1, bool);
define_field!(ThreeDControlRaw, 0x10, 0, 9, u16);

// R17 (0x11) ALC1
// R18 (0x12) ALC2
// R19 (0x12) ALC3

// R20 (0x14) Noise gate
define_field!(NoiseGateThreshold, 0x14, 3, 5, u8);
define_field!(NoiseGateEnable, 0x14, 0, 1, bool);

// R21 (0x15) Left ADC volume
define_field!(LeftAdcDigitalVolume, 0x15, 0, 8, u8);
define_field!(AdcVolumeUpdateLeft, 0x15, 8, 1, bool);

// R22 (0x16) Right ADC volume
define_field!(RightAdcDigitalVolume, 0x16, 0, 8, u8);
define_field!(AdcVolumeUpdateRight, 0x16, 8, 1, bool);

// R23 (0x17) Additional Control (1)
define_field!(ThermalShutDownEnable, 0x17, 8, 1, bool);
define_field!(AnalogBiasOptimisation, 0x17, 6, 2, u8);
define_field!(DacMonoMix, 0x17, 4, 1, bool);
define_field!(AdcDataOutputSelect, 0x17, 2, 2, u8);
define_field!(TimeoutClockSelect, 0x17, 1, 1, bool);
define_field!(TimeoutEnable, 0x17, 0, 1, bool);

// R24 (0x18) Additional Control (2)
define_field!(AdclrcDaclrcMode, 0x18, 2, 1, bool);
define_field!(Reg24Raw, 0x18, 0, 9, u16);

// R25 (0x19) Power Management (1)
define_field!(PowerMgmt1VmidSelect, 0x19, 7, 2, u8);
define_field!(PowerMgmt1VrefEnable, 0x19, 6, 1, bool);
define_field!(PowerMgmt1AinLeftEnable, 0x19, 5, 1, bool);
define_field!(PowerMgmt1AinRightEnable, 0x19, 4, 1, bool);
define_field!(PowerMgmt1EnableAdcLeft, 0x19, 3, 1, bool);
define_field!(PowerMgmt1EnableAdcRight, 0x19, 2, 1, bool);
define_field!(MicrophoneBiasEnable, 0x19, 1, 1, bool);
define_field!(MasterClockDisable, 0x19, 0, 1, bool);

// R26 (0x1A) Power Management (2)
define_field!(LeftDacEnable, 0x1A, 8, 1, bool);
define_field!(RightDacEnable, 0x1A, 7, 1, bool);
define_field!(LeftOutput1Enable, 0x1A, 6, 1, bool);
define_field!(RightOutput1Enable, 0x1A, 5, 1, bool);
define_field!(LeftSpeakerEnable, 0x1A, 4, 1, bool);
define_field!(RightSpeakerEnable, 0x1A, 3, 1, bool);
define_field!(Out3Enable, 0x1A, 1, 1, bool);
define_field!(PllEnable, 0x1A, 0, 1, bool);

// R27 (0x1B) Additional Control (3)
define_field!(VrefToAnalogueResistance, 0x1B, 6, 1, bool);
define_field!(CaplessHeadphoneSwitchEnable, 0x1B, 3, 1, bool);
define_field!(AdcAlcSampleRateSelect, 0x1B, 0, 3, u8);

// R28 (0x1C) Anti-pop 1
// R29 (0x1D) Anti-pop 2

// R30 (0x1E) Reserved
// R31 (0x1F) Reserved

// R32 (0x20) ADCL signal path
define_field!(LeftInput1ToInverting, 0x20, 8, 1, bool);
define_field!(LeftInput3ToNonInverting, 0x20, 7, 1, bool);
define_field!(LeftInput2ToNonInverting, 0x20, 6, 1, bool);
define_field!(LeftBoostGain, 0x20, 4, 2, u8);
define_field!(LeftInputToBoost, 0x20, 3, 1, bool);

// R33 (0x21) ADCR signal path
define_field!(RightMicBoost, 0x21, 4, 2, u8);
define_field!(AdcrSignalPathRaw, 0x21, 0, 9, u16);

// R34 (0x22) Left Out Mix (1)
define_field!(LeftDacToOutputMixer, 0x22, 8, 1, bool);
define_field!(LeftInput3ToOutputMixer, 0x22, 7, 1, bool);
define_field!(LeftInput3ToOutputMixerVolume, 0x22, 4, 3, u8);

// R35 (0x23) Reserved
// R36 (0x24) Reserved

// R37 (0x25) Right Out Mix (2)
define_field!(RightDacToOutputMixer, 0x25, 8, 1, bool);
define_field!(RightInput3ToOutputMixer, 0x25, 7, 1, bool);
define_field!(RightInput3ToOutputMixerVolume, 0x25, 4, 3, u8);

// R38 (0x26) Mono out Mix (1)
// R39 (0x27) Mono out Mix (2)

// R40 (0x28) LOUT2 volume
define_field!(LeftSpeakerVolumeUpdate, 0x28, 8, 1, bool);
define_field!(LeftSpeakerZeroCross, 0x28, 7, 1, bool);
define_field!(LeftSpeakerVolume, 0x28, 0, 7, u8);

// R41 (0x29) ROUT2 volume
define_field!(RightSpeakerVolumeUpdate, 0x28, 8, 1, bool);
define_field!(RightSpeakerZeroCross, 0x28, 7, 1, bool);
define_field!(RightSpeakerVolume, 0x28, 0, 7, u8);

// R42 (0x2A) MONOOUT volume
define_field!(MonoOutVolume, 0x2A, 6, 1, bool);

// R43 (0x2B) Input Boost Mixer (1)
define_field!(Linput3Boost, 0x2B, 4, 3, u8);
define_field!(Linput2Boost, 0x2B, 1, 3, u8);

// R44 (0x2C) Input Boost Mixer (2)
define_field!(Rinput3Boost, 0x2C, 4, 3, u8);
define_field!(Rinput2Boost, 0x2C, 1, 3, u8);

// R45 (0x2D) Bypass (1)
define_field!(LeftBoostToLeftOutputMix, 0x2D, 7, 1, bool);
define_field!(LeftBoostToLeftOutputMixVolume, 0x2D, 4, 3, u8);

// R46 (0x2E) Bypass (2)
define_field!(RightBoostToRightOutputMix, 0x2E, 7, 1, bool);
define_field!(RightBoostToRightOutputMixVolume, 0x2E, 4, 3, u8);

// R47 (0x2F) Power Management (3)
define_field!(LeftMicEnable, 0x2F, 5, 1, bool);
define_field!(RightMicEnable, 0x2F, 4, 1, bool);
define_field!(LeftOutputMixEnable, 0x2F, 3, 1, bool);
define_field!(RightOutputMixEnable, 0x2F, 2, 1, bool);

// R48 (0x30) Additional Control (4)

// R49 (0x31) Class D Control (1)
define_field!(ClassDSpeakerOutputEnable, 0x31, 6, 2, u8);

// R50 (0x32) Reserved

// R51 (0x33) Class D Control (3)
define_field!(SpeakerDcGain, 0x33, 3, 3, u8);
define_field!(SpeakerAcGain, 0x33, 0, 3, u8);

// R52 (0x34) PLL N
define_field!(OpClockDivider, 0x34, 6, 3, u8);
define_field!(IntegerModeEnable, 0x34, 5, 1, bool);
define_field!(PllRescale, 0x34, 4, 1, bool);
define_field!(PllN, 0x34, 0, 4, u8);

// R53, R54, R55 (0x35, 0x36, 0x37) PLL K
define_field!(PllKMsb, 0x35, 0, 8, u8);
define_field!(PllKMid, 0x36, 0, 8, u8);
define_field!(PllKLsb, 0x37, 0, 8, u8);
