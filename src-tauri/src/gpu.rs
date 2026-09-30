use std::process::Command;

use serde::Deserialize;

use crate::models::GpuInfo;

#[derive(Deserialize)]
struct WmiGpu {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "DriverVersion")]
    driver_version: Option<String>,
    #[serde(rename = "AdapterRAM")]
    adapter_ram: Option<u64>,
}

/// Detecta a GPU principal via WMI (Win32_VideoController).
/// Prefere a melhor GPU AMD (RDNA4 > RDNA3 > RDNA2) em vez da iGPU.
pub fn detect() -> GpuInfo {
    let gpus = query_wmi().unwrap_or_default();
    if gpus.is_empty() {
        return GpuInfo {
            name: "Desconhecida".into(),
            family: "unknown".into(),
            supported: false,
            message: "Nao foi possivel detectar a GPU. O mod requer AMD RDNA3/RDNA4."
                .into(),
            ..Default::default()
        };
    }

    // Escolhe o melhor adaptador: prefere a dGPU AMD RDNA3/RDNA4 em vez da iGPU
    // (ex.: "AMD Radeon(TM) Graphics"), que aparece primeiro no WMI mas nao suporta o mod.
    let chosen = gpus
        .iter()
        .max_by_key(|g| score(g.name.as_deref().unwrap_or(""), g.adapter_ram))
        .unwrap();

    let name = chosen.name.clone().unwrap_or_else(|| "Desconhecida".into());
    let driver = chosen.driver_version.clone().unwrap_or_default();
    let (family, supported) = classify(&name);
    let message = family_message(&family, &name);

    GpuInfo {
        name,
        driver_version: driver,
        family,
        supported,
        message,
    }
}

fn query_wmi() -> Option<Vec<WmiGpu>> {
    // Usa CIM (mesma origem do WMI) via PowerShell para evitar dependencias extras.
    let script = "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion | ConvertTo-Json -Compress";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if text.starts_with('[') {
        serde_json::from_str::<Vec<WmiGpu>>(text).ok()
    } else {
        serde_json::from_str::<WmiGpu>(text).ok().map(|g| vec![g])
    }
}

/// Pontuacao para escolher o adaptador principal. Quanto mais novo o suporte
/// RDNA, maior a nota; a VRAM desempata (iGPU costuma reportar menos).
fn score(name: &str, ram: Option<u64>) -> (u8, u64) {
    let (family, _) = classify(name);
    let rank = match family.as_str() {
        "rdna4" => 4,
        "rdna3" => 3,
        "rdna2" => 2,
        "other" => 1,
        _ => 0,
    };
    (rank, ram.unwrap_or(0))
}

fn classify(name: &str) -> (String, bool) {
    let n = name.to_lowercase();
    // RX 9000 = RDNA4, RX 7000 = RDNA3.
    let rx9000 = regex::Regex::new(r"rx\s*9\d{3}").unwrap();
    let rx7000 = regex::Regex::new(r"rx\s*7\d{3}").unwrap();
    let rx6000 = regex::Regex::new(r"rx\s*6\d{3}").unwrap();

    if rx9000.is_match(&n) {
        ("rdna4".into(), true)
    } else if rx7000.is_match(&n) {
        ("rdna3".into(), true)
    } else if rx6000.is_match(&n) {
        ("rdna2".into(), false)
    } else if n.contains("radeon") || n.contains("amd") {
        ("other".into(), false)
    } else {
        ("unknown".into(), false)
    }
}

fn family_message(family: &str, name: &str) -> String {
    match family {
        "rdna4" => format!("{name} (RDNA4) e compativel com o mod."),
        "rdna3" => format!("{name} (RDNA3) e compativel (relatos bem-vindos)."),
        "rdna2" => format!("{name} (RDNA2) nao e oficialmente suportada pelo mod."),
        "other" => format!("{name}: GPU AMD sem suporte oficial (requer RX 7000/9000)."),
        _ => format!("{name}: nao e uma GPU AMD RDNA3/RDNA4. O mod pode nao funcionar."),
    }
}
