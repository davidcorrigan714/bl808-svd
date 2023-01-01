use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};
use pest::Parser;
extern crate pest;

use svd_rs::{
    Access, Field, FieldInfo, PeripheralInfo, Register, RegisterCluster, RegisterInfo,
    ValidateLevel,
};

#[derive(Parser)]
#[grammar = "peripheral_from_docs.pest"] // relative to src
struct DocsRegParser;

pub fn peripheral_from_doc_rst(
    file: &Path,
    peripheral_name: String,
    alt_base: Option<u64>,
) -> Result<PeripheralInfo> {
    let file_string = fs::read_to_string(file)
        .with_context(|| format!("Error reading file: {}", file.display()))?
        .replace("Vsync|Hsync", "Vsync or Hsync");

    let peripheral_file = DocsRegParser::parse(Rule::PERIPHERAL_FILE, &file_string)
        .with_context(|| format!("Error parsing {}", file.display()))? // unwrap the parse result
        .next()
        .with_context(|| format!("Error parsing {}", file.display()))?; // get and unwrap the `peripheral_file` rule; never fails

    if Rule::PERIPHERAL_FILE != peripheral_file.as_rule() {
        return Err(anyhow!("Missing PERIPHERAL_FILE rule"));
    }
    let registers = peripheral_file.into_inner();
    let mut new_registers = Vec::<RegisterCluster>::new();

    let mut base_address: u32 = 0;

    for register in registers {
        if register.as_rule() == Rule::EOI {
            break;
        }
        let mut register = register.into_inner();
        let register_title = register
            .next()
            .with_context(|| "Failed unwrapping register title")?
            .as_str();
        let address: u32 = parse_hex_or_zero(
            register
                .next()
                .with_context(|| "Failed unwrapping address")?
                .as_str(),
        );
        if base_address == 0 {
            base_address = address;
        }
        let offset: u32 = address - base_address;

        let mut table = register
            .next()
            .with_context(|| "Getting table")?
            .into_inner();
        table.next(); // Skip table header

        let mut fields = Vec::<Field>::new();

        for field in table {
            if field.as_rule() == Rule::ROW_FIVE_CELL {
                let mut field_inner = field.into_inner();
                // TODO: Parse descriptions that can be enumerated values
                field_inner.next();
                field_inner.next();
                field_inner.next();
                field_inner.next();
                let fields_pos = fields.len() - 1;
                let current_description = match fields[fields_pos].description.as_ref() {
                    Some(description) => description,
                    None => "",
                };
                fields[fields_pos].description = Some(format!(
                    "{}\n{}",
                    current_description,
                    field_inner
                        .next()
                        .with_context(|| "Error Processing additional description text 2")?
                        .as_str()
                ));
                continue;
            }
            let mut field_inner = field.into_inner();
            let first_bit_rule = field_inner.next().unwrap();
            let start_bit: u32;
            let end_bit: u32;
            if Rule::BIT_START == first_bit_rule.as_rule() {
                end_bit = parse_u32_or_zero(first_bit_rule.as_str());
                start_bit = parse_u32_or_zero(field_inner.next().unwrap().as_str());
            } else {
                start_bit = parse_u32_or_zero(first_bit_rule.as_str());
                end_bit = start_bit;
            }
            let name = field_inner.next().unwrap().as_str();
            let access: Option<Access> = match field_inner.next().unwrap().as_str() {
                "r/w" => Some(svd_rs::Access::ReadWrite),
                "w" => Some(svd_rs::Access::WriteOnly),
                "rsvd" => None,
                "" => None,
                "HwInit" => Some(svd_rs::Access::ReadOnly),
                "roc" => Some(svd_rs::Access::ReadOnly),
                "roc/rw" => Some(svd_rs::Access::ReadWrite),
                "rw" => Some(svd_rs::Access::ReadWrite),
                "rwac" => Some(svd_rs::Access::ReadWrite),
                "rw1c" => Some(svd_rs::Access::ReadWrite),
                "w1c" => Some(svd_rs::Access::WriteOnce),
                "w1p" => Some(svd_rs::Access::WriteOnce), // Not 100% sure this is right
                "r" => Some(svd_rs::Access::ReadOnly),
                access_mode => {
                    return Err(anyhow!(format!("Unknown access mode: {}", access_mode)));
                }
            };
            field_inner.next(); // skip reset
                                //let reset: u32 = parse_hex_or_zero(field_inner.next().expect("mask").as_str());
            let description = field_inner.next().expect("description").as_str().to_owned();
            //println!("Adding field: {}", name.to_string());
            let field = FieldInfo::builder()
                .name(name.to_string())
                .bit_range(svd_rs::BitRange::from_msb_lsb(end_bit, start_bit))
                .access(access)
                .description(Some(description))
                .build(svd_rs::ValidateLevel::Weak)
                .with_context(|| {
                    format!(
                        "Building field: '{}' on register '{}'",
                        name, register_title
                    )
                })?;
            fields.push(Field::Single(field));
        }

        // Order matters for svd2html
        fields.reverse();

        new_registers.push(RegisterCluster::Register(Register::Single(
            RegisterInfo::builder()
                .name(register_title.to_string())
                .address_offset(offset)
                .fields(Some(fields))
                .build(ValidateLevel::Weak)?,
        )));
    }

    if let Some(base) = alt_base {
        base_address = base as u32;
    }

    Ok(PeripheralInfo::builder()
        .name(peripheral_name)
        .registers(Some(new_registers))
        .base_address(base_address as u64)
        .build(ValidateLevel::Strict)?)
}

fn parse_hex_or_zero(input: &str) -> u32 {
    if input.is_empty() {
        return 0;
    }

    if input.find('\'').is_some() {
        let input2 = &input[input.find('\'').unwrap() + 2..];
        u32::from_str_radix(input2, 16).expect("Error unwrapping string from hex")
    } else {
        u32::from_str_radix(input, 16).expect("Error unwrapping string from hex")
    }
}

fn parse_u32_or_zero(input: &str) -> u32 {
    if input.is_empty() {
        return 0;
    }

    input.parse::<u32>().unwrap()
}
