use super::binary::{
    BinWriter, CfbDoc, encode_flags, from_mm, v7_layer_id, FLAG_SAVED, FLAG_UNLOCKED,
};
use super::{hole_type_byte, pad_shape_byte, pcb_layer};
use crate::error::Result;
use crate::ir::{FootprintIr, IrPad};
use crate::util::{altium_section_key, unique_id};
use std::path::Path;

pub fn write(path: &Path, fp: &FootprintIr) -> Result<()> {
    let key = altium_section_key(&fp.name);
    let mut cfb = CfbDoc::create(path)?;

    cfb.stream("FileHeader", &file_header())?;

    cfb.storage("Library")?;
    cfb.stream("Library/Header", &i32_stream(1))?;
    cfb.stream("Library/Data", &library_data(&key))?;

    cfb.storage("Library/Models")?;
    cfb.stream("Library/Models/Header", &i32_stream(0))?;
    cfb.stream("Library/Models/Data", &[])?;

    cfb.storage("Library/LayerKindMapping")?;
    cfb.stream("Library/LayerKindMapping/Header", &i32_stream(1))?;
    cfb.stream("Library/LayerKindMapping/Data", &layer_kind_mapping())?;

    cfb.storage("Library/Textures")?;
    cfb.stream("Library/Textures/Header", &i32_stream(0))?;
    cfb.stream("Library/Textures/Data", &[])?;

    cfb.storage(&key)?;
    let prim_count = fp.pads.len() + fp.tracks.len() + fp.circles.len() + fp.arcs.len() + fp.regions.len();
    cfb.stream(&format!("{key}/Header"), &i32_stream(prim_count as i32))?;
    cfb.stream(&format!("{key}/Parameters"), &footprint_params(fp, &key))?;
    cfb.stream(&format!("{key}/WideStrings"), &empty_params())?;
    cfb.stream(&format!("{key}/Data"), &footprint_data(fp, &key))?;

    cfb.finish()
}

fn i32_stream(v: i32) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

fn file_header() -> Vec<u8> {
    let mut w = BinWriter::new();
    let version = "PCB 6.0 Binary Library File";
    w.write_i32(version.len() as i32);
    w.write_pascal_short(version);
    w.write_f64(5.01);
    let uid = unique_id();
    w.write_i32(uid.len() as i32);
    w.write_pascal_short(&uid);
    w.into_vec()
}

fn library_data(name: &str) -> Vec<u8> {
    let mut w = BinWriter::new();
    w.write_params(&[
        ("HEADER", "PCB 6.0 Binary Library File".into()),
        ("WEIGHT", "1".into()),
    ]);
    w.write_u32(1);
    w.write_string_block(name);
    w.into_vec()
}

fn layer_kind_mapping() -> Vec<u8> {
    let mut w = BinWriter::new();
    let text = "1.0\0";
    let bytes: Vec<u8> = text.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    w.write_i32(bytes.len() as i32);
    w.write_bytes(&bytes);
    w.write_u32(0); // signature
    w.write_u32(0); // count
    w.into_vec()
}

fn footprint_params(fp: &FootprintIr, name: &str) -> Vec<u8> {
    let mut w = BinWriter::new();
    w.write_params(&[
        ("PATTERN", name.into()),
        ("HEIGHT", "0mil".into()),
        ("DESCRIPTION", fp.description.clone()),
        ("ITEMGUID", "".into()),
        ("REVISIONGUID", "".into()),
    ]);
    w.into_vec()
}

fn empty_params() -> Vec<u8> {
    let mut w = BinWriter::new();
    w.write_params(&[]);
    w.into_vec()
}

fn footprint_data(fp: &FootprintIr, name: &str) -> Vec<u8> {
    let mut w = BinWriter::new();
    w.write_string_block(name);

    for pad in &fp.pads {
        w.write_u8(2);
        write_pad(&mut w, pad);
    }

    for track in &fp.tracks {
        let layer = pcb_layer(track.layer, 0.0);
        let width = from_mm(track.width.max(0.05));
        for win in track.points.windows(2) {
            w.write_u8(4);
            write_track(&mut w, layer, win[0], win[1], width);
        }
    }

    for c in &fp.circles {
        let layer = pcb_layer(c.layer, 0.0);
        w.write_u8(1);
        write_arc(
            &mut w,
            layer,
            from_mm(c.x),
            from_mm(c.y),
            from_mm(c.radius.abs()),
            0.0,
            360.0,
            from_mm(c.width.max(0.05)),
        );
    }

    for a in &fp.arcs {
        let layer = pcb_layer(a.layer, 0.0);
        w.write_u8(1);
        write_arc(
            &mut w,
            layer,
            from_mm(a.x),
            from_mm(a.y),
            from_mm(a.radius.abs()),
            a.start,
            a.end,
            from_mm(a.width.max(0.05)),
        );
    }

    for r in &fp.regions {
        if r.points.len() < 3 {
            continue;
        }
        w.write_u8(11);
        write_region(&mut w, pcb_layer(r.layer, 0.0), &r.points);
    }

    w.into_vec()
}

fn write_common(w: &mut BinWriter, layer: u8) {
    w.write_u8(layer);
    w.write_u16(encode_flags());
    w.write_u16(0xFFFF); // net
    w.write_u16(0xFFFF); // polygon
    w.write_u16(0xFFFF); // component
    w.write_u32(0xFFFF_FFFF);
}

fn write_track(w: &mut BinWriter, layer: u8, start: (f64, f64), end: (f64, f64), width: i32) {
    w.write_block(0, |w| {
        write_common(w, layer);
        w.write_coord_point(from_mm(start.0), from_mm(start.1));
        w.write_coord_point(from_mm(end.0), from_mm(end.1));
        w.write_coord(width);
        w.write_i16(0);
        w.write_coord(0);
        w.write_i16(0);
        w.write_u32(v7_layer_id(layer));
        w.write_u8(0);
        w.write_bytes(&[0, 0, 0]);
    });
}

fn write_arc(
    w: &mut BinWriter,
    layer: u8,
    x: i32,
    y: i32,
    radius: i32,
    start: f64,
    end: f64,
    width: i32,
) {
    w.write_block(0, |w| {
        write_common(w, layer);
        w.write_coord_point(x, y);
        w.write_coord(radius);
        w.write_f64(start);
        w.write_f64(end);
        w.write_coord(width);
        w.write_i16(0);
        w.write_coord(0);
        w.write_u8(0);
        w.write_u32(v7_layer_id(layer));
        w.write_u8(0);
        w.write_bytes(&[0, 0, 0]);
    });
}

fn write_region(w: &mut BinWriter, layer: u8, points: &[(f64, f64)]) {
    w.write_block(0, |w| {
        write_common_region(w, layer);
        w.write_u8(0);
        w.write_u16(0);
        w.write_u8(0);
        w.write_u8(0);
        let layer_name = match layer {
            1 => "TOP",
            32 => "BOTTOM",
            33 => "TOPOVERLAY",
            34 => "BOTTOMOVERLAY",
            74 => "MULTILAYER",
            _ => "TOPOVERLAY",
        };
        w.write_params_raw(&format!(
            "V7_LAYER={layer_name}|NAME=|KIND=0|SUBPOLYINDEX=0|UNIONINDEX=0|ARCRESOLUTION=0.5mil|ISSHAPEBASED=FALSE|CAVITYHEIGHT=0mil"
        ));
        w.write_u32(points.len() as u32);
        for (x, y) in points {
            w.write_f64(from_mm(*x) as f64);
            w.write_f64(from_mm(*y) as f64);
        }
    });
}

fn write_common_region(w: &mut BinWriter, layer: u8) {
    w.write_u8(layer);
    w.write_u16(encode_flags());
    w.write_u16(0xFFFF);
    w.write_u16(0); // polygon index 0 for regions
    w.write_u16(0xFFFF);
    w.write_u32(0xFFFF_FFFF);
}

fn write_pad(w: &mut BinWriter, pad: &IrPad) {
    let layer = pcb_layer(pad.layer, pad.hole);
    let shape = pad_shape_byte(&pad.shape, pad.width, pad.height);
    w.write_string_block(&pad.designator);
    w.write_string_block("");
    w.write_string_block("|&|0");
    w.write_block(0, |_| {}); // empty subrecord 4

    let size_x = from_mm(pad.width);
    let size_y = from_mm(pad.height);
    let hole = from_mm(pad.hole);
    let loc_x = from_mm(pad.x);
    let loc_y = from_mm(pad.y);

    w.write_block(0, |w| {
        write_common(w, layer);
        w.write_coord_point(loc_x, loc_y);
        w.write_coord_point(size_x, size_y);
        w.write_coord_point(size_x, size_y);
        w.write_coord_point(size_x, size_y);
        w.write_coord(hole);
        w.write_u8(shape);
        w.write_u8(shape);
        w.write_u8(shape);
        w.write_f64(crate::easyeda::normalize_angle(pad.rotation));
        w.write_bool(true);
        w.write_bytes(&pad_extended_tail(pad, layer));
    });

    // size/shape block for hole type
    w.write_block(0, |w| {
        for _ in 0..29 {
            w.write_i32(size_x);
        }
        for _ in 0..29 {
            w.write_i32(size_y);
        }
        for _ in 0..29 {
            w.write_u8(shape);
        }
        w.write_u8(0);
        w.write_u8(hole_type_byte(&pad.hole_shape));
        w.write_i32(from_mm(pad.hole_slot));
        w.write_f64(0.0);
        for _ in 0..32 {
            w.write_i32(0);
        }
        for _ in 0..32 {
            w.write_i32(0);
        }
        w.write_u8(if shape == 9 { 1 } else { 0 });
        for _ in 0..32 {
            w.write_u8(shape);
        }
        for _ in 0..32 {
            w.write_u8(50);
        }
    });
}

fn pad_extended_tail(pad: &IrPad, layer: u8) -> Vec<u8> {
    let mut ext = PAD_TAIL.to_vec();
    put_i32(&mut ext, 114 - 61, v7_layer_id(layer) as i32);
    put_i32(&mut ext, 90 - 61, from_mm(0.0));
    let _ = pad;
    ext
}

fn put_i32(buf: &mut [u8], off: usize, v: i32) {
    if off + 4 <= buf.len() {
        buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }
}

/// Canonical 141-byte pad SubRecord-5 tail (offsets 61-201) from AltiumSharp.
const PAD_TAIL: [u8; 141] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA0, 0x86, 0x01, 0x00, 0x04, 0x00, 0xA0, 0x86, 0x01,
    0x00, 0x40, 0x0D, 0x03, 0x00, 0x40, 0x0D, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x9C, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x40, 0x9C, 0x00, 0x00,
    0x00, 0x64, 0x9A, 0x92, 0x26, 0x10, 0xC7, 0xE4, 0x41, 0xA3, 0x2B, 0x29, 0x17, 0xA5, 0x35, 0x2E,
    0x67, 0x7F, 0xAB, 0x21, 0x20, 0xC3, 0x0B, 0x32, 0x47, 0xAD, 0xCE, 0x6C, 0xB7, 0xB8, 0xC9, 0x7E,
    0x68, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0x01, 0x1A,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[allow(dead_code)]
fn _flags() -> u16 {
    FLAG_UNLOCKED | FLAG_SAVED
}
