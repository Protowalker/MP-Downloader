use super::mc_data::mojang_version_data::{Artifact, MojangVersionData, Os};
use serde::{Deserialize, Serialize};
use std::path::Path;
use phf::phf_map;
use rayon::prelude::*;

const RESOURCE_URL: &str = "https://resources.download.minecraft.net";
static OS_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "macos" => "osx",
    "linux" => "linux",
    "windows" => "windows"
};

#[derive(Debug)]
pub enum InstallError {
    WebError(reqwest::Error),
    URLError(url::ParseError),
    IOError(std::io::Error),
    HashError(String),
    JSONError(serde_json::error::Error),
}

pub fn install_to_directory(
    version: &MojangVersionData,
    directory: &Path,
) -> Result<(), InstallError> {
    let resource_url = url::Url::parse(RESOURCE_URL)?;
    //Need a builder for temporary files so that we don't leave half an installation
    //let tmp_dir = tempfile::Builder::new().prefix("modpacker").tempdir()?;
    ///////////////First step: make sure assets folders exist
    let assets_path = Path::new("./assets");
    std::fs::create_dir_all(&assets_path.join("indexes"))?;
    std::fs::create_dir_all(&assets_path.join("objects"))?;
    ///////////////
    std::fs::create_dir_all(&directory)?;
    let client = reqwest::blocking::Client::new();

    let assets_filename = String::from(format!("{}{}", &version.assets, ".json"));

    //if asset index doesn't exist, create and populate it. Otherwise, carry on
    if let Err(_) = std::fs::read(
        Path::new("./assets/indexes/")
            .join(&version.assets)
            .with_extension("json"),
    ) {
        try_download_and_write(
            &version.asset_index.url,
            &*assets_path.join("indexes"),
            &assets_filename,
            Some(&client),
        )
        ?;
    }

    //if the objects folder isn't properly populated, create it
        let assets = std::fs::read_to_string(format!("./assets/indexes/{}", assets_filename))?;
        let objects: ResourceObjectData =
            serde_json::from_str::<ResourceData>(&assets[..])?.objects;
        objects.extra.par_iter().for_each(|(name, hash_data)| {
            //Files are stored in folders named with the first two characters in a hash.
            let dir = &assets_path.join("objects").join(&hash_data.hash[..2]);
            std::fs::create_dir_all(dir).unwrap();

            //Does the file exist? If so, is the hash correct?
            let download_necessary = match std::fs::read(&dir.join(&hash_data.hash[..])) {
                Err(_) => true,
                Ok(f) => sha1::Sha1::from(f).digest().to_string() != &hash_data.hash[..],
            };
            if download_necessary {
                try_download_and_write(
                    &resource_url
                        .join(&format!("{}/{}", &hash_data.hash[..2], &hash_data.hash[..])[..]).unwrap()
                        .to_string(),
                    &dir,
                    &hash_data.hash,
                    Some(&client),
                ).unwrap();

                println!("downloaded {}", name);
            }
        });
    //////
    //Next phase: installing libraries
    let lib_path = Path::new("./libraries");
    std::fs::create_dir_all(&lib_path)?;

    let (mut lib_artifacts, mut nat_artifacts) = get_needed_libraries(&version);
    lib_artifacts.append(&mut nat_artifacts);

    lib_artifacts.par_iter().for_each(|lib| {
        let path = lib.path.clone().unwrap();
        let mut path = lib_path.join(path);
        let name = path.file_name()
                        .unwrap_or(std::ffi::OsStr::new("THIS_IS_BAD_TELL_THE_DEV"))
                        .to_str()
                        .unwrap();
        let name = String::from(name);
        path.pop();
        try_download_and_write(&lib.url,
                               &path, 
                               &name,
                               Some(&client)).unwrap();
    });

    try_download_and_write(&version.downloads.client.url, 
                           &directory, 
                           &String::from("client.jar"),
                           Some(&client))?;
    
    let file = serde_json::to_string_pretty(version)?;
    let file_path = directory.join("version_info.json");
    let should_save_version = match std::fs::read_to_string(&file_path) {
        Err(_) => true,
        Ok(s) => s != file
    };

    if should_save_version {
        std::fs::write(file_path, file)?;
    }

    //Download logger data
    let file_path = directory.join("client.xml");
    let should_get_logger = match std::fs::read(&file_path) {
        Err(_) => true,
        Ok(b) => sha1::Sha1::from(b).digest().to_string() != version.logging.client.file.sha1 
    };
    
    if should_get_logger {
        let logging_client = &version.logging.client;
        try_download_and_write(&logging_client.file.url,
                               &directory,
                               &String::from("client.xml"),
                               Some(&client))?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct ResourceData {
    objects: ResourceObjectData,
}

#[derive(Serialize, Deserialize)]
struct ResourceObjectData {
    #[serde(flatten)]
    extra: std::collections::HashMap<String, HashData>,
}

#[derive(Serialize, Deserialize)]
struct HashData {
    hash: String,
    size: u32,
}

pub fn get_needed_libraries(version: &MojangVersionData) -> (Vec<Artifact>, Vec<Artifact>){
    let os_name = String::from(OS_MAP[std::env::consts::OS]);
    let mut libs: Vec<Artifact> = Vec::new();
    let mut nats: Vec<Artifact> = Vec::new();

    for lib in &version.libraries {
        let mut should_download_artifact = false;
        if let Some(rules) = lib.rules.as_ref() {
            for rule in rules {
                let allow = rule.action == "allow";

                //When there's a rule called allow with no OS, it means you can skip it because the
                //default is to allow
                if std::mem::discriminant(&None) == std::mem::discriminant(&rule.os) && allow {
                    should_download_artifact = true;
                    continue;
                }

                if let Some(Os {
                    name: Some(name), ..
                }) = &rule.os
                {
                    if name == &os_name {
                        should_download_artifact = allow;
                        if !allow {
                            break;
                        }
                    }
                    
                }
                if let Some(Os {
                    arch: Some(arch), ..
                }) = &rule.os
                {
                    if arch == std::env::consts::ARCH {
                        should_download_artifact = allow;
                        if !allow {
                            break;
                        }
                    }
                }
            }
        } else {
            should_download_artifact = true;
        }

        //if there's an artifact, add it to the list and check for natives
        if should_download_artifact {
            if let Some(library) = &lib.downloads.artifact {
                libs.push(library.clone());
            }
        
            //if there are any platform-specific artifacts (classifiers,) download those as well
            if let Some(classifiers) = &lib.downloads.classifiers {
                let classifier = match &*os_name {
                    "osx" => &classifiers.natives_osx,
                    "windows" => &classifiers.natives_windows,
                    "linux" => &classifiers.natives_linux,
                    _ => &None,
                };

                if let Some(classifier) = classifier {
                    nats.push(classifier.clone());
                }
            }
        
        }
    }



    (libs, nats)
}

fn _download_artifact(artifact: &Artifact, lib_path: &Path) -> Result<(), InstallError> {

    let mut path: Vec<&str> = artifact.path.as_ref().unwrap().split("/").collect();

    let file_name = path.pop().unwrap().to_string();
    let path: std::path::PathBuf = path
        .iter()
        .fold(lib_path.to_path_buf(), |acc, &x| acc.join(x));

    match std::fs::read(path.join(&file_name)) {
        Err(_) => (),
        Ok(bytes) => if sha1::Sha1::from(bytes).digest().to_string() == artifact.sha1 {
            return Ok(());
        }
    }
    std::fs::create_dir_all(&path)?;

    try_download_and_write(&artifact.url, &path, &file_name, None)?;
    println!("downloaded {}", file_name);
    Ok(())
}

pub fn try_download_and_write(
    url: &String,
    dir: &Path,
    name: &String,
    client: Option<&reqwest::blocking::Client>,
) -> Result<(), InstallError> {
    let result = download_and_check(url, client)?;
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(name), result)?;
    Ok(())
}

fn download_and_check(
    url: &String,
    client: Option<&reqwest::blocking::Client>,
) -> Result<Vec<u8>, InstallError> {
    let result = match client {
        None => reqwest::blocking::get(&url[..])?.bytes(),
        Some(client) => client.get(url).send()?.bytes(),
    }?;

    Ok(result.iter().map(|v| -> u8 { *v }).collect())
}

impl From<std::io::Error> for InstallError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<reqwest::Error> for InstallError {
    fn from(error: reqwest::Error) -> Self {
        Self::WebError(error)
    }
}

impl From<serde_json::Error> for InstallError {
    fn from(error: serde_json::Error) -> Self {
        Self::JSONError(error)
    }
}
impl From<url::ParseError> for InstallError {
    fn from(error: url::ParseError) -> Self {
        Self::URLError(error)
    }
}


use std::fmt;
impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WebError(e) => e.fmt(f)?,
            Self::URLError(e) => e.fmt(f)?,
            Self::JSONError(e) => e.fmt(f)?,
            Self::IOError(e) => e.fmt(f)?,
            Self::HashError(e) => e.fmt(f)?,
        }
        write!(f, "")
    }
}

use std::error::Error;
impl Error for InstallError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::WebError(e) => e.source(),
            Self::URLError(e) => e.source(),
            Self::JSONError(e) => e.source(),
            Self::IOError(e) => e.source(),
            Self::HashError(e) => None,
        }
    }
}
