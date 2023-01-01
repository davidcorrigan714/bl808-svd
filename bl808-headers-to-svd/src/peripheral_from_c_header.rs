use std::{
    fs::{self},
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use pest::Parser;
extern crate pest;

use svd_rs::{
    Access, Field, FieldInfo, PeripheralInfo, Register, RegisterCluster, RegisterInfo,
    ValidateLevel,
};

#[derive(Parser)]
#[grammar = "peripheral_from_c_header.pest"] // relative to src
struct HeaderRegParser;

pub fn peripheral_from_c_header(
    file: &Path,
    base_address: u64,
    name: String,
) -> Result<PeripheralInfo> {
    let new_registers = registers_from_c_header(file)?;

    Ok(PeripheralInfo::builder()
        .name(name)
        .registers(Some(new_registers))
        .base_address(base_address)
        .build(ValidateLevel::Weak)
        .unwrap())
}

pub fn append_registers_from_c_header(file: &Path, peripheral: &mut PeripheralInfo) -> Result<()> {
    let mut new_registers = registers_from_c_header(file)?;

    match &mut peripheral.registers {
        Some(registers) => registers.append(&mut new_registers),
        None => peripheral.registers = Some(new_registers),
    }

    Ok(())
}

pub fn registers_from_c_header(file: &Path) -> Result<Vec<RegisterCluster>> {
    let package_string = fs::read_to_string(file)
        .with_context(|| format!("Error reading file: {}", file.display()))?;

    let reg_file = HeaderRegParser::parse(Rule::reg_file, &package_string)?
        .next()
        .unwrap();

    if Rule::reg_file != reg_file.as_rule() {
        return Err(anyhow!("Missing reg_file rule"));
    }
    let mut reg_file = reg_file.into_inner();

    let peripheral = reg_file.next().unwrap();
    if Rule::peripheral != peripheral.as_rule() {
        return Err(anyhow!("Missing peripheral."));
    }
    let mut peripheral = peripheral.into_inner();

    let mut _peripheral_name = peripheral.next().unwrap().as_str();
    _peripheral_name = &_peripheral_name[..(_peripheral_name.len() - 4)];

    let registers = peripheral.next().unwrap();
    if Rule::registers != registers.as_rule() {
        return Err(anyhow!("Expected register listing"));
    }

    let mut new_registers = Vec::<RegisterCluster>::new();

    for register in registers.into_inner() {
        match register.as_rule() {
            Rule::reserved_register => {
                // TODO
            }
            Rule::register => {
                let mut register_inner = register.into_inner();
                let mut register_header = register_inner.next().unwrap().into_inner();
                let offset: u32 =
                    u32::from_str_radix(register_header.next().unwrap().as_str(), 16)?;
                let register_name = register_header.next().unwrap().as_str();

                let mut fields = Vec::<Field>::new();

                for field in register_inner {
                    let mut field_data = field.into_inner();
                    let field_name = field_data.next().unwrap().as_str();
                    let _size: u8 = field_data.next().unwrap().as_str().parse()?;
                    let first_position = field_data.next().unwrap();
                    let start_pos: u32 = first_position.as_str().parse()?;
                    let end_pos: u32 = match first_position.as_rule() {
                        Rule::field_pos_start => field_data.next().unwrap().as_str().parse()?,
                        _ => start_pos,
                    };

                    let access: Option<Access> = match field_data.next().unwrap().as_str() {
                        "RW" => Some(svd_rs::Access::ReadWrite),
                        "rw" => Some(svd_rs::Access::ReadWrite),
                        "RWAC" => Some(svd_rs::Access::ReadWrite),
                        "RW1C" => Some(svd_rs::Access::ReadWrite),
                        "ROC" => Some(svd_rs::Access::ReadOnly),
                        "r/w" => Some(svd_rs::Access::ReadWrite),
                        "w" => Some(svd_rs::Access::WriteOnly),
                        "rsvd" => None,
                        "RSVD" => None,
                        "None" => None,
                        "w1c" => Some(svd_rs::Access::WriteOnce),
                        "RO" => Some(svd_rs::Access::ReadOnly),
                        "r" => Some(svd_rs::Access::ReadOnly),
                        "R" => Some(svd_rs::Access::ReadOnly),
                        "WO" => Some(svd_rs::Access::WriteOnly),
                        "w1p" => Some(svd_rs::Access::WriteOnce), // TODO: Think this is the right mapping
                        access_mode => {
                            println!("Unknown access mode: {}", access_mode);
                            return Err(anyhow!(format!("Unknown access mode: {}", access_mode)));
                        }
                    };

                    let _mask: u32 = u32::from_str_radix(field_data.next().unwrap().as_str(), 16)
                        .unwrap_or_default();

                    let field = FieldInfo::builder()
                        .name(field_name.to_string())
                        .bit_range(svd_rs::BitRange::from_msb_lsb(start_pos, end_pos))
                        .access(access)
                        .build(svd_rs::ValidateLevel::Strict)?;
                    fields.push(Field::Single(field));
                }

                new_registers.push(RegisterCluster::Register(Register::Single(
                    RegisterInfo::builder()
                        .name(register_name.to_string())
                        .address_offset(offset)
                        .fields(Some(fields))
                        .build(ValidateLevel::Weak)?,
                )));
            }
            _ => {
                return Err(anyhow!("Unmatched register rule"));
            }
        }
    }

    Ok(new_registers)
}
