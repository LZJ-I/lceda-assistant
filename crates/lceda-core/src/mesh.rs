//! OBJ 网格解析，供预览使用。

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

pub fn parse_obj(text: &str, max_triangles: usize) -> Mesh {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("v ") {
            let mut parts = rest.split_whitespace();
            if let (Some(x), Some(y), Some(z)) = (parts.next(), parts.next(), parts.next()) {
                if let (Ok(x), Ok(y), Ok(z)) = (x.parse::<f32>(), y.parse::<f32>(), z.parse::<f32>()) {
                    vertices.push([x, y, z]);
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("f ") {
            let mut idx = Vec::new();
            for token in rest.split_whitespace() {
                let raw = token.split('/').next().unwrap_or("");
                if let Ok(mut n) = raw.parse::<i32>() {
                    if n < 0 {
                        n += vertices.len() as i32;
                    } else {
                        n -= 1;
                    }
                    if n >= 0 {
                        idx.push(n as u32);
                    }
                }
            }
            if idx.len() < 3 {
                continue;
            }
            let base = idx[0];
            for i in 1..idx.len() - 1 {
                triangles.push([base, idx[i], idx[i + 1]]);
                if triangles.len() >= max_triangles {
                    return Mesh { vertices, triangles };
                }
            }
        }
    }
    Mesh { vertices, triangles }
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

pub fn decimate(mesh: &Mesh, max_triangles: usize) -> Vec<[u32; 3]> {
    if mesh.triangles.len() <= max_triangles {
        return mesh.triangles.clone();
    }
    let step = (mesh.triangles.len() / max_triangles).max(1);
    mesh.triangles.iter().step_by(step).copied().take(max_triangles).collect()
}
