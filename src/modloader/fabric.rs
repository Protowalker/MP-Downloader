use std::path::Path;
use serde::{Serialize, Deserialize};
use super::super::types::{Or, Or::{First, Second}};

const VERSION_URL: &str = "https://meta.fabricmc.net/v2/versions";
pub enum Stability {
    Stable,
    Unstable,
    Both
}

pub fn install_fabric_at_instance(build: FabricBuild, instance_dir: &Path) {
    let lib_path = Path::new("./libraries");
    let libraries = build.launcher_meta.libraries;
    let libraries: Vec<FabricLibrary> = libraries.client.into_iter()
                    .chain(libraries.common.into_iter())
                    .collect();

    for lib in libraries.iter() {
        //TODO: Parse and install libraries from fabric
    }

    //TODO: create file for launcher to parse with changed class_path
    //and such
}

pub fn get_fabric_builds_from_version(version: &FabricGameVersion, stability: Stability ) -> Result<Vec<FabricBuild>, reqwest::Error> {
    let result: Vec<FabricBuild> = reqwest::blocking::get(reqwest::Url::parse(&format!("{}/loader/{}", VERSION_URL, version.version)).unwrap())?.json()?; 
    let result = if let Stability::Stable = stability {
        result.into_iter()
              .filter(|ver| ver.loader.stable)
              .map(|ver| ver)
              .collect::<Vec<FabricBuild>>()
    } else if let Stability::Unstable = stability {
        result.into_iter()
              .filter(|ver| !ver.loader.stable)
              .map(|ver| ver)
              .collect::<Vec<FabricBuild>>()
    } else { result };
    Ok(result)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricBuild {
    pub loader: FabricBuildLoader,
    pub intermediary: FabricBuildIntermediary,
    #[serde(alias = "launcherMeta")]
    pub launcher_meta: FabricBuildMeta
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricBuildLoader {
    separator: String,
    build: u32,
    maven: String,
    version: String,
    stable: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricBuildIntermediary {
    maven: String,
    version: String,
    stable: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricBuildMeta {
    version: u16,
    libraries: FabricBuildLibraries,
    #[serde(alias = "mainClass")]
    main_class: Or<FabricMainClass, String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricMainClass {
    client: String,
    server: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricBuildLibraries {
    client: Vec<FabricLibrary>,
    common: Vec<FabricLibrary>,
    server: Vec<FabricLibrary>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricLibrary {
    name: String,
    url: Option<String>
}

pub fn get_game_versions(stability: Stability) -> Result<Vec<FabricGameVersion>, reqwest::Error> {
    let result: Vec<FabricGameVersion> = reqwest::blocking::get(reqwest::Url::parse(&format!("{}/game", VERSION_URL)).unwrap())?.json()?;
    let result = if let Stability::Stable = stability {
        result.into_iter()
              .filter(|ver| ver.stable)
              .map(|ver| ver)
              .collect::<Vec<FabricGameVersion>>()
    } else if let Stability::Unstable = stability {
        result.into_iter()
              .filter(|ver| !ver.stable)
              .map(|ver| ver)
              .collect::<Vec<FabricGameVersion>>()
    } else { result };
    Ok(result)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricGameVersion {
    pub version: String,
    pub stable: bool
}
