extern crate serde;
extern crate reqwest;
extern crate sha1;
extern crate url;
extern crate clap;

use downloader::{mc_data, download};
use downloader::modloader::fabric;
use downloader::modloader::fabric::Stability;
use clap::{Arg, App, value_t};


const VER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let matches = App::new("downloader")
                       .version("0.1")
                       .author("Jackie Edwards <jacksonedwards6@gmail.com>")
                       .about("Downloads Minecraft and its modloaders")
                       .arg(
                           Arg::with_name("fabric")
                                .help("Download a version of fabric alongside this minecraft installation. For newest stable, just do '-f \"\"'")
                                .value_name("build")
                                .takes_value(true)
                                .short("f")
                                .long("fabric")
                                .required(false)
                                .empty_values(true)
                           )
                       .arg(
                           Arg::with_name("mc_version")
                                .help("the version of Minecraft you want to install.")
                                .required(true)
                           )
                       .get_matches();
    
    let mc_version = matches.value_of("mc_version").unwrap();
    

    let result = Box::new(reqwest::blocking::get(VER_MANIFEST)?
        .json::<mc_data::MojangVersionManifest>()?);

    
    let version = result.look_up_version(String::from(mc_version)).unwrap();
    

    let result = Box::new(reqwest::blocking::get(&version.url[..])?
        .json::<mc_data::mojang_version_data::MojangVersionData>()?);
    
    let instance_path = std::path::Path::new("./installations").join(String::from(mc_version));
    
    let mut fabric_version: Option<fabric::FabricBuild> = None;

    if matches.is_present("fabric") {
        let versions = fabric::get_game_versions(Stability::Stable)?;
        let fabric_build = versions.iter()
                                   .find(|ver| ver.version == String::from(mc_version));


        fabric_version = if let Some(fabric_build) = fabric_build {
            let mut builds =fabric::get_fabric_builds_from_version(&fabric_build, Stability::Stable)?;
            if let Ok(build) = value_t!(matches, "fabric", u32) {
                let build = builds.into_iter()
                      .find(|b| b.loader.build == build);
                if let Some(build) = build {
                    Some(build)
                } else {
                    println!("Fabric build not found.");
                    return Ok(());
                }
            } else {
                Some(builds.remove(0))
            }
        } else {
            println!("No fabric builds found for {}", mc_version);
            return Ok(());
        };
    }


    println!("downloading from {}", &version.url);
    download::install_to_directory(&result, &instance_path)?;

    if let Some(fabric_version) = fabric_version {
        println!("{:?}", fabric_version);
        fabric::install_fabric_at_instance(fabric_version, &instance_path.join(&version.id))?;
    }



    Ok(())
}
