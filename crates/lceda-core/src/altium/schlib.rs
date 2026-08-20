use super::binary::{BinWriter, CfbDoc, add_coord_param};
use crate::error::Result;
use crate::ir::SymbolIr;
use crate::util::{altium_section_key, unique_id};
use std::path::Path;

const BLUE_BGR: i32 = 0x00FF0000;
const RED_BGR: i32 = 0x000000FF;

pub fn write(path: &Path, symbol: &SymbolIr) -> Result<()> {
    let key = altium_section_key(&symbol.name);
    let mut cfb = CfbDoc::create(path)?;

    cfb.stream("FileHeader", &file_header(&key))?;
    cfb.stream("Storage", &empty_storage())?;
    cfb.storage(&key)?;
    cfb.stream(&format!("{key}/Data"), &component_data(symbol, &key))?;
    cfb.finish()
}

fn file_header(name: &str) -> Vec<u8> {
    let mut w = BinWriter::new();
    let uid = unique_id();
    let raw = format!(
        "|HEADER=Protel for Windows - Schematic Library Editor Binary File Version 5.0\
         |Weight=1|MinorVersion=2|UniqueID={uid}|FontIdCount=1|FontName1=Times New Roman|Size1=10\
         |UseMBCS=T|IsBOC=T|SheetStyle=9|BorderOn=T|Display_Unit=0"
    );
    w.write_params_raw(&raw);
    w.write_i32(1);
    w.write_string_block(name);
    w.into_vec()
}

fn empty_storage() -> Vec<u8> {
    let mut w = BinWriter::new();
    w.write_params(&[("HEADER", "Icon storage".into())]);
    w.into_vec()
}

fn component_data(symbol: &SymbolIr, libref: &str) -> Vec<u8> {
    let mut w = BinWriter::new();
    let uid = unique_id();
    w.write_params(&[
        ("RECORD", "1".into()),
        ("LibReference", libref.into()),
        ("ComponentDescription", symbol.description.clone()),
        ("PartCount", "1".into()),
        ("DisplayModeCount", "1".into()),
        ("IndexInSheet", "-1".into()),
        ("OwnerPartId", "-1".into()),
        ("CurrentPartId", "1".into()),
        ("LibraryPath", "*".into()),
        ("SourceLibraryName", "*".into()),
        ("SheetPartFileName", "*".into()),
        ("TargetFileName", "*".into()),
        ("UniqueID", uid),
        ("Color", BLUE_BGR.to_string()),
    ]);

    for pin in &symbol.pins {
        write_pin(&mut w, pin);
    }

    for r in &symbol.rects {
        let mut pairs = vec![
            ("RECORD".into(), "14".into()),
            ("OwnerPartId".into(), "1".into()),
            ("LineWidth".into(), "1".into()),
            ("Color".into(), BLUE_BGR.to_string()),
            ("IsSolid".into(), "F".into()),
            ("Transparent".into(), "T".into()),
        ];
        add_coord_param(&mut pairs, "Location.X", r.x1);
        add_coord_param(&mut pairs, "Location.Y", r.y1);
        add_coord_param(&mut pairs, "Corner.X", r.x2);
        add_coord_param(&mut pairs, "Corner.Y", r.y2);
        write_named(&mut w, &pairs);
    }

    for poly in &symbol.polys {
        if poly.len() < 2 {
            continue;
        }
        for win in poly.windows(2) {
            let mut pairs = vec![
                ("RECORD".into(), "13".into()),
                ("OwnerPartId".into(), "1".into()),
                ("LineWidth".into(), "1".into()),
                ("Color".into(), BLUE_BGR.to_string()),
            ];
            add_coord_param(&mut pairs, "Location.X", win[0].0);
            add_coord_param(&mut pairs, "Location.Y", win[0].1);
            add_coord_param(&mut pairs, "Corner.X", win[1].0);
            add_coord_param(&mut pairs, "Corner.Y", win[1].1);
            write_named(&mut w, &pairs);
        }
    }

    for e in &symbol.ellipses {
        let mut pairs = vec![
            ("RECORD".into(), "8".into()),
            ("OwnerPartId".into(), "1".into()),
            ("LineWidth".into(), "1".into()),
            ("Color".into(), BLUE_BGR.to_string()),
            ("AreaColor".into(), "0".into()),
            ("IsSolid".into(), "F".into()),
            ("Transparent".into(), "T".into()),
        ];
        add_coord_param(&mut pairs, "Location.X", e.x);
        add_coord_param(&mut pairs, "Location.Y", e.y);
        add_coord_param(&mut pairs, "Radius", e.rx);
        add_coord_param(&mut pairs, "SecondaryRadius", e.ry);
        write_named(&mut w, &pairs);
    }

    w.write_params(&[("RECORD", "44".into())]);
    w.into_vec()
}

fn write_named(w: &mut BinWriter, pairs: &[(String, String)]) {
    let refs: Vec<(&str, String)> = pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    w.write_params(&refs);
}

fn write_pin(w: &mut BinWriter, pin: &crate::ir::IrPin) {
    let orient = pin_orient(pin.rotation);
    let loc_x = dxp_num(pin.x);
    let loc_y = dxp_num(pin.y);
    let len = dxp_num(pin.length.max(2.54));
    let mut conglomerate = orient;
    conglomerate |= 0x08 | 0x10; // show name + designator
    w.write_block(0x01, |w| {
        w.write_i32(2);
        w.write_u8(0);
        w.write_i16(1); // OwnerPartId
        w.write_u8(0);
        w.write_u8(0);
        w.write_u8(0);
        w.write_u8(0);
        w.write_u8(0);
        w.write_pascal_short("");
        w.write_u8(0); // FormalType
        w.write_u8(4); // Passive
        w.write_u8(conglomerate);
        w.write_i16(len as i16);
        w.write_i16(loc_x as i16);
        w.write_i16(loc_y as i16);
        w.write_i32(RED_BGR);
        w.write_pascal_short(&pin.name);
        w.write_pascal_short(&pin.number);
        w.write_pascal_short("");
        w.write_pascal_short("");
        w.write_pascal_short("");
    });
}

fn pin_orient(rotation_deg: f64) -> u8 {
    let a = crate::easyeda::normalize_angle(rotation_deg + 180.0);
    let q = ((a / 90.0).round() as i32).rem_euclid(4) as u8;
    q
}

fn dxp_num(mm: f64) -> i32 {
    // 1 DXP = 10 mil = 0.254 mm
    (mm / 0.254).round() as i32
}

// expose normalize_angle - I'll add pub(crate) in easyeda instead of this
