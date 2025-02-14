use std::env::consts::OS;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

static DEFAULT_DIR_UNIX: &str = "/etc/panduza";

///
/// Provide the default directory where to put platform configs (tree.json, connection.json...)
///
pub fn system_default_config_dir() -> Result<PathBuf, std::io::Error> {
    match OS {
        "linux" => Ok(PathBuf::from_str(DEFAULT_DIR_UNIX).unwrap()),
        "windows" => Ok(PathBuf::from(dirs::public_dir().unwrap()).join("panduza")),
        os_name => {
            println!("!!! Unsupported => {:?} !!!", os_name);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unsupported OS",
            ));
        }
    }
}

///
/// TODO: change std::io::Error into a panduza Error
///
pub fn system_dyn_lib_extension() -> Result<String, std::io::Error> {
    match OS {
        "linux" => Ok("so".to_string()),
        "windows" => Ok("dll".to_string()),
        "macos" => Ok("dylib".to_string()),
        os_name => {
            println!("!!! Unsupported => {:?} !!!", os_name);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unsupported OS",
            ));
        }
    }
}

pub fn system_dyn_lib_prefix() -> Result<String, std::io::Error> {
    match OS {
        "linux" => Ok("lib".to_string()),
        "windows" => Ok("".to_string()),
        "macos" => Ok("lib".to_string()),
        os_name => {
            println!("!!! Unsupported => {:?} !!!", os_name);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unsupported OS",
            ));
        }
    }
}

///
///
///
pub fn system_default_device_tree_file() -> Result<PathBuf, std::io::Error> {
    let tree_path = system_default_config_dir().unwrap().join("tree.json");
    Ok(tree_path)
}

///
///
///
pub fn system_default_plugins_dir() -> Result<PathBuf, std::io::Error> {
    let tree_path = system_default_config_dir().unwrap().join("plugins");
    Ok(tree_path)
}

/// Location of the directory where logs should be stored
///
pub fn system_default_log_dir() -> Result<PathBuf, std::io::Error> {
    let tree_path = system_default_config_dir().unwrap().join("logs");
    Ok(tree_path)
}

///
/// Provide a list of all the possible location for plugins
///
pub fn system_plugins_dir_paths() -> Vec<PathBuf> {
    let mut res = Vec::new();
    // res.push(value);
    // a coté du binaire
    // si windows c:/
    let path = std::env::current_exe().unwrap();
    let parent = path.parent().unwrap();
    let ppp = parent.join("plugins");
    // println!("The current directory is {}", ppp.display()); // cd/plugins

    res.push(ppp);

    // main and alternate

    let default_path = system_default_plugins_dir().unwrap();

    res.push(default_path);

    return res;
}

///
/// Copy the plugin file 'plg_name' from 'plg_dir' into the system directory
///
pub fn system_copy_plugin_to_default_location(plg_dir: String, plg_name: &str) {
    let ext = system_dyn_lib_extension().unwrap();

    let pref = system_dyn_lib_prefix().unwrap();

    let plg_name_with_ext = format!("{}{}.{}", pref, plg_name, ext);

    let lib_path = PathBuf::from(plg_dir).join(&plg_name_with_ext);

    let system_plugins_dir = system_default_plugins_dir().unwrap();

    fs::create_dir_all(system_plugins_dir.clone()).unwrap();

    let system_plugins_path = system_plugins_dir.join(plg_name_with_ext);

    println!("*******");
    println!(
        "copy ({}) into ({})",
        lib_path.display(),
        system_plugins_path.display()
    );
    println!("*******");

    fs::copy(lib_path, system_plugins_path).unwrap();
}
