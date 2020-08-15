extern crate serde;
extern crate reqwest;
extern crate sha1;
extern crate url;
extern crate downloader;

use downloader::{mc_data, download};
use downloader::modloader::fabric;
use downloader::modloader::fabric::Stability;

const VER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
       println!("syntax: modpacker <version>");
       return Ok(());
    }

    let result = Box::new(reqwest::blocking::get(VER_MANIFEST)?
        .json::<mc_data::MojangVersionManifest>()?);

    
    let version = result.look_up_version(String::from(&*args[1])).unwrap();
    

    let result = Box::new(reqwest::blocking::get(&version.url[..])?
        .json::<mc_data::mojang_version_data::MojangVersionData>()?);
    
    let instance_path = std::path::Path::new("./installations").join(&*args[1]);
    
    let versions = fabric::get_game_versions(Stability::Stable)?;
    let fabric_build = versions.iter()
                                   .find(|ver| ver.version == String::from(&*args[1]));

    let fabric_version = if let Some(fabric_build) = fabric_build {
        fabric::get_fabric_builds_from_version(&fabric_build, Stability::Stable)?.remove(0)
    } else {
        println!("No fabric builds found for {}", &*args[1]);
        return Ok(());
    };

    println!("downloading from {}", &version.url);
    download::install_to_directory(&result, &instance_path)?;

    fabric::install_fabric_at_instance(fabric_version, &instance_path);



    Ok(())
}
