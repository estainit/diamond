//old_name_was isOlderThan
#[allow(dead_code, unused)]
pub fn is_older_than(v1: String, v_ref: String) -> i8 {
    let v1_value: i64 = convert_ver_to_val(v1);
    let v_ref_value: i64 = convert_ver_to_val(v_ref);
    if (v1_value < 0) || (v_ref_value < 0) {
        return -1;
    }
    if v1_value < v_ref_value { return 1; }
    return 0;
}

//old_name_was convertVerToVal
#[allow(dead_code, unused)]
pub fn convert_ver_to_val(version: String) -> i64 {
    let v_seg: Vec<&str> = version.split(".").collect();
    let mut version_segments: Vec<i64> = vec![];
    for &a_seg in v_seg.iter()
    {
        let seg_int2 = match a_seg.to_string().parse::<i64>() {
            Ok(i) => { i }
            Err(e) => {
                println!("convert_ver_to_val version, {:?}", version);
                println!("convert_ver_to_val v_seg, {:?}", v_seg);
                println!("convert_ver_to_val FAILED, {}", e);
                0
            }
        };
        // let ddseg_int = a_seg.to_string().parse::<i64>();
        let seg_int = a_seg.to_string().parse::<i64>().unwrap();
        if seg_int < 0 {
            return -1;
        }
        version_segments.push(seg_int);
    }

    let version_value: i64 = (1000000 * version_segments[0]) + (1000 * version_segments[1]) + version_segments[2];
    return version_value;
}

/*
}

bool VersionHandler::isValid(const String& version)
{
  VString v_seg = version.split(".");
  Vec<int> version_segments;
  for (auto a_seg: v_seg)
  {
    bool is_valid;
    int seg_int = a_seg.toInt(&is_valid);
    if (!is_valid || (seg_int < 0) || (a_seg != String::number(seg_int)))
      return false;
  }
  return true;
}


bool VersionHandler::isNewerThan(
  const String& v1,
  const String& v_ref)
{
  int64_t v1_value = convertVerToVal(v1);
  int64_t v_ref_value = convertVerToVal(v_ref);
  return (v1_value > v_ref_value);
}

 */

//old_name_was isValid
//old_name_was isValidVersionNumber
pub fn is_valid_version_number(version: &str) -> bool
{
    if version == ""
    { return false; }

    // let ver_segments = ver.split(".");
    // if (ver_segments.len() != 3)
    // return false;
    //
    // for (String aSeg : ver_segments)
    // {
    //     if (String::number(aSeg.toUInt()) != aSeg)
    //     return false;
    // }

    return true;
}
