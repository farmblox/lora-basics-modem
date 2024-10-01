use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lbm_dir = Path::new(&manifest_dir)
        .join("../lbm_lib")
        .canonicalize()
        .unwrap();
    let lbm_dir = lbm_dir.to_str().unwrap();

    make_clean(&lbm_dir);
    let mut include_paths = find_smtc_include_paths(&lbm_dir);
    include_paths.append(&mut find_sys_include_paths());

    let smtc_headers = find_smtc_headers(&lbm_dir);
    make_build(&lbm_dir);

    {
        // Where to search for the lib
        println!("cargo:rustc-link-search={lbm_dir}/build",);
        // Name of the lib - `verbatim` is necessary as this doesn't follow the standard
        // libsomething.a naming
        println!("cargo:rustc-link-lib=static:+verbatim=basic_modem.a");
    }

    let bindings = bindgen::Builder::default()
        .use_core()
        .clang_args(
            include_paths
                .iter()
                .map(|dir| String::from("-I") + dir)
                .collect::<Vec<String>>(),
        )
        .headers(
            smtc_headers
                .iter()
                .map(|header| header.to_str().unwrap().to_owned())
                .collect::<Vec<String>>(),
        );

    let bindings = bindings.generate().expect("Unable to generate bindings");

    // Write the bindings to the file included by src/lib.rs
    bindings
        .write_to_file("src/smtc_bindings.rs")
        .expect("Couldn't write bindings!");
}

fn find_smtc_headers(lbm_dir: &str) -> Vec<PathBuf> {
    let mut headers = vec![];
    for dir in find_smtc_include_paths(lbm_dir) {
        let entries = std::fs::read_dir(dir).unwrap();
        for entry_res in entries {
            if let Ok(entry) = entry_res {
                let file_name = entry.file_name().into_string().unwrap();
                if file_name.ends_with(".h") {
                    headers.push(entry.path().canonicalize().unwrap())
                }
            }
        }
    }

    // remove headers for radios we do not use
    headers.retain(|header| header.file_name().unwrap() != "ral_lr11xx_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "ralf_lr11xx_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "ral_sx127x_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "ralf_sx127x_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "ral_sx128x_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "ralf_sx128x_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "ral_llcc68_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "ralf_llcc68_bsp.h");
    headers.retain(|header| header.file_name().unwrap() != "smtc_modem_hal_dbg_trace.h");
    headers.retain(|header| header.file_name().unwrap() != "smtc_modem_geolocation_api.h");

    headers
}

fn find_smtc_include_paths(lbm_dir: &str) -> Vec<String> {
    let include_dirs = vec![
        format!("{lbm_dir}/smtc_modem_core/radio_drivers/sx126x_driver/src"),
        format!("{lbm_dir}/smtc_modem_core/smtc_ral/src"),
        format!("{lbm_dir}/smtc_modem_core/smtc_ralf/src"),
        format!("{lbm_dir}/smtc_modem_api/"),
        format!("{lbm_dir}/smtc_modem_hal/"),
    ];

    include_dirs
}

fn make_clean(lbm_dir: &str) {
    if !Command::new("make")
        .current_dir(lbm_dir)
        .arg("clean_sx1262")
        .output()
        .unwrap()
        .status
        .success()
    {
        panic!("Failed to clean LBM stack");
    }
}

fn make_build(lbm_dir: &str) {
    if !Command::new("make")
        .current_dir(lbm_dir)
        .env("DEBUG", "yes")
        .env("LBM_FUOTA", "yes")
        .env("LBM_DEVICE_MANAGEMENT", "yes")
        .env("LBM_MULTICAST", "yes")
        .env(
            "MCU_FLAGS",
            "-mcpu=cortex-m4 -mthumb -mabi=aapcs -mfpu=fpv4-sp-d16 -mfloat-abi=hard",
        )
        .arg("basic_modem_sx1262")
        .output()
        .unwrap()
        .status
        .success()
    {
        panic!("Failed to build LBM stack");
    }
}

fn find_sys_include_paths() -> Vec<String> {
    // Example output of shell command to find include paths:
    // ignoring duplicate directory "/Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../lib/gcc/../../lib/gcc/arm-none-eabi/10.3.1/include"
    // ignoring nonexistent directory "/Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../arm-none-eabi/usr/local/include"
    // ignoring duplicate directory "/Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../lib/gcc/../../lib/gcc/arm-none-eabi/10.3.1/include-fixed"
    // ignoring duplicate directory "/Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../lib/gcc/../../lib/gcc/arm-none-eabi/10.3.1/../../../../arm-none-eabi/include"
    // ignoring duplicate directory "/Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../arm-none-eabi/include"
    // #include "..." search starts here:
    // #include <...> search starts here:
    //  /Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../lib/gcc/arm-none-eabi/10.3.1/include
    //  /Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../lib/gcc/arm-none-eabi/10.3.1/include-fixed
    //  /Users/lena/Library/xPacks/@xpack-dev-tools/arm-none-eabi-gcc/10.3.1-2.3.1/.content/bin/../lib/gcc/arm-none-eabi/10.3.1/../../../../arm-none-eabi/include
    // End of search list.

    let res = Command::new("sh")
        .args(&["-c", "echo | arm-none-eabi-gcc -Wp,-v -x c - -fsyntax-only"])
        .output()
        .unwrap();

    if !res.status.success() {
        panic!("Failed to run GCC to collect include paths");
    }

    let stderr = String::from_utf8(res.stderr).unwrap();
    let mut started = false;
    let mut sys_include_paths = vec![];
    for line in stderr.lines() {
        if line == "#include <...> search starts here:" {
            started = true;
            continue;
        };
        if line == "End of search list." {
            break;
        };
        if started {
            sys_include_paths.push(line.trim().to_owned());
        };
    }
    sys_include_paths
}
