extern crate serde;
extern crate reqwest;
extern crate tokio;
extern crate sha1;
extern crate url;
extern crate downloader;

use downloader::{mc_data, download};

const VER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
       println!("syntax: modpacker <version>");
       return Ok(());
    }

    let result = Box::new(reqwest::get(VER_MANIFEST)
        .await?
        .json::<mc_data::MojangVersionManifest>()
        .await?);
    
    let version = result.look_up_version(String::from(&*args[1])).unwrap();
    
    println!("downloading from {}", &version.url);

    let result = Box::new(reqwest::get(&version.url[..])
        .await?
        .json::<mc_data::mojang_version_data::MojangVersionData>()
        .await?);
    
    let instance_path = std::path::Path::new("./installations").join(&*args[1]);

    let download_successful = match download::install_to_directory(&result, &instance_path).await {
        Err(_) => false,
        _ => true
    };
    
    if download_successful {
        //launcher::launch_instance(&instance_path);
    }

    Ok(())
}
