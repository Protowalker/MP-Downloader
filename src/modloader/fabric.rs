use std::path::Path;
use serde::{Serialize, Deserialize};
use super::super::types::{Or, Or::{First, Second}};
use super::super::download::{InstallError, try_download_and_write};

const VERSION_URL: &str = "https://meta.fabricmc.net/v2/versions";
pub enum Stability {
    Stable,
    Unstable,
    Both
}

const MAVEN_URL: &str = "https://maven.fabricmc.net";

pub fn install_fabric_at_instance(build: FabricBuild, instance_dir: &Path) -> Result<(), InstallError> {
    let lib_path = Path::new("./libraries");
    let libraries = &build.launcher_meta.libraries;
    let mut libraries: Vec<&FabricLibrary> = libraries.client.iter()
                    .chain(libraries.common.iter())
                    .collect();
    //Add loader and intermediary to libraries
    let loader = FabricLibrary {
        name: build.loader.maven.clone(),
        url: Some(String::from(MAVEN_URL)) 
    };
    let intermediary = FabricLibrary {
        name: build.intermediary.maven.clone(),
        url: Some(String::from(MAVEN_URL))
    };
    libraries.push(&loader);
    libraries.push(&intermediary);

    for lib in libraries.iter() {
        if let FabricLibrary {name, url: Some(url)} = &lib {
            //library URLs don't include the path, only the domain
            let mut name = name.split(":");
            
            //All library names are split as such:
            //path.to.lib : unique-lib-name : version-identifier
            let path = name.next().unwrap();
            let id = name.next().unwrap();
            let version = name.next().unwrap();

            //turn path into an actual path
            let path = path.split(".")
                           .fold(String::from(""), |state, path| state + path + "/");
            
            let url = format!("{}/{}{}/{}", url, path, id, version);
            let filename = format!("{}-{}", id, version);

            let hash_url = reqwest::Url::parse(&format!("{}/{}.jar.sha1", url, filename))?;
            let hash = reqwest::blocking::get(hash_url)?
                                         .text()?;
            
            let lib_location = lib_path.join(path).join(id).join(version);
            let should_download = match std::fs::read(&lib_location.join(format!("{}.jar", filename))) {
                Err(_) => true,
                Ok(b) => sha1::Sha1::from(b).digest().to_string() != hash
            };

            if should_download {
                let jar_url = &format!("{}/{}.jar", url, filename);
                try_download_and_write(jar_url, &*lib_location, &format!("{}.jar", filename), None)?;
                println!("Installed {}", filename);
            }
        }
    }

    let launcher_data = serde_json::to_string_pretty(&build)?;

    std::fs::write(&instance_dir.with_file_name("fabric_info.json"), launcher_data)?;
    Ok(())
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
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    pub stable: bool
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
