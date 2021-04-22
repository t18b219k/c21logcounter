use std::path::Path;
use std::io::{Read, Write};
/// download mesa from github and install
fn inject_mesa<P:AsRef<Path>>(path:P) {
    let urls = [
        "https://raw.githubusercontent.com/C21Logcounter/c21logcounter/master/mesa/z.dll",
        "https://raw.githubusercontent.com/C21Logcounter/c21logcounter/master/mesa/dxcompiler.dll",
        "https://raw.githubusercontent.com/C21Logcounter/c21logcounter/master/mesa/dxil.dll",
        "https://raw.githubusercontent.com/C21Logcounter/c21logcounter/master/mesa/opengl32.dll",
    ];
    let file_names=[
        "z.dll",
        "dxcompiler.dll",
        "dxil.dll",
        "opengl32.dll"
    ];
    for (url,file_name) in urls.iter().zip(file_names.iter()) {
        let mut content = ureq::get(url).call().unwrap().into_reader();
        let mut data =Vec::new();
        content.read_to_end(&mut data);
        let path=path.as_ref().join(file_name);
        let mut file =std::fs::File::create(path).unwrap();
        file.write(&data);
        file.flush();

    }
}
