use std::env;
use std::io::Write;
use std::path::Path;
use std::{fs::File, path::PathBuf, str::FromStr};

extern crate pest;

use svd_rs::{Device, MaybeArray, Peripheral, PeripheralInfo, ValidateLevel};

#[macro_use]
extern crate pest_derive;

mod peripheral_from_c_header;
mod peripheral_from_doc_rst;

use peripheral_from_c_header::{append_registers_from_c_header, peripheral_from_c_header};
use peripheral_from_doc_rst::peripheral_from_doc_rst;

const HEADER_FOLDERS: [&str; 3] = [
    "M1s_BL808_SDK/components/platform/soc/bl808/bl808_std/BL808_BSP_Driver/dsp2_reg/",
    "M1s_BL808_SDK/components/platform/soc/bl808/bl808_std/BL808_BSP_Driver/Peripherals/",
    "M1s_BL808_SDK/components/platform/soc/bl808/bl808_e907_std/bl808_bsp_driver/regs/",
];

const OSD_A_BASE: u64 = 0x30013000;
const OSD_B_BASE: u64 = 0x30014000;
const OSD_DP_BASE: u64 = 0x30015000;
const OSD_BLEND0_OFFSET: u64 = 0x000;
const OSD_BLEND1_OFFSET: u64 = 0x100;
const OSD_BLEND2_OFFSET: u64 = 0x200;
const OSD_BLEND3_OFFSET: u64 = 0x300;
const OSD_DRAW_LOW_OFFSET: u64 = 0x400;
const OSD_DRAW_HIGH_OFFSET: u64 = 0x504;

fn main() {
    let mut peripherals: Vec<Peripheral> = Vec::new();

    // TODO HBN_RAM_BASE 0x20010000

    // TODO BL_CNN_BASE     0x30024000
    peripheral_from_rst("mjdec_register.rst", "MJDEC", &mut peripherals, None);
    // TODO VIDEO_BASE      0x30022000
    peripheral_from_header("mjpeg_q_reg.h", 0x30021000, "MJPEG_Q", &mut peripherals); // 0x0 0x1FC
    peripheral_from_header("mjpeg_reg.h", 0x30021000, "MJPEG", &mut peripherals); // 0x400 0x4FC
    peripheral_from_header(
        "codec_misc_reg.h",
        0x30020000,
        "CODEC_MISC",
        &mut peripherals,
    );
    // TODO mipi_reg & csi_register overlap
    peripheral_from_rst("csi_register.rst", "CSI", &mut peripherals, None);
    peripheral_from_header("mipi_reg.h", 0x3001a000, "MIPI", &mut peripherals);
    peripheral_from_rst("dsi_register.rst", "DSI", &mut peripherals, None);
    peripheral_from_rst("dbi_register.rst", "DBI", &mut peripherals, None);

    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_A_BASE + OSD_BLEND0_OFFSET,
        "OSD_A_BLEND_LAYER0",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_A_BASE + OSD_BLEND1_OFFSET,
        "OSD_A_BLEND_LAYER1",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_A_BASE + OSD_BLEND2_OFFSET,
        "OSD_A_BLEND_LAYER2",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_A_BASE + OSD_BLEND3_OFFSET,
        "OSD_A_BLEND_LAYER3",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_B_BASE + OSD_BLEND0_OFFSET,
        "OSD_B_BLEND_LAYER0",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_B_BASE + OSD_BLEND1_OFFSET,
        "OSD_B_BLEND_LAYER1",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_DP_BASE + OSD_BLEND0_OFFSET,
        "OSD_DP_BLEND_LAYER0",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_DP_BASE + OSD_BLEND1_OFFSET,
        "OSD_DP_BLEND_LAYER1",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_DP_BASE + OSD_BLEND2_OFFSET,
        "OSD_DP_BLEND_LAYER2",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_blend_reg.h",
        OSD_DP_BASE + OSD_BLEND3_OFFSET,
        "OSD_DP_BLEND_LAYER3",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_draw_l_reg.h",
        OSD_A_BASE + OSD_DRAW_LOW_OFFSET,
        "OSD_A_DRAW_LAYER_L",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_draw_h_reg.h",
        OSD_A_BASE + OSD_DRAW_HIGH_OFFSET,
        "OSD_A_DRAW_LAYER_H",
        &mut peripherals,
    );

    peripheral_from_header(
        "osd_draw_l_reg.h",
        OSD_B_BASE + OSD_DRAW_LOW_OFFSET,
        "OSD_B_DRAW_LAYER_L",
        &mut peripherals,
    );

    peripheral_from_header(
        "osd_draw_l_reg.h",
        OSD_DP_BASE + OSD_DRAW_LOW_OFFSET,
        "OSD_DP_DRAW_LAYER_L",
        &mut peripherals,
    );
    peripheral_from_header(
        "osd_draw_h_reg.h",
        OSD_DP_BASE + OSD_DRAW_HIGH_OFFSET,
        "OSD_DP_DRAW_LAYER_H",
        &mut peripherals,
    );

    peripheral_from_header("osd_probe_reg.h", 0x30012b00, "OSD_PROBE", &mut peripherals);

    // This assignment is a tad iffy but seems to be correct
    peripheral_from_header(
        "dsp2_axi_ctrl_reg.h",
        0x30012a00,
        "AXI_CTRL_NR3d",
        &mut peripherals,
    );

    // TODO DVP_TSRC1_BASE       0x30012900
    // TODO DVP_TSRC0_BASE       0x30012800
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP7",
        &mut peripherals,
        Some(0x30012700),
    );
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP6",
        &mut peripherals,
        Some(0x30012600),
    );
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP5",
        &mut peripherals,
        Some(0x30012500),
    );
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP4",
        &mut peripherals,
        Some(0x30012400),
    );
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP3",
        &mut peripherals,
        Some(0x30012300),
    );
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP2",
        &mut peripherals,
        Some(0x30012200),
    );
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP1",
        &mut peripherals,
        Some(0x30012100),
    );
    peripheral_from_rst(
        "dvp2axi_register.rst",
        "DVP0",
        &mut peripherals,
        Some(0x30012000),
    );
    peripheral_from_header("dsp2_misc_reg.h", 0x30010000, "DSP2_MISC", &mut peripherals); // 0x0  0x2FC

    peripheral_from_header("dsp2_tg_reg.h", 0x30011000, "DSP2", &mut peripherals); // 0x0  0x2FC
    let i = peripherals.len() - 1;
    append_peripheral_from_header("dsp2_front_reg.h", &mut peripherals[i]); // 0x110 0x1F0 DSP2_BASE
    append_peripheral_from_header("dsp2_middle_reg.h", &mut peripherals[i]); // 0x224 0x2F8 DSP2_base
                                                                             // TODO: Seems that back_reg has a few things blback doesn't
                                                                             //append_peripheral_from_header("dsp2_back_reg.h", &mut peripherals[i]); // 0x314 0x3FF DSP2_BASE
                                                                             // blback seems to have 0x360 - 0x3FC unique to it, but overlaps with back_reg before that with less complete register data(?)
    append_peripheral_from_header("dsp2_blback_reg.h", &mut peripherals[i]); // 0x314  & 0x900 0xF38  DSP2_BASE
    append_peripheral_from_header("dsp2_auto_reg.h", &mut peripherals[i]); // 0x444 0x454 DSP2_base
    append_peripheral_from_header("dsp2_blae_reg.h", &mut peripherals[i]); // 0x500 0x504 DSP2_base
    append_peripheral_from_header("dsp2_blawb_reg.h", &mut peripherals[i]); // 0x600 0x604 DSP2_base
    append_peripheral_from_header("dsp2_gamma_reg.h", &mut peripherals[i]); // 0x700 0x7FC DSP2_BASE

    peripheral_from_header(
        "dsp2_tg_reg.h",
        0x30016000,
        "DSP2_AWB3_BASE",
        &mut peripherals,
    ); // 0x0  0x2FC
    append_peripheral_from_header("dsp2_middle2_reg.h", &mut peripherals[i]); // 0x800 0x8B0
    append_peripheral_from_header("dsp2_middle3_reg.h", &mut peripherals[i]); // 0x500 0x584
    append_peripheral_from_header("dsp2_middle4_reg.h", &mut peripherals[i]); // 0x600 0x60C
    append_peripheral_from_header("dsp2_middle5_reg.h", &mut peripherals[i]); // 0x000 0x19C

    // TODO: double check all psram stuff
    peripheral_from_rst("psram_register.rst", "pSRAM", &mut peripherals, None);
    peripheral_from_rst("tmr_register.rst", "TIMER1", &mut peripherals, None);
    peripheral_from_rst(
        "spi_register.rst",
        "SPI1",
        &mut peripherals,
        Some(0x30008000),
    );
    // peripheral_from_header("mm_glb_reg.h", 0x30007000, "MM_GLB", &mut peripherals);
    // clkrst_reg.h seems to include mm_glb_reg.h
    peripheral_from_header(
        "clkrst_reg.h",
        0x30007000,
        "MM_GLB_CLK_RST",
        &mut peripherals,
    );
    peripheral_from_rst("2ddma_register.rst", "DMA2D", &mut peripherals, None);
    peripheral_from_header("ipc_reg.h", 0x30005000, "IPC2", &mut peripherals);
    peripheral_from_rst(
        "i2c_register.rst",
        "I2C3",
        &mut peripherals,
        Some(0x30004000),
    );
    peripheral_from_rst(
        "i2c_register.rst",
        "I2C2",
        &mut peripherals,
        Some(0x30003000),
    );
    peripheral_from_rst(
        "uart_register.rst",
        "UART3",
        &mut peripherals,
        Some(0x30002000),
    );
    peripheral_from_rst(
        "dma_register.rst",
        "DMA2",
        &mut peripherals,
        Some(0x30001000),
    );
    peripheral_from_header("mm_misc_reg.h", 0x30000000, "MM_MISC", &mut peripherals);
    peripheral_from_rst(
        "dma_register.rst",
        "DMA1",
        &mut peripherals,
        Some(0x20071000),
    );
    peripheral_from_header("ethmac_reg.h", 0x20070000, "EMAC", &mut peripherals);
    peripheral_from_rst("SDH_register.rst", "SDH", &mut peripherals, None);
    //peripheral_from_header("sdh_reg.h", 0x20060000, "SDH", &mut peripherals);
    peripheral_from_header("audio_reg.h", 0x20055000, "AUDIO", &mut peripherals);

    peripheral_from_header("usb_reg.h", 0x2007200, "USB", &mut peripherals);
    // TODO: Double check psram_reg is PSRAM_CTRL_BASE
    // peripheral_from_header("psram_reg.h", 0x20052000, "PSRAM", &mut peripherals);
    // TODO: EMI_MISC 0x20050000
    peripheral_from_header("aon_reg.h", 0x2000f000, "AON", &mut peripherals);
    // peripheral_from_header("hbn_reg.h", 0x2000F000, "HBN" &mut peripherals);
    peripheral_from_rst("HBN_register.rst", "LowPower", &mut peripherals, None);
    peripheral_from_header("pds_reg.h", 0x2000E000, "PDS", &mut peripherals);
    peripheral_from_rst(
        "dma_register.rst",
        "DMA0",
        &mut peripherals,
        Some(0x2000C000),
    );
    peripheral_from_header("sf_ctrl_reg.h", 0x2000b000, "SF_CTRL", &mut peripherals);
    // QSPI 0x2000b000
    peripheral_from_rst("lz4_register.rst", "LZ4D", &mut peripherals, None);
    peripheral_from_header("pdm_reg.h", 0x3000C000, "PDM0", &mut peripherals);
    peripheral_from_header("pdm_reg.h", 0x3000D000, "PDM1", &mut peripherals);
    peripheral_from_rst("i2s_register.rst", "I2S", &mut peripherals, None);
    // TODO ISO11898/UART2: 0x2000AA00
    peripheral_from_rst(
        "i2c_register.rst",
        "I2C1",
        &mut peripherals,
        Some(0x2000A900),
    );
    peripheral_from_header("ipc_reg.h", 0x2000a8400, "IPC1", &mut peripherals);
    peripheral_from_header("ipc_reg.h", 0x2000a8000, "IPC0", &mut peripherals);
    // CKS: TODO 0x2000a700
    peripheral_from_rst_zh_cn("ir_register.rst", "IR", &mut peripherals, Some(0x2000A600));
    peripheral_from_rst(
        "tmr_register.rst",
        "TIMER0",
        &mut peripherals,
        Some(0x2000a500),
    );
    peripheral_from_rst_zh_cn(
        "pwm_register.rst",
        "PWM",
        &mut peripherals,
        Some(0x2000A400),
    );
    peripheral_from_rst(
        "i2c_register.rst",
        "I2C0",
        &mut peripherals,
        Some(0x2000A300),
    );
    peripheral_from_rst(
        "spi_register.rst",
        "SPI0",
        &mut peripherals,
        Some(0x2000A200),
    );
    peripheral_from_rst(
        "uart_register.rst",
        "UART1",
        &mut peripherals,
        Some(0x2000A100),
    );
    peripheral_from_rst(
        "uart_register.rst",
        "UART0",
        &mut peripherals,
        Some(0x2000A000),
    );
    // L1C: 0x20009000 Docs MIA, Seems to be a simple register documented in bl808_l1c.h
    peripheral_from_header("mcu_misc_reg.h", 0x20009000, "MCU_MISC", &mut peripherals);
    peripheral_from_header("cci_reg.h", 0x20008000, "CCI", &mut peripherals);
    peripheral_from_header("ef_ctrl_reg.h", 0x20056000, "eFuse_Ctrl", &mut peripherals);
    peripheral_from_header(
        "ef_data_0_reg.h",
        0x20056000,
        "eFuse_Data0",
        &mut peripherals,
    );
    peripheral_from_header(
        "ef_data_1_reg.h",
        0x20056000,
        "eFuse_Data1",
        &mut peripherals,
    );
    peripheral_from_rst("sec_register.rst", "SEC_ENG", &mut peripherals, None);
    peripheral_from_header("sec_dbg_reg.h", 0x20003000, "SEC_DBG", &mut peripherals);
    // AGC: 0x20002c00 - Docs MIA
    // PHY: 0x20002800 - Docs MIA
    // GPIP: 0x20002000 - General purpose DAC/ADC/ACOMP interface control register
    // TODO GPIP & ADC/DAC overlap but no ACOMP
    //peripheral_from_header("gpip_reg.h", 0x20002000, "GPIP", &mut peripherals);
    peripheral_from_rst("adc_register.rst", "ADC", &mut peripherals, None);
    peripheral_from_rst("dac_register.rst", "DAC", &mut peripherals, None);
    peripheral_from_header("glb_reg.h", 0x20000000, "GLB", &mut peripherals);

    //
    //

    //peripheral_from_header("dtsrc_reg.h", 0, &mut peripherals);
    //peripheral_from_header("tzc_nsec_reg.h", 0, "TZC", &mut peripherals);
    //peripheral_from_header("tzc_sec_reg.h", 0, "TZC", &mut peripherals);
    //peripheral_from_header_m1s("bd_reg.h", 0, "TZC", &mut peripherals);

    for p in &peripherals {
        let mut max_addr: u32 = p.base_address as u32;
        for r in p.registers() {
            if (r.address_offset + max_addr) > max_addr {
                max_addr = r.address_offset + (p.base_address as u32);
            }
        }
        println!("{},{},{}", p.name, p.base_address, max_addr);
    }

    let device = Device::builder()
        .name("BL808".to_string())
        .peripherals(peripherals)
        .version("0.1".to_string())
        .description("Bouffalo Labs BL808".to_string())
        .address_unit_bits(8)
        .width(32)
        .build(ValidateLevel::Strict)
        .unwrap();
    let result = svd_encoder::encode(&device).unwrap();
    let mut file = File::create("output.svd").unwrap();
    file.write_all(result.as_bytes()).unwrap();
}

fn get_git_root() -> PathBuf {
    // TODO: Cache this
    let current_path = env::current_dir().expect("Unable to get current directory.");
    let mut current_dir = Some(current_path.as_path());
    while current_dir.is_some() {
        let potential_root = current_dir.as_ref().unwrap().join(".git");
        if potential_root.as_path().exists() {
            return PathBuf::from(potential_root.parent().unwrap());
        }
        current_dir = current_dir.unwrap().parent();
    }

    panic!("didn't find git root");
}

fn peripheral_from_rst(
    filename: &str,
    peripheral_name: &str,
    peripherals: &mut Vec<Peripheral>,
    alt_base: Option<u64>,
) {
    let peripheral = PathBuf::from_str(&format!(
        "/home/david/Documents/bl808-svd/bl_docs/BL808_RM/en/RST/{}",
        filename
    ))
    .unwrap();
    let peripheral = peripheral_from_doc_rst(&peripheral, peripheral_name.to_owned(), alt_base);
    match peripheral {
        Ok(p) => peripherals.push(Peripheral::Single(p)),
        Err(e) => {
            println!(
                "Error processing peripheral {} from {}, {:?}",
                peripheral_name, filename, e
            );
        }
    }
}

fn peripheral_from_rst_zh_cn(
    filename: &str,
    peripheral_name: &str,
    peripherals: &mut Vec<Peripheral>,
    alt_base: Option<u64>,
) {
    let repo_root = get_git_root();
    let peripheral = repo_root
        .join(PathBuf::from_str(&format!("bl_docs/BL808_RM/zh_CN/RST/{}", filename)).unwrap());
    let peripheral = peripheral_from_doc_rst(&peripheral, peripheral_name.to_owned(), alt_base);
    match peripheral {
        Ok(p) => peripherals.push(Peripheral::Single(p)),
        Err(e) => {
            println!(
                "Error processing peripheral {} from {}, {:?}",
                peripheral_name, filename, e
            );
        }
    }
}

fn peripheral_from_header(
    filename: &str,
    base_addr: u64,
    name: &str,
    peripherals: &mut Vec<Peripheral>,
) {
    let repo_root = get_git_root();
    let mut header_path: Option<PathBuf> = None;
    for folder in HEADER_FOLDERS {
        let file_path = repo_root.join(folder).join(Path::new(filename));
        let file = Path::new(&file_path);
        if file.exists() {
            header_path = Some(file_path);
            break;
        }
    }

    match header_path {
        Some(header) => match peripheral_from_c_header(&header, base_addr, name.to_owned()) {
            Ok(peripheral) => peripherals.push(Peripheral::Single(peripheral)),
            Err(e) => println!("Unable to parse: {} because: {:?}", header.display(), e),
        },
        None => println!("Header file not found: {}", filename),
    }
}

fn append_peripheral_from_header(filename: &str, peripheral: &mut MaybeArray<PeripheralInfo>) {
    let repo_root = get_git_root();
    let mut header_path: Option<PathBuf> = None;
    for folder in HEADER_FOLDERS {
        let file_path = repo_root.join(folder).join(Path::new(filename));
        let file = Path::new(&file_path);
        if file.exists() {
            header_path = Some(file_path);
            break;
        }
    }

    let peripheral = match peripheral {
        svd_rs::MaybeArray::Single(p) => p,
        svd_rs::MaybeArray::Array(_, _) => {
            panic!("Nope!");
        }
    };

    match header_path {
        Some(header) => match append_registers_from_c_header(&header, peripheral) {
            Ok(_) => {}
            Err(e) => println!("Unable to parse: {} because: {:?}", header.display(), e),
        },
        None => println!("Header file not found: {}", filename),
    }
}
