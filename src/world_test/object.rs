use dot_vox::DotVoxData;



pub fn load_model(s:&str) -> dot_vox::DotVoxData
{
    let s = format!("assets/{}", s);
    let v = dot_vox::load(s.as_str()).unwrap();
    v
}
