use std::io::Read;
use del_msh_core::vtx2xyz::HasXyz;
use crate::Format::BinaryLittleEndian;

enum Format {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian
}

struct XyzRgb {
    xyz: [f64; 3],
    rgb: [u8; 3]
}

fn read_xyzrgb() -> anyhow::Result<Vec<XyzRgb>> {
    let file_path = "C:/Users/nobuy/Downloads/juice_box.ply";
    let file = std::fs::File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    use std::io::BufRead;
    let mut line = String::new();
    let hoge = reader.read_line(&mut line)?;
    assert_eq!(line, "ply\n");
    line.clear();
    let hoge = reader.read_line(&mut line)?;
    dbg!(&line);
    let strs: Vec<_> = line.split_whitespace().collect();
    assert_eq!(strs[0], "format");
    let format = match strs[1] {
        "binary_little_endian" => {
            BinaryLittleEndian
        },
        &_ => panic!(),
    };
    let hoge = reader.read_line(&mut line)?;
    dbg!(&line);
    line.clear();
    //
    let hoge = reader.read_line(&mut line)?; // element vertex
    dbg!(&line);
    let strs: Vec<_> = line.split_whitespace().collect();
    assert_eq!(strs[0], "element");
    use std::str::FromStr;
    let num_elem = usize::from_str(strs[2]).unwrap();
    dbg!(num_elem);
    //
    let hoge = reader.read_line(&mut line)?; // property double x
    let hoge = reader.read_line(&mut line)?; // property double y
    let hoge = reader.read_line(&mut line)?; // property double z
    let hoge = reader.read_line(&mut line)?; // property uchar red
    let hoge = reader.read_line(&mut line)?; // property uchar green
    let hoge = reader.read_line(&mut line)?; // property uchar blue
    line.clear();
    //
    let hoge = reader.read_line(&mut line)?; // property uchar green
    assert_eq!(line,"end_header\n");
    //
    let mut buf: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buf)?;
    let mut i_byte = 0usize;
    let mut vtx2xyzrgb: Vec<XyzRgb> = vec!();
    for i_elem in 0..num_elem {
        let i_bype = i_elem * (8*3 + 3);
        let x = f64::from_le_bytes(buf[i_byte..i_byte+8].try_into()? );
        let y = f64::from_le_bytes(buf[i_byte+8..i_byte+16].try_into()? );
        let z = f64::from_le_bytes(buf[i_byte+16..i_byte+24].try_into()? );
        let r = u8::from_le_bytes(buf[i_byte+24..i_byte+25].try_into()? );
        let g = u8::from_le_bytes(buf[i_byte+25..i_byte+26].try_into()? );
        let b = u8::from_le_bytes(buf[i_byte+26..i_byte+27].try_into()? );
        let xyzrgb = XyzRgb {
            xyz: [x,y,z],
            rgb: [r,g,b]
        };
        vtx2xyzrgb.push(xyzrgb);
    }
    Ok(vtx2xyzrgb)
}

impl del_msh_core::vtx2xyz::HasXyz<f64> for XyzRgb
{
    fn xyz(&self) -> [f64; 3] {
        self.xyz
    }
}

fn main() -> anyhow::Result<()>{
    let vtx2xyzrgb = read_xyzrgb()?;
    let aabb3 = del_msh_core::vtx2xyz::aabb3_from_points(&vtx2xyzrgb);
    dbg!(aabb3);
    Ok(())
}