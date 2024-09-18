pub fn erode(v: &[u8], img_shape: (usize, usize)) -> Vec<u8> {
    let nw = img_shape.0;
    let nh = img_shape.1;
    let mut o = vec![0u8; nw * nh];
    for iw1 in 0..nw {
        for ih1 in 0..nh {
            let mut a = v[ih1 * nw + iw1];
            if iw1 > 0 {
                let iw0 = iw1 - 1;
                if ih1 > 0 {
                    let ih0 = ih1 - 1;
                    a = a.min(v[ih0 * nw + iw0]);
                }
                a = a.min(v[ih1 * nw + iw0]);
                if ih1 < nh - 1 {
                    let ih2 = ih1 + 1;
                    a = a.min(v[ih2 * nw + iw0]);
                }
            }
            {
                if ih1 > 0 {
                    let ih0 = ih1 - 1;
                    a = a.min(v[ih0 * nw + iw1]);
                }
                a = a.min(v[ih1 * nw + iw1]);
                if ih1 < nh - 1 {
                    let ih2 = ih1 + 1;
                    a = a.min(v[ih2 * nw + iw1]);
                }
            }
            if iw1 < nw - 1 {
                let iw2 = iw1 + 1;
                if ih1 > 0 {
                    let ih0 = ih1 - 1;
                    a = a.min(v[ih0 * nw + iw2]);
                }
                a = a.min(v[ih1 * nw + iw2]);
                if ih1 < nh - 1 {
                    let ih2 = ih1 + 1;
                    a = a.min(v[ih2 * nw + iw2]);
                }
            }
            o[ih1 * nw + iw1] = a;
        }
    }
    o
}
