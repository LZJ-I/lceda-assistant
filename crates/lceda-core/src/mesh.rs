//! OBJ 网格：用 tobj 解析（含 MTL 的 Kd），供预览使用。
//! 立创 OBJ 把材质内嵌在同一文件里，导出时仍用 [`split_obj_mtl`] 拆出 `.mtl`。

use std::io::{BufReader, Cursor};

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    /// 每个三角形的漫反射色（来自 MTL `Kd`）。
    pub tri_rgb: Vec<[u8; 3]>,
}

const FALLBACK_RGB: [u8; 3] = [196, 196, 200];

pub fn load_preview_obj(text: &str) -> Result<Mesh, String> {
    let (_, mtl) = split_obj_mtl(text);
    let mut obj = String::new();
    if !text
        .lines()
        .any(|line| line.trim_start().starts_with("mtllib "))
    {
        obj.push_str("mtllib lceda.mtl\n");
    }
    obj.push_str(text);

    let mtl_src = mtl;
    let mut reader = BufReader::new(Cursor::new(obj.into_bytes()));
    let (models, materials) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ignore_points: true,
            ignore_lines: true,
            ..Default::default()
        },
        |_| {
            if mtl_src.trim().is_empty() {
                return Ok((Vec::new(), Default::default()));
            }
            let mut mtl_reader = BufReader::new(Cursor::new(mtl_src.as_bytes()));
            tobj::load_mtl_buf(&mut mtl_reader)
        },
    )
    .map_err(|e| format!("OBJ 解析失败: {e}"))?;
    let materials = materials.unwrap_or_default();

    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let mut tri_rgb = Vec::new();
    for model in models {
        let mesh = model.mesh;
        let rgb = mesh
            .material_id
            .and_then(|id| materials.get(id))
            .map(material_rgb)
            .unwrap_or(FALLBACK_RGB);
        let base = vertices.len() as u32;
        for pos in mesh.positions.chunks_exact(3) {
            vertices.push([pos[0], pos[1], pos[2]]);
        }
        for idx in mesh.indices.chunks_exact(3) {
            triangles.push([base + idx[0], base + idx[1], base + idx[2]]);
            tri_rgb.push(rgb);
        }
    }
    if vertices.is_empty() || triangles.is_empty() {
        return Err("OBJ 里没有可用三角面".into());
    }
    Ok(Mesh {
        vertices,
        triangles,
        tri_rgb,
    })
}

fn material_rgb(mat: &tobj::Material) -> [u8; 3] {
    let d = mat.diffuse.unwrap_or([0.77, 0.77, 0.78]);
    [
        (d[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (d[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (d[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
}

pub fn split_obj_mtl(text: &str) -> (String, String) {
    let mut mtl = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("newmtl") {
            mtl.push(lines[i].to_string());
            i += 1;
            while i < lines.len() {
                let token = lines[i].trim().split_whitespace().next().unwrap_or("");
                if matches!(
                    token,
                    "newmtl" | "v" | "vt" | "vn" | "f" | "o" | "g" | "s" | "usemtl" | "mtllib"
                ) {
                    break;
                }
                mtl.push(lines[i].to_string());
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    (text.to_string(), mtl.join("\n"))
}

pub fn decimate(mesh: &Mesh, max_triangles: usize) -> Mesh {
    if mesh.triangles.len() <= max_triangles {
        return mesh.clone();
    }
    let area2 = |t: &[u32; 3]| -> f32 {
        let Some(&a) = mesh.vertices.get(t[0] as usize) else {
            return 0.0;
        };
        let Some(&b) = mesh.vertices.get(t[1] as usize) else {
            return 0.0;
        };
        let Some(&c) = mesh.vertices.get(t[2] as usize) else {
            return 0.0;
        };
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        nx * nx + ny * ny + nz * nz
    };
    let mut scored: Vec<(f32, u32)> = mesh
        .triangles
        .iter()
        .enumerate()
        .map(|(i, t)| (area2(t), i as u32))
        .collect();
    scored.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let n_body = (max_triangles * 2 / 3).max(1).min(scored.len());
    let mut used = vec![false; mesh.triangles.len()];
    let mut triangles = Vec::with_capacity(max_triangles);
    let mut tri_rgb = Vec::with_capacity(max_triangles);
    let push = |i: usize, triangles: &mut Vec<[u32; 3]>, tri_rgb: &mut Vec<[u8; 3]>| {
        triangles.push(mesh.triangles[i]);
        tri_rgb.push(mesh.tri_rgb.get(i).copied().unwrap_or(FALLBACK_RGB));
    };
    for (_, i) in scored.iter().take(n_body) {
        used[*i as usize] = true;
        push(*i as usize, &mut triangles, &mut tri_rgb);
    }
    let remain = max_triangles.saturating_sub(triangles.len());
    if remain > 0 {
        let rest: Vec<u32> = (0..mesh.triangles.len() as u32)
            .filter(|i| !used[*i as usize])
            .collect();
        if !rest.is_empty() {
            let step = (rest.len() / remain).max(1);
            for i in rest.into_iter().step_by(step).take(remain) {
                push(i as usize, &mut triangles, &mut tri_rgb);
            }
        }
    }
    Mesh {
        vertices: mesh.vertices.clone(),
        triangles,
        tri_rgb,
    }
}

pub fn compact(mesh: &Mesh) -> Mesh {
    if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
        return Mesh::default();
    }
    let mut remap = vec![u32::MAX; mesh.vertices.len()];
    let mut vertices = Vec::new();
    let mut triangles = Vec::with_capacity(mesh.triangles.len());
    let mut tri_rgb = Vec::with_capacity(mesh.triangles.len());
    for (i, tri) in mesh.triangles.iter().enumerate() {
        let mut out = [0_u32; 3];
        let mut ok = true;
        for j in 0..3 {
            let old = tri[j] as usize;
            if old >= remap.len() {
                ok = false;
                break;
            }
            if remap[old] == u32::MAX {
                remap[old] = vertices.len() as u32;
                vertices.push(mesh.vertices[old]);
            }
            out[j] = remap[old];
        }
        if ok {
            triangles.push(out);
            tri_rgb.push(mesh.tri_rgb.get(i).copied().unwrap_or(FALLBACK_RGB));
        }
    }
    Mesh {
        vertices,
        triangles,
        tri_rgb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cube_obj() {
        let obj = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mesh = load_preview_obj(obj).unwrap();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.triangles.len(), 2);
    }

    #[test]
    fn parses_scientific_notation() {
        let obj = "v 1e-3 2.0 3\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let mesh = load_preview_obj(obj).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.triangles, vec![[0, 1, 2]]);
        assert!((mesh.vertices[0][0] - 0.001).abs() < 1e-6);
    }

    #[test]
    fn uses_inline_mtl_kd() {
        let obj = "\
newmtl body
Kd 0.25 0.25 0.25
newmtl pin
Kd 0.80 0.45 0.20
v 0 0 0
v 1 0 0
v 0 1 0
usemtl body
f 1 2 3
v 0 0 1
v 1 0 1
v 0 1 1
usemtl pin
f 4 5 6
";
        let mesh = load_preview_obj(obj).unwrap();
        assert_eq!(mesh.triangles.len(), 2);
        assert_eq!(mesh.tri_rgb[0], [64, 64, 64]);
        assert_eq!(mesh.tri_rgb[1], [204, 115, 51]);
    }

    #[test]
    fn parses_real_obj_if_present() {
        let Ok(text) = std::fs::read_to_string("/tmp/lceda-probe/sample.obj") else {
            return;
        };
        let mesh = load_preview_obj(&text).unwrap();
        assert!(mesh.vertices.len() > 10_000, "v={}", mesh.vertices.len());
        assert!(mesh.triangles.len() > 10_000, "f={}", mesh.triangles.len());
        let unique: std::collections::HashSet<_> = mesh.tri_rgb.iter().copied().collect();
        assert!(
            unique.len() >= 2,
            "expected multiple Kd colors, got {unique:?}"
        );
        let slim = compact(&decimate(&mesh, 5_000));
        assert!(!slim.vertices.is_empty());
        assert!(slim.triangles.len() <= 5_000);
        assert_eq!(slim.triangles.len(), slim.tri_rgb.len());
    }
}
