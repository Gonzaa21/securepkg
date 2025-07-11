// zip_dir function to build cmd
use std::fs::File;
use zip::write::FileOptions;
use zip::CompressionMethod;
use std::path::Path;
use walkdir::WalkDir;

// this fn compress dirs in .zip
pub fn zip_dir(src_dir: &Path, dst_file: &Path) -> zip::result::ZipResult<()> {
    let file = File::create(dst_file)?; // Create file
    let mut zip = zip::ZipWriter::new(file); // zip writer

    // file options
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // travel dirs and subdirs in src_dir
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap(); // keep path structure and use a relative way

        if path.is_file() { // if is file:
            zip.start_file(name.to_string_lossy(), options)?; // create file
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?; // copy its content in zip archive
        } else if name.as_os_str().len() != 0 { // if is folder:
            zip.add_directory(name.to_string_lossy(), options)?; // add a dir to zip
        }
        zip.finish()?; // close zip
    }
    Ok(())
}
