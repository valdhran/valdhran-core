use axum::Json;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

// Extensiones a ignorar — binarios, assets, imágenes, fuentes
const IGNORED_EXTENSIONS: &[&str] = &[
    "png", "svg", "jpg", "jpeg", "gif", "ico", "webp", "bmp",
    "woff", "woff2", "ttf", "otf", "eot",
    "pdf", "zip", "tar", "gz", "lock",
];

// Directorios a ignorar
const IGNORED_DIRS: &[&str] = &["target", ".git", "node_modules", ".cargo"];

#[derive(Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub crate_type: String, // "library" | "binary"
    pub path: String,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub kind: String, // "internal" | "external"
}

#[derive(Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub stats: GraphStats,
}

#[derive(Serialize)]
pub struct GraphStats {
    pub total_crates: usize,
    pub total_dependencies: usize,
    pub internal_dependencies: usize,
    pub files_scanned: usize,
}

pub async fn graphify() -> Json<GraphResponse> {
    let workspace_root = find_workspace_root();
    let graph = build_graph(&workspace_root);
    Json(graph)
}

fn find_workspace_root() -> PathBuf {
    // Desde el binario compilado, subir hasta encontrar Cargo.toml con [workspace]
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        let candidate = current.join("Cargo.toml");
        if candidate.exists() {
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                if content.contains("[workspace]") {
                    return current;
                }
            }
        }
        if !current.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

fn should_ignore_path(path: &Path) -> bool {
    // Ignorar directorios conocidos
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();
        if IGNORED_DIRS.iter().any(|d| *d == s.as_ref()) {
            return true;
        }
    }
    // Ignorar extensiones de assets/binarios
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        return IGNORED_EXTENSIONS.iter().any(|e| *e == &ext);
    }
    false
}

fn build_graph(workspace_root: &Path) -> GraphResponse {
    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut files_scanned = 0usize;

    // Leer Cargo.toml raíz para obtener members del workspace
    let root_cargo = workspace_root.join("Cargo.toml");
    let members = parse_workspace_members(&root_cargo);

    // Mapa name → path para detectar dependencias internas
    let mut crate_names: HashMap<String, String> = HashMap::new();

    // Primera pasada: recolectar nodos
    for member in &members {
        let cargo_path = workspace_root.join(member).join("Cargo.toml");
        if !cargo_path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            files_scanned += 1;
            if let Some((name, crate_type)) = parse_crate_meta(&content) {
                crate_names.insert(name.clone(), member.clone());
                nodes.push(GraphNode {
                    id: name.clone(),
                    label: name.clone(),
                    crate_type,
                    path: member.clone(),
                });
            }
        }
    }

    // Segunda pasada: recolectar aristas
    for member in &members {
        let cargo_path = workspace_root.join(member).join("Cargo.toml");
        if !cargo_path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if let Some((from_name, _)) = parse_crate_meta(&content) {
                let deps = parse_dependencies(&content);
                for dep in deps {
                    let kind = if crate_names.contains_key(&dep) {
                        "internal"
                    } else {
                        "external"
                    };
                    edges.push(GraphEdge {
                        from: from_name.clone(),
                        to: dep,
                        kind: kind.to_string(),
                    });
                }
            }
        }
    }

    // Contar archivos Rust escaneados (excluyendo assets)
    let walker = WalkDir::new(workspace_root)
        .into_iter()
        .filter_entry(|e| !should_ignore_path(e.path()));

    for entry in walker.flatten() {
        if entry.path().extension().map(|e| e == "rs").unwrap_or(false) {
            files_scanned += 1;
        }
    }

    let internal_deps = edges.iter().filter(|e| e.kind == "internal").count();
    let total_deps = edges.len();

    GraphResponse {
        stats: GraphStats {
            total_crates: nodes.len(),
            total_dependencies: total_deps,
            internal_dependencies: internal_deps,
            files_scanned,
        },
        nodes,
        edges,
    }
}

fn parse_workspace_members(cargo_toml: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(cargo_toml) else {
        return vec![];
    };
    let Ok(value) = content.parse::<toml::Value>() else {
        return vec![];
    };
    value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_crate_meta(content: &str) -> Option<(String, String)> {
    let value = content.parse::<toml::Value>().ok()?;
    let name = value
        .get("package")?
        .get("name")?
        .as_str()?
        .to_string();
    let crate_type = if value.get("lib").is_some() {
        "library"
    } else {
        "binary"
    }
    .to_string();
    Some((name, crate_type))
}

fn parse_dependencies(content: &str) -> Vec<String> {
    let Ok(value) = content.parse::<toml::Value>() else {
        return vec![];
    };
    let mut deps = Vec::new();
    if let Some(table) = value.get("dependencies").and_then(|d| d.as_table()) {
        for key in table.keys() {
            deps.push(key.clone());
        }
    }
    deps
}
