extern crate serde;
extern crate reqwest;
extern crate sha1;
extern crate url;
extern crate downloader;

use downloader::{mc_data, download};

const VER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
       println!("syntax: modpacker <version>");
       return Ok(());
    }

    let result = Box::new(reqwest::get(VER_MANIFEST)?
        .json::<mc_data::MojangVersionManifest>()?);

    
    let version = result.look_up_version(String::from(&*args[1])).unwrap();
    
    println!("downloading from {}", &version.url);

    let result = Box::new(reqwest::get(&version.url[..])?
        .json::<mc_data::mojang_version_data::MojangVersionData>()?);
    
    let instance_path = std::path::Path::new("./installations").join(&*args[1]);

    Ok(())
}
