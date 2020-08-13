use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MojangVersionManifest {
    pub latest: MojangVersionManifestLatest,
    pub versions: Vec<MojangReleaseProfile>,
}

impl MojangVersionManifest {
    pub fn look_up_version(&self, version: String) -> Option<&MojangReleaseProfile> {
        self.versions.iter().filter(|v| v.id == version).next()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MojangVersionManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MojangReleaseProfile {
    pub id: String,
    #[serde(rename = "type")]
    pub release_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

pub mod mojang_version_data {
    use crate::types::{Or, OrVec};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct MojangVersionData {
        pub arguments: Option<Arguments>,
        #[serde(rename = "assetIndex")]
        pub asset_index: AssetIndex,
        pub assets: String,
        pub downloads: Downloads,
        pub id: String,
        pub libraries: Vec<Library>,
        pub logging: Logging,
        #[serde(rename = "mainClass")]
        pub main_class: String,
        #[serde(rename = "minecraftArguments")]
        pub minecraft_arguments: Option<String>,
        #[serde(rename = "minimumLauncherVersion")]
        pub minimum_launcher_version: u16,
        #[serde(rename = "releaseTime")]
        pub release_time: String,
        pub time: String,
        #[serde(rename = "type")]
        pub release_type: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Arguments {
        pub game: Vec<Or<String, Argument>>,
        pub jvm: Vec<Or<String, Argument>>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Argument {
        pub rules: Option<Vec<Rule>>,
        pub value: OrVec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Logging {
        pub client: LoggingConfig,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct LoggingConfig {
        pub argument: String,
        pub file: File,
        #[serde(rename = "type")]
        pub file_type: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Library {
        pub downloads: LibraryDownload,
        pub extract: Option<Extract>,
        pub name: String,
        pub natives: Option<Natives>,
        pub rules: Option<Vec<Rule>>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct LibraryDownload {
        pub classifiers: Option<Classifiers>,
        pub artifact: Option<Artifact>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Extract {
        exclude: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Natives {
        linux: Option<String>,
        osx: Option<String>,
        windows: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Rule {
       pub action: String,
       pub os: Option<Os>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Os {
        pub name: Option<String>,
        pub arch: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Classifiers {
        #[serde(rename = "natives-linux")]
        pub natives_linux: Option<Artifact>,
        #[serde(rename = "natives-osx")]
        pub natives_osx: Option<Artifact>,
        #[serde(rename = "natives-windows")]
        pub natives_windows: Option<Artifact>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Downloads {
        pub client: Artifact,
        pub server: Artifact,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Artifact {
        pub path: Option<String>,
        pub sha1: String,
        pub size: u32,
        pub url: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct File {
        pub id: String,
        pub sha1: String,
        pub size: u32,
        pub url: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct AssetIndex {
        pub id: String,
        pub sha1: String,
        pub size: u32,
        #[serde(rename = "totalSize")]
        pub total_size: u32,
        pub url: String,
    }
}

//https://launchermeta.mojang.com/mc/game/version_manifest.json

//https://adfoc.us/serve/sitelinks/?id=271228&url=https://files.minecraftforge.net/maven/net/minecraftforge/forge/1.15.2-31.2.33/forge-1.15.2-31.2.33-universal.jar
